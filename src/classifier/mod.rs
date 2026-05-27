mod assess;
mod collapse;
mod framework;
mod project;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{Classification, Framework, Ownership, PortEntry, Project, RawPort};

pub fn classify(raw: RawPort) -> PortEntry {
    let mut project_cache = HashMap::new();
    let mut framework_cache = HashMap::new();
    classify_with_caches(raw, &mut project_cache, &mut framework_cache)
}

fn classify_with_caches(
    raw: RawPort,
    project_cache: &mut HashMap<PathBuf, Option<Project>>,
    framework_cache: &mut HashMap<PathBuf, Option<Framework>>,
) -> PortEntry {
    let classification = determine_classification(&raw);
    let cwd = raw.cwd.clone().or_else(|| process_cwd(raw.pid));
    let project = cwd.and_then(|dir| {
        project_cache
            .entry(dir.clone())
            .or_insert_with(|| project::detect_project(&dir))
            .clone()
    });

    let framework_hint = framework::detect_framework(
        &raw.process_name,
        &raw.command_line,
        raw.parent_command_line.as_deref(),
        &project,
        framework_cache,
    );

    let project = project.map(|mut p| {
        if p.framework.is_none() {
            p.framework = framework_hint;
        }
        p
    });

    let all_addrs = vec![raw.local_addr.clone()];

    PortEntry {
        port: raw.port,
        protocol: raw.protocol,
        pid: raw.pid,
        process_name: raw.process_name,
        command_line: raw.command_line,
        state: raw.state,
        classification,
        project,
        local_addr: raw.local_addr,
        all_addrs,
        ownership: Ownership::Untracked,
    }
}

pub fn classify_all(raw_ports: Vec<RawPort>, watched_ports: &[u16]) -> Vec<PortEntry> {
    let mut project_cache: HashMap<PathBuf, Option<Project>> = HashMap::new();
    let mut framework_cache: HashMap<PathBuf, Option<Framework>> = HashMap::new();
    let classified: Vec<PortEntry> = raw_ports
        .into_iter()
        .map(|raw| classify_with_caches(raw, &mut project_cache, &mut framework_cache))
        .collect();
    let mut collapsed = collapse::collapse(classified);
    assess::assess(&mut collapsed, watched_ports);
    collapsed
}

fn determine_classification(raw: &RawPort) -> Classification {
    if is_docker(raw) {
        return Classification::Docker;
    }
    if is_ssh_tunnel(raw) {
        return Classification::SshTunnel;
    }
    if is_system(raw) {
        return Classification::System;
    }
    if is_browser(raw) {
        return Classification::Browser;
    }
    if is_database(raw) {
        return Classification::Database;
    }
    if is_message_queue(raw) {
        return Classification::MessageQueue;
    }
    if is_proxy(raw) {
        return Classification::Proxy;
    }
    if is_language_server(raw) {
        return Classification::LanguageServer;
    }
    if is_build_tool(raw) {
        return Classification::BuildTool;
    }
    if is_dev_server(raw) {
        return Classification::DevServer;
    }
    Classification::Unknown
}

fn is_docker(raw: &RawPort) -> bool {
    let name = &raw.process_name;
    let cmd = &raw.command_line;
    name.contains("docker-proxy")
        || name.starts_with("com.docker.")
        || name == "containerd"
        || name == "dockerd"
        || name == "podman"
        || name == "containerd-shim"
        || cmd.contains("docker")
}

fn is_ssh_tunnel(raw: &RawPort) -> bool {
    raw.process_name == "ssh"
        && (raw.command_line.contains(" -L ")
            || raw.command_line.contains(" -R ")
            || raw.command_line.contains(" -D "))
}

fn is_system(raw: &RawPort) -> bool {
    if raw.pid <= 1 {
        return true;
    }
    let system_procs = [
        "systemd",
        "systemd-resolve",
        "systemd-network",
        "NetworkManager",
        "avahi-daemon",
        "cupsd",
        "dnsmasq",
        "sshd",
        "ntpd",
        "chronyd",
        "dhclient",
        "dhcpcd",
        "bluetoothd",
        "pulseaudio",
        "pipewire",
        "wireplumber",
    ];
    system_procs.iter().any(|p| raw.process_name == *p)
}

fn is_browser(raw: &RawPort) -> bool {
    let browser_procs = [
        "chromium",
        "chrome",
        "google-chrome",
        "firefox",
        "firefox-esr",
        "brave",
        "vivaldi",
        "opera",
        "electron",
        "msedge",
        "Safari",
        "WebKit",
    ];
    browser_procs
        .iter()
        .any(|p| raw.process_name.contains(p) || raw.process_name == *p)
}

fn is_database(raw: &RawPort) -> bool {
    let db_procs = [
        "postgres",
        "postgresql",
        "mysqld",
        "mariadbd",
        "mariadb",
        "mongod",
        "mongos",
        "redis-server",
        "redis-sentinel",
        "valkey-server",
        "memcached",
        "clickhouse",
        "cockroach",
        "couchdb",
        "arangod",
        "neo4j",
        "influxd",
        "prometheus",
        "victoriametrics",
        "minio",
        "etcd",
        "consul",
        "vault",
        "surrealdb",
        "dragonflydb",
        "keydb-server",
        "tikv-server",
        "foundationdb",
        "sqlite3",
    ];
    db_procs
        .iter()
        .any(|p| raw.process_name == *p || raw.process_name.starts_with(p))
}

fn is_message_queue(raw: &RawPort) -> bool {
    let mq_procs = [
        "rabbitmq",
        "beam.smp",
        "kafka",
        "nats-server",
        "mosquitto",
        "emqx",
        "pulsar",
        "zeromq",
    ];
    let cmd = &raw.command_line;
    mq_procs.iter().any(|p| raw.process_name.contains(p))
        || cmd.contains("rabbitmq")
        || cmd.contains("kafka")
        || cmd.contains("nats-server")
}

fn is_proxy(raw: &RawPort) -> bool {
    let proxy_procs = [
        "nginx", "caddy", "traefik", "haproxy", "envoy", "squid", "varnish", "httpd", "apache2",
        "lighttpd",
    ];
    proxy_procs
        .iter()
        .any(|p| raw.process_name == *p || raw.process_name.starts_with(p))
}

fn is_language_server(raw: &RawPort) -> bool {
    let cmd = &raw.command_line;
    cmd.contains("rust-analyzer")
        || cmd.contains("typescript-language-server")
        || cmd.contains("tsserver")
        || cmd.contains("gopls")
        || cmd.contains("pylsp")
        || cmd.contains("pyright")
        || cmd.contains("lua-language-server")
        || cmd.contains("clangd")
        || cmd.contains("ccls")
        || cmd.contains("solargraph")
        || cmd.contains("ruby-lsp")
        || cmd.contains("elixir-ls")
        || cmd.contains("erlang_ls")
        || cmd.contains("jdtls")
        || cmd.contains("omnisharp")
        || cmd.contains("zls")
        || cmd.contains("haskell-language-server")
}

fn is_build_tool(raw: &RawPort) -> bool {
    let cmd = &raw.command_line;
    let name = &raw.process_name;

    if cmd.contains("webpack") && !cmd.contains("webpack-dev-server") {
        return true;
    }
    if cmd.contains("esbuild") && !cmd.contains("--serve") {
        return true;
    }
    if cmd.contains("turbopack") || cmd.contains("rollup") {
        return true;
    }
    if cmd.contains("parcel") && cmd.contains("watch") {
        return true;
    }
    if name == "gradle" || name == "gradlew" || name == "mvn" || name == "mvnw" {
        return true;
    }
    cmd.contains("tsc ") && cmd.contains("--watch")
}

fn is_dev_server(raw: &RawPort) -> bool {
    let dev_processes = [
        "node",
        "deno",
        "bun",
        "python",
        "python3",
        "python3.10",
        "python3.11",
        "python3.12",
        "python3.13",
        "ruby",
        "go",
        "cargo",
        "rustc",
        "java",
        "kotlin",
        "php",
        "php-fpm",
        "beam.smp",
        "mix",
        "uvicorn",
        "gunicorn",
        "hypercorn",
        "puma",
        "unicorn",
        "thin",
        "passenger",
        "iex",
        "elixir",
        "dotnet",
        "swift",
    ];

    let cmd = &raw.command_line;
    let name = &raw.process_name;

    if dev_processes
        .iter()
        .any(|p| name.ends_with(p) || name == *p)
    {
        return true;
    }

    cmd.contains("webpack-dev-server")
        || cmd.contains("--serve")
        || cmd.contains("dev server")
        || cmd.contains("npm run dev")
        || cmd.contains("yarn dev")
        || cmd.contains("pnpm dev")
        || cmd.contains("bun run dev")
        || cmd.contains("cargo run")
        || cmd.contains("go run")
        || cmd.contains("flask run")
        || cmd.contains("uvicorn")
        || cmd.contains("gunicorn")
        || cmd.contains("manage.py runserver")
        || cmd.contains("rails server")
        || cmd.contains("rails s")
        || cmd.contains("mix phx.server")
        || cmd.contains("php artisan serve")
        || cmd.contains("hugo server")
        || cmd.contains("hugo serve")
        || cmd.contains("jekyll serve")
        || cmd.contains("gatsby develop")
        || cmd.contains("npx")
}

#[cfg(target_os = "linux")]
fn process_cwd(pid: u32) -> Option<std::path::PathBuf> {
    if pid == 0 {
        return None;
    }
    std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

#[cfg(target_os = "macos")]
fn process_cwd(_pid: u32) -> Option<std::path::PathBuf> {
    // TODO: use proc_pidinfo / libproc on macOS
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn process_cwd(_pid: u32) -> Option<std::path::PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PortState, Protocol};

    fn raw(name: &str, cmd: &str) -> RawPort {
        RawPort {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 100,
            process_name: name.to_string(),
            command_line: cmd.to_string(),
            state: PortState::Listen,
            local_addr: "127.0.0.1:3000".to_string(),
            parent_pid: None,
            parent_command_line: None,
            cwd: None,
        }
    }

    fn raw_with_pid(name: &str, cmd: &str, pid: u32) -> RawPort {
        RawPort {
            pid,
            ..raw(name, cmd)
        }
    }

    // --- is_docker ---

    #[test]
    fn classifies_docker_proxy() {
        assert!(is_docker(&raw("docker-proxy", "")));
    }

    #[test]
    fn classifies_com_docker() {
        assert!(is_docker(&raw("com.docker.backend", "")));
    }

    #[test]
    fn classifies_containerd() {
        assert!(is_docker(&raw("containerd", "")));
    }

    #[test]
    fn classifies_dockerd() {
        assert!(is_docker(&raw("dockerd", "")));
    }

    #[test]
    fn classifies_podman() {
        assert!(is_docker(&raw("podman", "")));
    }

    #[test]
    fn classifies_docker_in_cmdline() {
        assert!(is_docker(&raw("runc", "docker run nginx")));
    }

    #[test]
    fn not_docker_for_node() {
        assert!(!is_docker(&raw("node", "node server.js")));
    }

    // --- is_ssh_tunnel ---

    #[test]
    fn classifies_ssh_local_forward() {
        assert!(is_ssh_tunnel(&raw(
            "ssh",
            "ssh -L 3000:localhost:3000 remote"
        )));
    }

    #[test]
    fn classifies_ssh_remote_forward() {
        assert!(is_ssh_tunnel(&raw(
            "ssh",
            "ssh -R 3000:localhost:3000 remote"
        )));
    }

    #[test]
    fn classifies_ssh_dynamic_forward() {
        assert!(is_ssh_tunnel(&raw("ssh", "ssh -D 1080 remote")));
    }

    #[test]
    fn not_ssh_tunnel_without_flags() {
        assert!(!is_ssh_tunnel(&raw("ssh", "ssh user@host")));
    }

    #[test]
    fn not_ssh_tunnel_if_not_ssh_process() {
        assert!(!is_ssh_tunnel(&raw("node", "ssh -L 3000:localhost:3000")));
    }

    // --- is_system ---

    #[test]
    fn classifies_pid_0_as_system() {
        assert!(is_system(&raw_with_pid("kernel", "", 0)));
    }

    #[test]
    fn classifies_pid_1_as_system() {
        assert!(is_system(&raw_with_pid("systemd", "", 1)));
    }

    #[test]
    fn classifies_systemd_resolve() {
        assert!(is_system(&raw("systemd-resolve", "")));
    }

    #[test]
    fn classifies_sshd() {
        assert!(is_system(&raw("sshd", "")));
    }

    #[test]
    fn classifies_cupsd() {
        assert!(is_system(&raw("cupsd", "")));
    }

    #[test]
    fn classifies_network_manager() {
        assert!(is_system(&raw("NetworkManager", "")));
    }

    #[test]
    fn classifies_pipewire() {
        assert!(is_system(&raw("pipewire", "")));
    }

    #[test]
    fn not_system_for_node() {
        assert!(!is_system(&raw("node", "")));
    }

    // --- is_browser ---

    #[test]
    fn classifies_chrome() {
        assert!(is_browser(&raw("chrome", "")));
    }

    #[test]
    fn classifies_firefox() {
        assert!(is_browser(&raw("firefox", "")));
    }

    #[test]
    fn classifies_electron() {
        assert!(is_browser(&raw("electron", "")));
    }

    #[test]
    fn classifies_brave() {
        assert!(is_browser(&raw("brave", "")));
    }

    #[test]
    fn not_browser_for_node() {
        assert!(!is_browser(&raw("node", "")));
    }

    // --- is_database ---

    #[test]
    fn classifies_postgres() {
        assert!(is_database(&raw("postgres", "")));
    }

    #[test]
    fn classifies_mysqld() {
        assert!(is_database(&raw("mysqld", "")));
    }

    #[test]
    fn classifies_redis_server() {
        assert!(is_database(&raw("redis-server", "")));
    }

    #[test]
    fn classifies_mongod() {
        assert!(is_database(&raw("mongod", "")));
    }

    #[test]
    fn classifies_clickhouse() {
        assert!(is_database(&raw("clickhouse", "")));
    }

    #[test]
    fn classifies_influxd() {
        assert!(is_database(&raw("influxd", "")));
    }

    #[test]
    fn classifies_etcd() {
        assert!(is_database(&raw("etcd", "")));
    }

    #[test]
    fn classifies_surrealdb() {
        assert!(is_database(&raw("surrealdb", "")));
    }

    #[test]
    fn not_database_for_node() {
        assert!(!is_database(&raw("node", "")));
    }

    // --- is_message_queue ---

    #[test]
    fn classifies_rabbitmq_process() {
        assert!(is_message_queue(&raw("rabbitmq", "")));
    }

    #[test]
    fn classifies_beam_smp() {
        assert!(is_message_queue(&raw("beam.smp", "")));
    }

    #[test]
    fn classifies_kafka_in_cmd() {
        assert!(is_message_queue(&raw(
            "java",
            "kafka.Kafka config.properties"
        )));
    }

    #[test]
    fn classifies_nats_server_in_cmd() {
        assert!(is_message_queue(&raw("unknown", "nats-server --port 4222")));
    }

    #[test]
    fn not_message_queue_for_node() {
        assert!(!is_message_queue(&raw("node", "node app.js")));
    }

    // --- is_proxy ---

    #[test]
    fn classifies_nginx() {
        assert!(is_proxy(&raw("nginx", "")));
    }

    #[test]
    fn classifies_caddy() {
        assert!(is_proxy(&raw("caddy", "")));
    }

    #[test]
    fn classifies_traefik() {
        assert!(is_proxy(&raw("traefik", "")));
    }

    #[test]
    fn classifies_haproxy() {
        assert!(is_proxy(&raw("haproxy", "")));
    }

    #[test]
    fn classifies_apache2() {
        assert!(is_proxy(&raw("apache2", "")));
    }

    #[test]
    fn not_proxy_for_node() {
        assert!(!is_proxy(&raw("node", "")));
    }

    // --- is_language_server ---

    #[test]
    fn classifies_rust_analyzer() {
        assert!(is_language_server(&raw("ra", "rust-analyzer")));
    }

    #[test]
    fn classifies_tsserver() {
        assert!(is_language_server(&raw("node", "tsserver")));
    }

    #[test]
    fn classifies_gopls() {
        assert!(is_language_server(&raw("gopls", "gopls serve")));
    }

    #[test]
    fn classifies_pyright() {
        assert!(is_language_server(&raw("node", "pyright --stdio")));
    }

    #[test]
    fn classifies_clangd() {
        assert!(is_language_server(&raw(
            "clangd",
            "clangd --background-index"
        )));
    }

    #[test]
    fn not_language_server_for_node() {
        assert!(!is_language_server(&raw("node", "node app.js")));
    }

    // --- is_build_tool ---

    #[test]
    fn classifies_webpack_without_dev_server() {
        assert!(is_build_tool(&raw("node", "webpack --watch")));
    }

    #[test]
    fn not_build_tool_for_webpack_dev_server() {
        assert!(!is_build_tool(&raw("node", "webpack-dev-server")));
    }

    #[test]
    fn classifies_esbuild_without_serve() {
        assert!(is_build_tool(&raw("node", "esbuild src/index.ts")));
    }

    #[test]
    fn not_build_tool_for_esbuild_serve() {
        assert!(!is_build_tool(&raw("node", "esbuild --serve")));
    }

    #[test]
    fn classifies_turbopack() {
        assert!(is_build_tool(&raw("node", "turbopack build")));
    }

    #[test]
    fn classifies_rollup() {
        assert!(is_build_tool(&raw("node", "rollup -c")));
    }

    #[test]
    fn classifies_parcel_watch() {
        assert!(is_build_tool(&raw("node", "parcel watch src/index.html")));
    }

    #[test]
    fn classifies_gradle() {
        assert!(is_build_tool(&raw("gradle", "gradle build")));
    }

    #[test]
    fn classifies_mvn() {
        assert!(is_build_tool(&raw("mvn", "mvn compile")));
    }

    #[test]
    fn classifies_tsc_watch() {
        assert!(is_build_tool(&raw("node", "tsc --watch")));
    }

    #[test]
    fn not_build_tool_for_node() {
        assert!(!is_build_tool(&raw("node", "node app.js")));
    }

    // --- is_dev_server ---

    #[test]
    fn classifies_node_as_dev_server() {
        assert!(is_dev_server(&raw("node", "node server.js")));
    }

    #[test]
    fn classifies_python3_as_dev_server() {
        assert!(is_dev_server(&raw("python3", "python3 -m http.server")));
    }

    #[test]
    fn classifies_deno() {
        assert!(is_dev_server(&raw("deno", "deno run server.ts")));
    }

    #[test]
    fn classifies_bun() {
        assert!(is_dev_server(&raw("bun", "bun run server.ts")));
    }

    #[test]
    fn classifies_cargo() {
        assert!(is_dev_server(&raw("cargo", "cargo run")));
    }

    #[test]
    fn classifies_uvicorn_process() {
        assert!(is_dev_server(&raw("uvicorn", "uvicorn main:app")));
    }

    #[test]
    fn classifies_uvicorn_in_cmd() {
        assert!(is_dev_server(&raw("python3", "uvicorn main:app")));
    }

    #[test]
    fn classifies_npm_run_dev() {
        assert!(is_dev_server(&raw("whatever", "npm run dev")));
    }

    #[test]
    fn classifies_yarn_dev() {
        assert!(is_dev_server(&raw("whatever", "yarn dev")));
    }

    #[test]
    fn classifies_flask_run() {
        assert!(is_dev_server(&raw("whatever", "flask run")));
    }

    #[test]
    fn classifies_rails_server() {
        assert!(is_dev_server(&raw("whatever", "rails server")));
    }

    #[test]
    fn classifies_rails_s() {
        assert!(is_dev_server(&raw("whatever", "rails s")));
    }

    #[test]
    fn classifies_hugo_server() {
        assert!(is_dev_server(&raw("whatever", "hugo server")));
    }

    #[test]
    fn classifies_npx() {
        assert!(is_dev_server(&raw("whatever", "npx serve")));
    }

    #[test]
    fn classifies_webpack_dev_server_as_dev() {
        assert!(is_dev_server(&raw("node", "webpack-dev-server")));
    }

    #[test]
    fn classifies_manage_py_runserver() {
        assert!(is_dev_server(&raw("python3", "manage.py runserver")));
    }

    #[test]
    fn classifies_mix_phx_server() {
        assert!(is_dev_server(&raw("beam.smp", "mix phx.server")));
    }

    #[test]
    fn classifies_dotnet() {
        assert!(is_dev_server(&raw("dotnet", "dotnet run")));
    }

    #[test]
    fn not_dev_server_for_sshd() {
        assert!(!is_dev_server(&raw("sshd", "sshd")));
    }

    // --- determine_classification priority ---

    #[test]
    fn docker_takes_priority_over_dev_server() {
        let r = raw("docker-proxy", "docker run node server.js");
        assert_eq!(determine_classification(&r), Classification::Docker);
    }

    #[test]
    fn ssh_tunnel_takes_priority_over_system() {
        let r = raw("ssh", "ssh -L 3000:localhost:3000 host");
        assert_eq!(determine_classification(&r), Classification::SshTunnel);
    }

    #[test]
    fn system_pid_0() {
        let r = raw_with_pid("kernel", "", 0);
        assert_eq!(determine_classification(&r), Classification::System);
    }

    #[test]
    fn unknown_for_unrecognized() {
        let r = raw("myapp", "./myapp --port 3000");
        assert_eq!(determine_classification(&r), Classification::Unknown);
    }

    #[test]
    fn classify_produces_correct_classification() {
        let r = raw("postgres", "postgres -D /data");
        let entry = classify(r);
        assert_eq!(entry.classification, Classification::Database);
        assert_eq!(entry.port, 3000);
    }

    #[test]
    fn classify_all_with_empty_input() {
        let result = classify_all(vec![], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn classify_all_sets_ownership() {
        let node = RawPort {
            port: 3000,
            pid: 100,
            ..raw("node", "node server.js")
        };
        let pg = RawPort {
            port: 5432,
            pid: 200,
            ..raw("postgres", "postgres -D /data")
        };
        let result = classify_all(vec![node, pg], &[3000, 5432]);
        let node_entry = result.iter().find(|e| e.process_name == "node").unwrap();
        assert_eq!(node_entry.ownership, Ownership::Owned);
        let pg_entry = result
            .iter()
            .find(|e| e.process_name == "postgres")
            .unwrap();
        assert_eq!(pg_entry.ownership, Ownership::Blocked);
    }
}
