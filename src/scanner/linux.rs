use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

use super::{ScanError, Scanner};
use crate::model::{PortState, Protocol, RawPort};

pub struct LinuxScanner;

impl Scanner for LinuxScanner {
    fn scan(&self) -> Result<Vec<RawPort>, ScanError> {
        let inode_pids = build_inode_pid_map();
        let uid_map = load_uid_map();
        let mut ports = Vec::new();
        ports.extend(parse_proc_net(
            "/proc/net/tcp",
            Protocol::Tcp,
            false,
            &inode_pids,
            &uid_map,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/tcp6",
            Protocol::Tcp,
            true,
            &inode_pids,
            &uid_map,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/udp",
            Protocol::Udp,
            false,
            &inode_pids,
            &uid_map,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/udp6",
            Protocol::Udp,
            true,
            &inode_pids,
            &uid_map,
        )?);

        for port in &mut ports {
            if port.process_name.is_empty() {
                if let Some(hint) = well_known_hint(port.port) {
                    port.process_name = hint.to_string();
                }
            }
        }

        Ok(ports)
    }
}

fn build_inode_pid_map() -> HashMap<String, u32> {
    let mut map = HashMap::new();
    let proc = Path::new("/proc");
    let entries = match fs::read_dir(proc) {
        Ok(e) => e,
        Err(_) => return map,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = proc.join(name_str).join("fd");
        let fds = match fs::read_dir(&fd_dir) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for fd in fds.flatten() {
            let link = match fs::read_link(fd.path()) {
                Ok(l) => l,
                Err(_) => continue,
            };
            let link_str = link.to_string_lossy();
            if let Some(inode) = link_str
                .strip_prefix("socket:[")
                .and_then(|s| s.strip_suffix(']'))
            {
                map.entry(inode.to_string()).or_insert(pid);
            }
        }
    }

    map
}

fn parse_proc_net(
    path: &str,
    protocol: Protocol,
    ipv6: bool,
    inode_pids: &HashMap<String, u32>,
    uid_map: &HashMap<u32, String>,
) -> Result<Vec<RawPort>, ScanError> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(ScanError::PermissionDenied)
        }
        Err(e) => return Err(ScanError::ProcessInfo(e)),
    };

    let mut ports = Vec::new();

    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }

        let local_addr_hex = fields[1];
        let remote_addr_hex = fields[2];
        let state_hex = fields[3];

        let state = match parse_state(state_hex) {
            Some(s) => s,
            None => continue,
        };

        let (addr_str, port) = if ipv6 {
            parse_addr_v6(local_addr_hex)?
        } else {
            parse_addr_v4(local_addr_hex)?
        };

        if port == 0 {
            continue;
        }

        // Field 7 (0-indexed) is the UID of the socket owner — world-readable
        let uid: Option<u32> = fields.get(7).and_then(|s| s.parse().ok());

        // Parse remote address for established connections
        let remote_addr = parse_remote_addr(remote_addr_hex, ipv6);

        let inode = fields[9];
        let pid = inode_pids.get(inode).copied();

        let info = match pid {
            Some(p) => read_process_info(p),
            None => ProcessInfo {
                name: String::new(),
                command_line: String::new(),
                parent_pid: None,
                parent_command_line: None,
                cwd: None,
            },
        };

        let user = uid.and_then(|u| uid_map.get(&u).cloned());

        ports.push(RawPort {
            port,
            protocol,
            pid: pid.unwrap_or(0),
            process_name: info.name,
            command_line: info.command_line,
            state,
            local_addr: format!("{}:{}", addr_str, port),
            parent_pid: info.parent_pid,
            parent_command_line: info.parent_command_line,
            cwd: info.cwd,
            uid,
            user,
            remote_addr,
        });
    }

    Ok(ports)
}

/// Parse a remote address from a /proc/net/tcp hex field.
/// Returns None if the address is all-zeros (no remote peer) or on parse error.
fn parse_remote_addr(addr_hex: &str, ipv6: bool) -> Option<String> {
    let result = if ipv6 {
        parse_addr_v6(addr_hex)
    } else {
        parse_addr_v4(addr_hex)
    };

    match result {
        Ok((addr, port)) => {
            if port == 0 {
                None
            } else {
                Some(format!("{}:{}", addr, port))
            }
        }
        Err(_) => None,
    }
}

fn load_uid_map() -> HashMap<u32, String> {
    let Ok(content) = fs::read_to_string("/etc/passwd") else {
        return HashMap::new();
    };
    parse_passwd_to_uid_map(&content)
}

fn parse_passwd_to_uid_map(content: &str) -> HashMap<u32, String> {
    content
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, ':');
            let name = parts.next()?;
            parts.next(); // skip password field
            let uid: u32 = parts.next()?.parse().ok()?;
            Some((uid, name.to_string()))
        })
        .collect()
}

fn parse_state(hex: &str) -> Option<PortState> {
    match hex {
        "0A" => Some(PortState::Listen),
        "01" => Some(PortState::Established),
        _ => None,
    }
}

fn parse_addr_v4(addr_port: &str) -> Result<(String, u16), ScanError> {
    let parts: Vec<&str> = addr_port.split(':').collect();
    if parts.len() != 2 {
        return Err(ScanError::Parse(format!("invalid addr:port: {addr_port}")));
    }

    let addr_hex = parts[0];
    let port = u16::from_str_radix(parts[1], 16)
        .map_err(|_| ScanError::Parse(format!("invalid port hex: {}", parts[1])))?;

    let addr_u32 = u32::from_str_radix(addr_hex, 16)
        .map_err(|_| ScanError::Parse(format!("invalid addr hex: {addr_hex}")))?;
    let ip = Ipv4Addr::from(addr_u32.to_be());

    Ok((ip.to_string(), port))
}

fn parse_addr_v6(addr_port: &str) -> Result<(String, u16), ScanError> {
    let parts: Vec<&str> = addr_port.split(':').collect();
    if parts.len() != 2 {
        return Err(ScanError::Parse(format!("invalid addr:port: {addr_port}")));
    }

    let addr_hex = parts[0];
    let port = u16::from_str_radix(parts[1], 16)
        .map_err(|_| ScanError::Parse(format!("invalid port hex: {}", parts[1])))?;

    if addr_hex.len() != 32 {
        return Err(ScanError::Parse(format!(
            "invalid ipv6 addr hex length: {addr_hex}"
        )));
    }

    // /proc/net/tcp6 stores IPv6 as four 32-bit words in host byte order
    let mut octets = [0u8; 16];
    for i in 0..4 {
        let word_hex = &addr_hex[i * 8..(i + 1) * 8];
        let word = u32::from_str_radix(word_hex, 16)
            .map_err(|_| ScanError::Parse(format!("invalid ipv6 word: {word_hex}")))?;
        let bytes = word.to_le_bytes();
        octets[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }

    let ip = Ipv6Addr::from(octets);
    Ok((format_ipv6(&ip), port))
}

fn format_ipv6(ip: &Ipv6Addr) -> String {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return v4.to_string();
    }
    if ip.is_unspecified() {
        return "::".to_string();
    }
    ip.to_string()
}

struct ProcessInfo {
    name: String,
    command_line: String,
    parent_pid: Option<u32>,
    parent_command_line: Option<String>,
    cwd: Option<PathBuf>,
}

fn read_process_info(pid: u32) -> ProcessInfo {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let cmdline = fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .unwrap_or_default()
        .replace('\0', " ")
        .trim()
        .to_string();

    let parent_pid = read_parent_pid(pid);

    let parent_command_line = parent_pid.and_then(|ppid| {
        fs::read_to_string(format!("/proc/{ppid}/cmdline"))
            .ok()
            .map(|s| s.replace('\0', " ").trim().to_string())
            .filter(|s| !s.is_empty())
    });

    let cwd = fs::read_link(format!("/proc/{pid}/cwd")).ok();

    ProcessInfo {
        name: comm,
        command_line: cmdline,
        parent_pid,
        parent_command_line,
        cwd,
    }
}

fn read_parent_pid(pid: u32) -> Option<u32> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in status.lines() {
        if let Some(val) = line.strip_prefix("PPid:\t") {
            return val.trim().parse().ok();
        }
    }
    None
}

fn well_known_hint(port: u16) -> Option<&'static str> {
    match port {
        // System / network
        21 => Some("ftp"),
        22 => Some("sshd"),
        23 => Some("telnet"),
        25 | 465 | 587 => Some("smtp"),
        53 => Some("dns"),
        67 | 68 => Some("dhcp"),
        69 => Some("tftp"),
        80 => Some("httpd"),
        110 => Some("pop3"),
        119 => Some("nntp"),
        123 => Some("ntpd"),
        143 => Some("imap"),
        161 | 162 => Some("snmp"),
        389 => Some("ldap"),
        443 => Some("https"),
        445 => Some("smb"),
        514 => Some("syslog"),
        515 => Some("lpd"),
        631 => Some("cupsd"),
        636 => Some("ldaps"),
        873 => Some("rsync"),
        993 => Some("imaps"),
        995 => Some("pop3s"),

        // Databases
        1433 => Some("mssql"),
        1521 => Some("oracle"),
        3306 => Some("mysql"),
        5432 => Some("postgres"),
        5433 => Some("postgres"),
        5984 => Some("couchdb"),
        6333 | 6334 => Some("qdrant"),
        7474 => Some("neo4j"),
        8086 => Some("influxdb"),
        8529 => Some("arangodb"),
        9042 => Some("cassandra"),
        9200 | 9300 => Some("elasticsearch"),
        11211 => Some("memcached"),
        19042 => Some("cassandra"),
        27017..=27019 => Some("mongodb"),
        28015 | 29015 => Some("rethinkdb"),

        // Caches / KV stores
        6379..=6381 => Some("redis"),
        11212 => Some("memcached"),

        // Message queues / streaming
        1883 | 8883 => Some("mqtt"),
        4222 => Some("nats"),
        5671 | 5672 => Some("rabbitmq"),
        6650 | 6651 => Some("pulsar"),
        9092 => Some("kafka"),
        9093 => Some("kafka-ssl"),
        9094 => Some("kafka"),
        15672 => Some("rabbitmq-mgmt"),
        15692 => Some("rabbitmq"),

        // Web servers / proxies / load balancers
        81 | 8080 | 8443 | 8888 => Some("httpd"),
        3128 => Some("squid"),
        8123 => Some("privoxy"),

        // Container / orchestration
        2375 | 2376 => Some("dockerd"),
        2377 => Some("docker-swarm"),
        2379 | 2380 => Some("etcd"),
        5000 => Some("registry"),
        6443 => Some("kubernetes-api"),
        8001 => Some("kubectl-proxy"),
        10250 | 10255 | 10256 => Some("kubelet"),

        // Monitoring / observability
        3000 => Some("grafana"),
        4040 => Some("spark-ui"),
        4317 | 4318 => Some("otel-collector"),
        9090 => Some("prometheus"),
        9091 => Some("prometheus-pushgateway"),
        9100 => Some("node-exporter"),
        9115 => Some("blackbox-exporter"),
        9411 => Some("zipkin"),
        14250 | 14268 | 14269 => Some("jaeger"),
        16686 => Some("jaeger-ui"),

        // CI/CD / developer tools
        8834 => Some("tenable"),
        9000 => Some("sonarqube"),
        9418 => Some("git-daemon"),
        50000 => Some("jenkins"),

        // HashiCorp stack
        8200 => Some("vault"),
        8300..=8302 => Some("consul"),
        8500 => Some("consul-ui"),
        8600 => Some("consul-dns"),

        // Common dev server ports (Node, React, Vue, etc.)
        1234 => Some("parcel"),
        3001..=3003 => Some("dev-server"),
        4000 | 4001 => Some("dev-server"),
        4200 => Some("angular"),
        5173 => Some("vite"),
        5174 => Some("vite"),
        6006 => Some("storybook"),
        7000 => Some("dev-server"),
        8000 => Some("dev-server"),
        8008 => Some("httpd"),
        8081..=8083 => Some("dev-server"),
        8100 => Some("ionic"),
        8787 => Some("wrangler"),
        9229 => Some("node-inspect"),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_state_listen() {
        assert_eq!(parse_state("0A"), Some(PortState::Listen));
    }

    #[test]
    fn parse_state_established() {
        assert_eq!(parse_state("01"), Some(PortState::Established));
    }

    #[test]
    fn parse_state_other_returns_none() {
        assert_eq!(parse_state("06"), None);
        assert_eq!(parse_state("02"), None);
        assert_eq!(parse_state("FF"), None);
        assert_eq!(parse_state("00"), None);
    }

    #[test]
    fn parse_state_lowercase_ignored() {
        assert_eq!(parse_state("0a"), None);
        assert_eq!(parse_state("01"), Some(PortState::Established));
    }

    #[test]
    fn parse_ipv4_loopback_port_3000() {
        let (addr, port) = parse_addr_v4("0100007F:0BB8").unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 3000);
    }

    #[test]
    fn parse_ipv4_any_port_8080() {
        let (addr, port) = parse_addr_v4("00000000:1F90").unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port, 8080);
    }

    #[test]
    fn parse_ipv4_port_22() {
        let (addr, port) = parse_addr_v4("00000000:0016").unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port, 22);
    }

    #[test]
    fn parse_ipv4_port_443() {
        let (addr, port) = parse_addr_v4("00000000:01BB").unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port, 443);
    }

    #[test]
    fn parse_ipv4_specific_addr() {
        let (addr, port) = parse_addr_v4("0100000A:0050").unwrap();
        assert_eq!(addr, "10.0.0.1");
        assert_eq!(port, 80);
    }

    #[test]
    fn parse_ipv6_any_port_443() {
        let (addr, port) = parse_addr_v6("00000000000000000000000000000000:01BB").unwrap();
        assert_eq!(addr, "::");
        assert_eq!(port, 443);
    }

    #[test]
    fn parse_ipv6_loopback() {
        let (addr, port) = parse_addr_v6("00000000000000000000000001000000:0050").unwrap();
        assert_eq!(port, 80);
        assert!(!addr.is_empty());
    }

    #[test]
    fn format_ipv6_unspecified() {
        assert_eq!(format_ipv6(&Ipv6Addr::UNSPECIFIED), "::");
    }

    #[test]
    fn format_ipv6_mapped_v4() {
        let ip = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001);
        assert_eq!(format_ipv6(&ip), "127.0.0.1");
    }

    #[test]
    fn format_ipv6_regular() {
        let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        let s = format_ipv6(&ip);
        assert!(s.contains("2001"));
    }

    #[test]
    fn parse_addr_v4_invalid_format() {
        assert!(parse_addr_v4("not_valid").is_err());
    }

    #[test]
    fn parse_addr_v4_invalid_hex() {
        assert!(parse_addr_v4("ZZZZZZZZ:0050").is_err());
    }

    #[test]
    fn parse_addr_v4_invalid_port_hex() {
        assert!(parse_addr_v4("00000000:ZZZZ").is_err());
    }

    #[test]
    fn parse_addr_v6_wrong_length() {
        assert!(parse_addr_v6("ABCD:01BB").is_err());
    }

    #[test]
    fn parse_addr_v6_invalid_word() {
        assert!(parse_addr_v6("ZZZZZZZZ000000000000000000000000:0050").is_err());
    }

    #[test]
    fn parse_proc_net_empty_content() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        std::fs::write(&path, "  sl  local_address ...\n").unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        );
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_missing_file() {
        let inode_pids = HashMap::new();
        let result = parse_proc_net(
            "/nonexistent/path",
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        );
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_short_fields_skipped() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        std::fs::write(&path, "header\nshort line\n").unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        );
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_valid_line() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        let line = "   0: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("  sl  local_address ...\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].port, 8080);
        assert_eq!(result[0].state, PortState::Listen);
    }

    #[test]
    fn parse_proc_net_skips_port_zero() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        let line = "   0: 00000000:0000 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_proc_net_resolves_inode_to_pid() {
        let mut inode_pids = HashMap::new();
        inode_pids.insert("99999".to_string(), 42);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        let line = "   0: 00000000:0050 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 99999 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, 42);
        assert_eq!(result[0].port, 80);
    }

    // --- UID parsing from /proc/net/tcp ---

    #[test]
    fn parse_proc_net_extracts_uid_field() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        // Field index 7 (0-based) is the UID — value 1000 here
        let line = "   0: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12345 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uid, Some(1000));
    }

    #[test]
    fn parse_proc_net_uid_zero_for_root() {
        let inode_pids = HashMap::new();
        let uid_map = HashMap::from([(0, "root".to_string())]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        // UID 0 = root
        let line = "   0: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 11111 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &uid_map,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uid, Some(0));
        assert_eq!(result[0].user, Some("root".to_string()));
    }

    // --- Remote address parsing ---

    #[test]
    fn parse_remote_addr_zero_returns_none() {
        // All-zeros remote addr means no peer (LISTEN state)
        assert_eq!(parse_remote_addr("00000000:0000", false), None);
    }

    #[test]
    fn parse_remote_addr_nonzero_port_returns_some() {
        // 192.168.1.1:54321 in hex: 0101A8C0:D431
        let result = parse_remote_addr("0101A8C0:D431", false);
        assert!(result.is_some());
        let addr = result.unwrap();
        assert!(addr.contains(':'));
    }

    #[test]
    fn parse_remote_addr_invalid_hex_returns_none() {
        assert_eq!(parse_remote_addr("ZZZZZZZZ:ZZZZ", false), None);
    }

    #[test]
    fn parse_proc_net_remote_addr_for_established() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        // State 01 = ESTABLISHED, remote = 192.168.1.100:443 (6401A8C0:01BB)
        let line = "   0: 0100007F:1F90 6401A8C0:01BB 01 00000000:00000000 00:00000000 00000000  1000        0 55555 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].remote_addr.is_some());
        let remote = result[0].remote_addr.as_ref().unwrap();
        assert!(remote.contains(':'));
    }

    #[test]
    fn parse_proc_net_remote_addr_none_for_listen() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        // State 0A = LISTEN, remote = 0.0.0.0:0
        let line = "   0: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("header\n{}\n", line)).unwrap();
        let result = parse_proc_net(
            path.to_str().unwrap(),
            Protocol::Tcp,
            false,
            &inode_pids,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].remote_addr, None);
    }

    // --- parse_passwd_to_uid_map ---

    #[test]
    fn parse_passwd_finds_root() {
        let content = "root:x:0:0:root:/root:/bin/bash\nnobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\n";
        let map = parse_passwd_to_uid_map(content);
        assert_eq!(map.get(&0), Some(&"root".to_string()));
        assert_eq!(map.get(&65534), Some(&"nobody".to_string()));
    }

    #[test]
    fn parse_passwd_missing_uid_returns_none() {
        let content = "root:x:0:0:root:/root:/bin/bash\n";
        let map = parse_passwd_to_uid_map(content);
        assert!(!map.contains.key(&9999));
    }

    #[test]
    fn parse_passwd_resolves_multiple_users() {
        let content = "root:x:0:0:root:/root:/bin/bash\ndaemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\nalice:x:1000:1000:Alice:/home/alice:/bin/bash\n";
        let map = parse_passwd_to_uid_map(content);
        assert_eq!(map.get(&0), Some(&"root".to_string()));
        assert_eq!(map.get(&1), Some(&"daemon".to_string()));
        assert_eq!(map.get(&1000), Some(&"alice".to_string()));
    }

    #[test]
    fn parse_passwd_empty_content() {
        let map = parse_passwd_to_uid_map("");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_passwd_malformed_lines_skipped() {
        let content = "root:x:0:0:root\nbadline\nalice:x:1000:1000:Alice:/home/alice:/bin/bash\n";
        let map = parse_passwd_to_uid_map(content);
        assert_eq!(map.get(&0), Some(&"root".to_string()));
        assert_eq!(map.get(&1000), Some(&"alice".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn resolve_uid_from_mock_passwd_root() {
        let mock_passwd =
            "root:x:0:0:root:/root:/bin/bash\nalice:x:1000:1000:Alice:/home/alice:/bin/bash\n";
        let uid_to_find: u32 = 0;
        let found = mock_passwd.lines().find_map(|line| {
            let parts: Vec<&str> = line.splitn(7, ':').collect();
            if parts.len() >= 3 {
                if let Ok(file_uid) = parts[2].parse::<u32>() {
                    if file_uid == uid_to_find {
                        return Some(parts[0].to_string());
                    }
                }
            }
            None
        });
        assert_eq!(found, Some("root".to_string()));
    }

    #[test]
    fn resolve_uid_from_mock_passwd_not_found() {
        let mock_passwd =
            "root:x:0:0:root:/root:/bin/bash\nalice:x:1000:1000:Alice:/home/alice:/bin/bash\n";
        let uid_to_find: u32 = 9999;
        let found: Option<String> = mock_passwd.lines().find_map(|line| {
            let parts: Vec<&str> = line.splitn(7, ':').collect();
            if parts.len() >= 3 {
                if let Ok(file_uid) = parts[2].parse::<u32>() {
                    if file_uid == uid_to_find {
                        return Some(parts[0].to_string());
                    }
                }
            }
            None
        });
        assert!(found.is_none());
    }

    // --- Expanded well_known_hint ---

    #[test]
    fn well_known_hint_ssh() {
        assert_eq!(well_known_hint(22), Some("sshd"));
    }

    #[test]
    fn well_known_hint_ftp() {
        assert_eq!(well_known_hint(21), Some("ftp"));
    }

    #[test]
    fn well_known_hint_telnet() {
        assert_eq!(well_known_hint(23), Some("telnet"));
    }

    #[test]
    fn well_known_hint_dns() {
        assert_eq!(well_known_hint(53), Some("dns"));
    }

    #[test]
    fn well_known_hint_dhcp() {
        assert_eq!(well_known_hint(67), Some("dhcp"));
        assert_eq!(well_known_hint(68), Some("dhcp"));
    }

    #[test]
    fn well_known_hint_http() {
        assert_eq!(well_known_hint(80), Some("httpd"));
    }

    #[test]
    fn well_known_hint_https() {
        assert_eq!(well_known_hint(443), Some("https"));
    }

    #[test]
    fn well_known_hint_smtp_variants() {
        assert_eq!(well_known_hint(25), Some("smtp"));
        assert_eq!(well_known_hint(465), Some("smtp"));
        assert_eq!(well_known_hint(587), Some("smtp"));
    }

    #[test]
    fn well_known_hint_ldap() {
        assert_eq!(well_known_hint(389), Some("ldap"));
        assert_eq!(well_known_hint(636), Some("ldaps"));
    }

    #[test]
    fn well_known_hint_smb() {
        assert_eq!(well_known_hint(445), Some("smb"));
    }

    #[test]
    fn well_known_hint_rsync() {
        assert_eq!(well_known_hint(873), Some("rsync"));
    }

    #[test]
    fn well_known_hint_imap() {
        assert_eq!(well_known_hint(143), Some("imap"));
        assert_eq!(well_known_hint(993), Some("imaps"));
    }

    #[test]
    fn well_known_hint_pop3() {
        assert_eq!(well_known_hint(110), Some("pop3"));
        assert_eq!(well_known_hint(995), Some("pop3s"));
    }

    #[test]
    fn well_known_hint_databases() {
        assert_eq!(well_known_hint(3306), Some("mysql"));
        assert_eq!(well_known_hint(5432), Some("postgres"));
        assert_eq!(well_known_hint(5433), Some("postgres"));
        assert_eq!(well_known_hint(27017), Some("mongodb"));
        assert_eq!(well_known_hint(27018), Some("mongodb"));
        assert_eq!(well_known_hint(1433), Some("mssql"));
        assert_eq!(well_known_hint(1521), Some("oracle"));
        assert_eq!(well_known_hint(5984), Some("couchdb"));
        assert_eq!(well_known_hint(9042), Some("cassandra"));
    }

    #[test]
    fn well_known_hint_redis() {
        assert_eq!(well_known_hint(6379), Some("redis"));
        assert_eq!(well_known_hint(6380), Some("redis"));
    }

    #[test]
    fn well_known_hint_rabbitmq() {
        assert_eq!(well_known_hint(5672), Some("rabbitmq"));
        assert_eq!(well_known_hint(5671), Some("rabbitmq"));
        assert_eq!(well_known_hint(15672), Some("rabbitmq-mgmt"));
    }

    #[test]
    fn well_known_hint_kafka() {
        assert_eq!(well_known_hint(9092), Some("kafka"));
        assert_eq!(well_known_hint(9093), Some("kafka-ssl"));
    }

    #[test]
    fn well_known_hint_nats() {
        assert_eq!(well_known_hint(4222), Some("nats"));
    }

    #[test]
    fn well_known_hint_mqtt() {
        assert_eq!(well_known_hint(1883), Some("mqtt"));
        assert_eq!(well_known_hint(8883), Some("mqtt"));
    }

    #[test]
    fn well_known_hint_pulsar() {
        assert_eq!(well_known_hint(6650), Some("pulsar"));
    }

    #[test]
    fn well_known_hint_misc() {
        assert_eq!(well_known_hint(631), Some("cupsd"));
        assert_eq!(well_known_hint(6333), Some("qdrant"));
        assert_eq!(well_known_hint(9090), Some("prometheus"));
        assert_eq!(well_known_hint(9200), Some("elasticsearch"));
        assert_eq!(well_known_hint(9300), Some("elasticsearch"));
        assert_eq!(well_known_hint(11211), Some("memcached"));
    }

    #[test]
    fn well_known_hint_etcd() {
        assert_eq!(well_known_hint(2379), Some("etcd"));
        assert_eq!(well_known_hint(2380), Some("etcd"));
    }

    #[test]
    fn well_known_hint_docker() {
        assert_eq!(well_known_hint(2375), Some("dockerd"));
        assert_eq!(well_known_hint(2376), Some("dockerd"));
    }

    #[test]
    fn well_known_hint_kubernetes() {
        assert_eq!(well_known_hint(6443), Some("kubernetes-api"));
        assert_eq!(well_known_hint(10250), Some("kubelet"));
    }

    #[test]
    fn well_known_hint_monitoring() {
        assert_eq!(well_known_hint(3000), Some("grafana"));
        assert_eq!(well_known_hint(9100), Some("node-exporter"));
        assert_eq!(well_known_hint(9411), Some("zipkin"));
        assert_eq!(well_known_hint(16686), Some("jaeger-ui"));
    }

    #[test]
    fn well_known_hint_vault_consul() {
        assert_eq!(well_known_hint(8200), Some("vault"));
        assert_eq!(well_known_hint(8500), Some("consul-ui"));
    }

    #[test]
    fn well_known_hint_dev_tools() {
        assert_eq!(well_known_hint(4200), Some("angular"));
        assert_eq!(well_known_hint(5173), Some("vite"));
        assert_eq!(well_known_hint(6006), Some("storybook"));
        assert_eq!(well_known_hint(9229), Some("node-inspect"));
    }

    #[test]
    fn well_known_hint_git_daemon() {
        assert_eq!(well_known_hint(9418), Some("git-daemon"));
    }

    #[test]
    fn well_known_hint_unknown_port() {
        assert_eq!(well_known_hint(12345), None);
        assert_eq!(well_known_hint(0), None);
        assert_eq!(well_known_hint(65535), None);
    }
}
