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
