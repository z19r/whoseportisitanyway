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
        let mut ports = Vec::new();
        ports.extend(parse_proc_net(
            "/proc/net/tcp",
            Protocol::Tcp,
            false,
            &inode_pids,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/tcp6",
            Protocol::Tcp,
            true,
            &inode_pids,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/udp",
            Protocol::Udp,
            false,
            &inode_pids,
        )?);
        ports.extend(parse_proc_net(
            "/proc/net/udp6",
            Protocol::Udp,
            true,
            &inode_pids,
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
        });
    }

    Ok(ports)
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
        22 => Some("sshd"),
        25 | 465 | 587 => Some("smtp"),
        53 => Some("dns"),
        80 => Some("httpd"),
        443 => Some("https"),
        631 => Some("cupsd"),
        1433 => Some("mssql"),
        3306 => Some("mysql"),
        5432 => Some("postgres"),
        5672 | 5671 => Some("rabbitmq"),
        6379 | 6380 => Some("redis"),
        6333 => Some("qdrant"),
        9090 => Some("prometheus"),
        9200 | 9300 => Some("elasticsearch"),
        11211 => Some("memcached"),
        15672 => Some("rabbitmq-mgmt"),
        27017 => Some("mongodb"),
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
    fn well_known_hint_ssh() {
        assert_eq!(well_known_hint(22), Some("sshd"));
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
    fn well_known_hint_databases() {
        assert_eq!(well_known_hint(3306), Some("mysql"));
        assert_eq!(well_known_hint(5432), Some("postgres"));
        assert_eq!(well_known_hint(27017), Some("mongodb"));
        assert_eq!(well_known_hint(1433), Some("mssql"));
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
    fn well_known_hint_misc() {
        assert_eq!(well_known_hint(53), Some("dns"));
        assert_eq!(well_known_hint(631), Some("cupsd"));
        assert_eq!(well_known_hint(6333), Some("qdrant"));
        assert_eq!(well_known_hint(9090), Some("prometheus"));
        assert_eq!(well_known_hint(9200), Some("elasticsearch"));
        assert_eq!(well_known_hint(9300), Some("elasticsearch"));
        assert_eq!(well_known_hint(11211), Some("memcached"));
    }

    #[test]
    fn well_known_hint_unknown_port() {
        assert_eq!(well_known_hint(12345), None);
        assert_eq!(well_known_hint(0), None);
        assert_eq!(well_known_hint(65535), None);
    }

    #[test]
    fn parse_proc_net_empty_content() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        std::fs::write(&path, "  sl  local_address ...\n").unwrap();
        let result = parse_proc_net(path.to_str().unwrap(), Protocol::Tcp, false, &inode_pids);
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_missing_file() {
        let inode_pids = HashMap::new();
        let result = parse_proc_net("/nonexistent/path", Protocol::Tcp, false, &inode_pids);
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_short_fields_skipped() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        std::fs::write(&path, "header\nshort line\n").unwrap();
        let result = parse_proc_net(path.to_str().unwrap(), Protocol::Tcp, false, &inode_pids);
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_proc_net_valid_line() {
        let inode_pids = HashMap::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tcp");
        let line = "   0: 00000000:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 12345 1 0000000000000000 100 0 0 10 0";
        std::fs::write(&path, format!("  sl  local_address ...\n{}\n", line)).unwrap();
        let result =
            parse_proc_net(path.to_str().unwrap(), Protocol::Tcp, false, &inode_pids).unwrap();
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
        let result =
            parse_proc_net(path.to_str().unwrap(), Protocol::Tcp, false, &inode_pids).unwrap();
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
        let result =
            parse_proc_net(path.to_str().unwrap(), Protocol::Tcp, false, &inode_pids).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, 42);
        assert_eq!(result[0].port, 80);
    }
}
