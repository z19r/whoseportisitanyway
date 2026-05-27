use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PortState {
    Listen,
    Established,
}

impl fmt::Display for PortState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortState::Listen => write!(f, "LISTEN"),
            PortState::Established => write!(f, "ESTABLISHED"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Classification {
    DevServer,
    Database,
    Docker,
    BuildTool,
    LanguageServer,
    Proxy,
    Browser,
    MessageQueue,
    SshTunnel,
    System,
    Unknown,
}

impl fmt::Display for Classification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Classification::DevServer => write!(f, "Dev Server"),
            Classification::Database => write!(f, "Database"),
            Classification::Docker => write!(f, "Docker"),
            Classification::BuildTool => write!(f, "Build Tool"),
            Classification::LanguageServer => write!(f, "Lang Server"),
            Classification::Proxy => write!(f, "Proxy"),
            Classification::Browser => write!(f, "Browser"),
            Classification::MessageQueue => write!(f, "Msg Queue"),
            Classification::SshTunnel => write!(f, "SSH Tunnel"),
            Classification::System => write!(f, "System"),
            Classification::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Framework {
    Vite,
    Next,
    Remix,
    Astro,
    SvelteKit,
    Nuxt,
    Gatsby,
    Turbopack,
    Webpack,
    Expo,
    Storybook,
    Nest,
    Express,
    Fastify,
    Rails,
    Django,
    Flask,
    FastAPI,
    Spring,
    Gin,
    Phoenix,
    Laravel,
    Hugo,
    Actix,
    Axum,
    Rocket,
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::Vite => write!(f, "Vite"),
            Framework::Next => write!(f, "Next.js"),
            Framework::Remix => write!(f, "Remix"),
            Framework::Astro => write!(f, "Astro"),
            Framework::SvelteKit => write!(f, "SvelteKit"),
            Framework::Nuxt => write!(f, "Nuxt"),
            Framework::Gatsby => write!(f, "Gatsby"),
            Framework::Turbopack => write!(f, "Turbopack"),
            Framework::Webpack => write!(f, "webpack"),
            Framework::Expo => write!(f, "Expo"),
            Framework::Storybook => write!(f, "Storybook"),
            Framework::Nest => write!(f, "NestJS"),
            Framework::Express => write!(f, "Express"),
            Framework::Fastify => write!(f, "Fastify"),
            Framework::Rails => write!(f, "Rails"),
            Framework::Django => write!(f, "Django"),
            Framework::Flask => write!(f, "Flask"),
            Framework::FastAPI => write!(f, "FastAPI"),
            Framework::Spring => write!(f, "Spring"),
            Framework::Gin => write!(f, "Gin"),
            Framework::Phoenix => write!(f, "Phoenix"),
            Framework::Laravel => write!(f, "Laravel"),
            Framework::Hugo => write!(f, "Hugo"),
            Framework::Actix => write!(f, "Actix"),
            Framework::Axum => write!(f, "Axum"),
            Framework::Rocket => write!(f, "Rocket"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Ownership {
    Owned,
    Blocked,
    Untracked,
}

impl fmt::Display for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ownership::Owned => write!(f, "Owned"),
            Ownership::Blocked => write!(f, "Blocked"),
            Ownership::Untracked => write!(f, "—"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawPort {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub process_name: String,
    pub command_line: String,
    pub state: PortState,
    pub local_addr: String,
    pub parent_pid: Option<u32>,
    pub parent_command_line: Option<String>,
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub framework: Option<Framework>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub process_name: String,
    pub command_line: String,
    pub state: PortState,
    pub classification: Classification,
    pub project: Option<Project>,
    pub local_addr: String,
    pub all_addrs: Vec<String>,
    pub ownership: Ownership,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_display() {
        assert_eq!(Protocol::Tcp.to_string(), "TCP");
        assert_eq!(Protocol::Udp.to_string(), "UDP");
    }

    #[test]
    fn port_state_display() {
        assert_eq!(PortState::Listen.to_string(), "LISTEN");
        assert_eq!(PortState::Established.to_string(), "ESTABLISHED");
    }

    #[test]
    fn classification_display_all_variants() {
        assert_eq!(Classification::DevServer.to_string(), "Dev Server");
        assert_eq!(Classification::Database.to_string(), "Database");
        assert_eq!(Classification::Docker.to_string(), "Docker");
        assert_eq!(Classification::BuildTool.to_string(), "Build Tool");
        assert_eq!(Classification::LanguageServer.to_string(), "Lang Server");
        assert_eq!(Classification::Proxy.to_string(), "Proxy");
        assert_eq!(Classification::Browser.to_string(), "Browser");
        assert_eq!(Classification::MessageQueue.to_string(), "Msg Queue");
        assert_eq!(Classification::SshTunnel.to_string(), "SSH Tunnel");
        assert_eq!(Classification::System.to_string(), "System");
        assert_eq!(Classification::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn framework_display_all_variants() {
        assert_eq!(Framework::Vite.to_string(), "Vite");
        assert_eq!(Framework::Next.to_string(), "Next.js");
        assert_eq!(Framework::Remix.to_string(), "Remix");
        assert_eq!(Framework::Astro.to_string(), "Astro");
        assert_eq!(Framework::SvelteKit.to_string(), "SvelteKit");
        assert_eq!(Framework::Nuxt.to_string(), "Nuxt");
        assert_eq!(Framework::Gatsby.to_string(), "Gatsby");
        assert_eq!(Framework::Turbopack.to_string(), "Turbopack");
        assert_eq!(Framework::Webpack.to_string(), "webpack");
        assert_eq!(Framework::Expo.to_string(), "Expo");
        assert_eq!(Framework::Storybook.to_string(), "Storybook");
        assert_eq!(Framework::Nest.to_string(), "NestJS");
        assert_eq!(Framework::Express.to_string(), "Express");
        assert_eq!(Framework::Fastify.to_string(), "Fastify");
        assert_eq!(Framework::Rails.to_string(), "Rails");
        assert_eq!(Framework::Django.to_string(), "Django");
        assert_eq!(Framework::Flask.to_string(), "Flask");
        assert_eq!(Framework::FastAPI.to_string(), "FastAPI");
        assert_eq!(Framework::Spring.to_string(), "Spring");
        assert_eq!(Framework::Gin.to_string(), "Gin");
        assert_eq!(Framework::Phoenix.to_string(), "Phoenix");
        assert_eq!(Framework::Laravel.to_string(), "Laravel");
        assert_eq!(Framework::Hugo.to_string(), "Hugo");
        assert_eq!(Framework::Actix.to_string(), "Actix");
        assert_eq!(Framework::Axum.to_string(), "Axum");
        assert_eq!(Framework::Rocket.to_string(), "Rocket");
    }

    #[test]
    fn ownership_display() {
        assert_eq!(Ownership::Owned.to_string(), "Owned");
        assert_eq!(Ownership::Blocked.to_string(), "Blocked");
        assert_eq!(Ownership::Untracked.to_string(), "—");
    }

    #[test]
    fn protocol_equality() {
        assert_eq!(Protocol::Tcp, Protocol::Tcp);
        assert_ne!(Protocol::Tcp, Protocol::Udp);
    }

    #[test]
    fn classification_equality() {
        assert_eq!(Classification::Docker, Classification::Docker);
        assert_ne!(Classification::Docker, Classification::Database);
    }

    #[test]
    fn port_state_equality() {
        assert_eq!(PortState::Listen, PortState::Listen);
        assert_ne!(PortState::Listen, PortState::Established);
    }

    #[test]
    fn ownership_equality() {
        assert_eq!(Ownership::Owned, Ownership::Owned);
        assert_ne!(Ownership::Owned, Ownership::Blocked);
    }

    #[test]
    fn framework_equality() {
        assert_eq!(Framework::Next, Framework::Next);
        assert_ne!(Framework::Next, Framework::Vite);
    }

    #[test]
    fn raw_port_clone() {
        let raw = RawPort {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1234,
            process_name: "node".to_string(),
            command_line: "node server.js".to_string(),
            state: PortState::Listen,
            local_addr: "127.0.0.1:3000".to_string(),
            parent_pid: Some(1),
            parent_command_line: None,
            cwd: None,
        };
        let cloned = raw.clone();
        assert_eq!(cloned.port, 3000);
        assert_eq!(cloned.pid, 1234);
        assert_eq!(cloned.process_name, "node");
    }

    #[test]
    fn port_entry_serializes() {
        let entry = PortEntry {
            port: 8080,
            protocol: Protocol::Tcp,
            pid: 42,
            process_name: "node".to_string(),
            command_line: "node index.js".to_string(),
            state: PortState::Listen,
            classification: Classification::DevServer,
            project: None,
            local_addr: "0.0.0.0:8080".to_string(),
            all_addrs: vec!["0.0.0.0:8080".to_string()],
            ownership: Ownership::Untracked,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"port\":8080"));
        assert!(json.contains("\"DevServer\""));
    }

    #[test]
    fn project_with_framework_serializes() {
        let proj = Project {
            name: "myapp".to_string(),
            root: PathBuf::from("/tmp/myapp"),
            framework: Some(Framework::Next),
        };
        let json = serde_json::to_string(&proj).unwrap();
        assert!(json.contains("\"myapp\""));
        assert!(json.contains("\"Next\""));
    }
}
