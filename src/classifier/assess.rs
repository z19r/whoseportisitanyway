use crate::model::{Classification, Ownership, PortEntry};

pub fn assess(entries: &mut [PortEntry], watched_ports: &[u16]) {
    for entry in entries.iter_mut() {
        entry.ownership = determine_ownership(entry, watched_ports);
    }
}

fn determine_ownership(entry: &PortEntry, watched_ports: &[u16]) -> Ownership {
    if watched_ports.is_empty() {
        return Ownership::Untracked;
    }

    if !watched_ports.contains(&entry.port) {
        return Ownership::Untracked;
    }

    match entry.classification {
        Classification::DevServer => Ownership::Owned,
        Classification::Docker
        | Classification::SshTunnel
        | Classification::System
        | Classification::Database
        | Classification::Proxy
        | Classification::Browser
        | Classification::MessageQueue
        | Classification::BuildTool
        | Classification::LanguageServer
        | Classification::Unknown => Ownership::Blocked,
    }
}
