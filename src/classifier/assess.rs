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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PortState, Protocol};

    fn make_entry(port: u16, classification: Classification) -> PortEntry {
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            pid: 100,
            process_name: "test".to_string(),
            command_line: "test".to_string(),
            state: PortState::Listen,
            classification,
            project: None,
            local_addr: format!("127.0.0.1:{port}"),
            all_addrs: vec![format!("127.0.0.1:{port}")],
            ownership: Ownership::Untracked,
        }
    }

    #[test]
    fn empty_watched_ports_means_untracked() {
        let entry = make_entry(3000, Classification::DevServer);
        assert_eq!(determine_ownership(&entry, &[]), Ownership::Untracked);
    }

    #[test]
    fn unwatched_port_is_untracked() {
        let entry = make_entry(3000, Classification::DevServer);
        assert_eq!(determine_ownership(&entry, &[8080]), Ownership::Untracked);
    }

    #[test]
    fn watched_dev_server_is_owned() {
        let entry = make_entry(3000, Classification::DevServer);
        assert_eq!(determine_ownership(&entry, &[3000]), Ownership::Owned);
    }

    #[test]
    fn watched_docker_is_blocked() {
        let entry = make_entry(3000, Classification::Docker);
        assert_eq!(determine_ownership(&entry, &[3000]), Ownership::Blocked);
    }

    #[test]
    fn watched_database_is_blocked() {
        let entry = make_entry(5432, Classification::Database);
        assert_eq!(determine_ownership(&entry, &[5432]), Ownership::Blocked);
    }

    #[test]
    fn watched_system_is_blocked() {
        let entry = make_entry(22, Classification::System);
        assert_eq!(determine_ownership(&entry, &[22]), Ownership::Blocked);
    }

    #[test]
    fn watched_unknown_is_blocked() {
        let entry = make_entry(9999, Classification::Unknown);
        assert_eq!(determine_ownership(&entry, &[9999]), Ownership::Blocked);
    }

    #[test]
    fn all_non_dev_classifications_are_blocked() {
        let blocked_types = [
            Classification::Docker,
            Classification::SshTunnel,
            Classification::System,
            Classification::Database,
            Classification::Proxy,
            Classification::Browser,
            Classification::MessageQueue,
            Classification::BuildTool,
            Classification::LanguageServer,
            Classification::Unknown,
        ];
        for class in blocked_types {
            let entry = make_entry(3000, class);
            assert_eq!(
                determine_ownership(&entry, &[3000]),
                Ownership::Blocked,
                "expected Blocked for {:?}",
                entry.classification
            );
        }
    }

    #[test]
    fn assess_mutates_ownership() {
        let mut entries = vec![
            make_entry(3000, Classification::DevServer),
            make_entry(5432, Classification::Database),
            make_entry(9999, Classification::Unknown),
        ];
        assess(&mut entries, &[3000, 5432]);
        assert_eq!(entries[0].ownership, Ownership::Owned);
        assert_eq!(entries[1].ownership, Ownership::Blocked);
        assert_eq!(entries[2].ownership, Ownership::Untracked);
    }
}
