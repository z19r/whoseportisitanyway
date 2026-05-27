use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::Path;

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

        let (process_name, command_line) = match pid {
            Some(p) => read_process_info(p),
            None => (String::new(), String::new()),
        };

        ports.push(RawPort {
            port,
            protocol: protocol.clone(),
            pid: pid.unwrap_or(0),
            process_name,
            command_line,
            state,
            local_addr: format!("{}:{}", addr_str, port),
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

fn read_process_info(pid: u32) -> (String, String) {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let cmdline = fs::read_to_string(format!("/proc/{pid}/cmdline"))
        .unwrap_or_default()
        .replace('\0', " ")
        .trim()
        .to_string();

    (comm, cmdline)
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
    }

    #[test]
    fn parse_ipv4_loopback_port_3000() {
        // 0100007F = 127.0.0.1 in little-endian, 0BB8 = 3000
        let (addr, port) = parse_addr_v4("0100007F:0BB8").unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 3000);
    }

    #[test]
    fn parse_ipv4_any_port_8080() {
        // 00000000 = 0.0.0.0, 1F90 = 8080
        let (addr, port) = parse_addr_v4("00000000:1F90").unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port, 8080);
    }

    #[test]
    fn parse_ipv6_any_port_443() {
        // 32 zeros = ::, 01BB = 443
        let (addr, port) = parse_addr_v6("00000000000000000000000000000000:01BB").unwrap();
        assert_eq!(addr, "::");
        assert_eq!(port, 443);
    }

    #[test]
    fn format_ipv6_unspecified() {
        let ip = Ipv6Addr::UNSPECIFIED;
        assert_eq!(format_ipv6(&ip), "::");
    }

    #[test]
    fn parse_addr_v4_invalid_format() {
        assert!(parse_addr_v4("not_valid").is_err());
    }

    #[test]
    fn parse_addr_v6_wrong_length() {
        assert!(parse_addr_v6("ABCD:01BB").is_err());
    }
}
