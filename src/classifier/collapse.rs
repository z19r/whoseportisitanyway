use std::collections::HashMap;

use crate::model::PortEntry;

pub fn collapse(entries: Vec<PortEntry>) -> Vec<PortEntry> {
    let mut by_key: HashMap<(u16, u32), Vec<PortEntry>> = HashMap::new();

    for entry in entries {
        let key = (entry.port, entry.pid);
        by_key.entry(key).or_default().push(entry);
    }

    by_key
        .into_values()
        .map(|group| {
            let mut merged = group.into_iter().reduce(merge_entries).unwrap();
            merged.all_addrs.sort();
            merged.all_addrs.dedup();
            merged
        })
        .collect()
}

fn merge_entries(mut primary: PortEntry, other: PortEntry) -> PortEntry {
    if !primary.all_addrs.contains(&other.local_addr) {
        primary.all_addrs.push(other.local_addr.clone());
    }
    if is_wildcard(&primary.local_addr) && !is_wildcard(&other.local_addr) {
        // keep the wildcard — it's the broader listener
    } else if !is_wildcard(&primary.local_addr) && is_wildcard(&other.local_addr) {
        primary.local_addr = other.local_addr;
    }
    primary
}

fn is_wildcard(addr: &str) -> bool {
    addr.starts_with("0.0.0.0") || addr.starts_with("::") || addr.starts_with("*")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Classification, Ownership, PortState, Protocol};

    fn make_entry(port: u16, pid: u32, addr: &str) -> PortEntry {
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            pid,
            process_name: "node".to_string(),
            command_line: "node server.js".to_string(),
            state: PortState::Listen,
            classification: Classification::DevServer,
            project: None,
            local_addr: addr.to_string(),
            all_addrs: vec![addr.to_string()],
            ownership: Ownership::Untracked,
        }
    }

    #[test]
    fn is_wildcard_ipv4() {
        assert!(is_wildcard("0.0.0.0:3000"));
    }

    #[test]
    fn is_wildcard_ipv6() {
        assert!(is_wildcard(":::3000"));
    }

    #[test]
    fn is_wildcard_star() {
        assert!(is_wildcard("*:3000"));
    }

    #[test]
    fn is_not_wildcard_loopback() {
        assert!(!is_wildcard("127.0.0.1:3000"));
    }

    #[test]
    fn is_not_wildcard_specific_ip() {
        assert!(!is_wildcard("192.168.1.1:3000"));
    }

    #[test]
    fn collapse_single_entry_unchanged() {
        let entries = vec![make_entry(3000, 1, "127.0.0.1:3000")];
        let result = collapse(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].port, 3000);
    }

    #[test]
    fn collapse_merges_same_port_pid() {
        let entries = vec![
            make_entry(3000, 1, "127.0.0.1:3000"),
            make_entry(3000, 1, "0.0.0.0:3000"),
        ];
        let result = collapse(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].all_addrs.len(), 2);
        assert!(result[0].all_addrs.contains(&"127.0.0.1:3000".to_string()));
        assert!(result[0].all_addrs.contains(&"0.0.0.0:3000".to_string()));
    }

    #[test]
    fn collapse_keeps_wildcard_as_primary() {
        let entries = vec![
            make_entry(3000, 1, "127.0.0.1:3000"),
            make_entry(3000, 1, "0.0.0.0:3000"),
        ];
        let result = collapse(entries);
        assert_eq!(result[0].local_addr, "0.0.0.0:3000");
    }

    #[test]
    fn collapse_different_pids_not_merged() {
        let entries = vec![
            make_entry(3000, 1, "127.0.0.1:3000"),
            make_entry(3000, 2, "0.0.0.0:3000"),
        ];
        let result = collapse(entries);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn collapse_different_ports_not_merged() {
        let entries = vec![
            make_entry(3000, 1, "127.0.0.1:3000"),
            make_entry(8080, 1, "127.0.0.1:8080"),
        ];
        let result = collapse(entries);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn collapse_deduplicates_addrs() {
        let entries = vec![
            make_entry(3000, 1, "127.0.0.1:3000"),
            make_entry(3000, 1, "127.0.0.1:3000"),
        ];
        let result = collapse(entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].all_addrs.len(), 1);
    }

    #[test]
    fn collapse_empty_input() {
        let result = collapse(vec![]);
        assert!(result.is_empty());
    }
}
