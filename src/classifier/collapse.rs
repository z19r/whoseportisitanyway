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
