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
