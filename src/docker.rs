//! Optional Docker enrichment via the daemon socket.
//!
//! Without root, the scanner cannot read another user's `/proc/<pid>/...`, so
//! root-owned `docker-proxy` listeners surface with no process, type, or
//! project. The Docker daemon socket, however, is readable by any member of the
//! `docker` group and exposes the published-port -> container -> compose-project
//! mapping directly. Querying it lets us label those ports without elevation —
//! and with *more* detail than `sudo` provides (compose project/service names
//! live in container labels, not in `/proc`).
//!
//! This module is read-only: it issues a single `GET /containers/json` and
//! never mutates daemon state. If the socket is absent or unreadable, every
//! entry point degrades to `None` and behavior is identical to before.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

/// Socket I/O timeout. A stale or wedged daemon must never freeze a scan.
const SOCKET_TIMEOUT: Duration = Duration::from_millis(400);

const LABEL_COMPOSE_PROJECT: &str = "com.docker.compose.project";
const LABEL_COMPOSE_SERVICE: &str = "com.docker.compose.service";
const LABEL_COMPOSE_WORKDIR: &str = "com.docker.compose.project.working_dir";

const DEFAULT_SOCKET: &str = "/var/run/docker.sock";

/// What Docker knows about the container publishing a given host port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DockerEndpoint {
    pub container_name: String,
    pub image: String,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
    pub working_dir: Option<PathBuf>,
}

impl DockerEndpoint {
    /// The best human label for this endpoint's project: the compose project
    /// when present, otherwise the bare container name (so a standalone
    /// container like `redis` still reads as a project rather than `—`).
    pub fn project_label(&self) -> Option<String> {
        self.compose_project
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| Some(self.container_name.clone()).filter(|s| !s.is_empty()))
    }
}

/// Maps each published host port to the container that owns it.
#[derive(Debug, Clone, Default)]
pub struct DockerIndex {
    by_port: HashMap<u16, DockerEndpoint>,
}

impl DockerIndex {
    /// Probe the local Docker daemon. Returns `None` when no socket is
    /// reachable or the daemon yields nothing usable — never errors outward.
    pub fn probe() -> Option<Self> {
        let socket = resolve_socket()?;
        let body = fetch_containers_json(&socket)?;
        let index = Self::from_containers_json(&body);
        if index.is_empty() {
            None
        } else {
            Some(index)
        }
    }

    pub fn get(&self, host_port: u16) -> Option<&DockerEndpoint> {
        self.by_port.get(&host_port)
    }

    pub fn is_empty(&self) -> bool {
        self.by_port.is_empty()
    }

    /// Build an index from a raw `/containers/json` body. Exposed within the
    /// crate for tests in other modules that need a populated index.
    #[cfg(test)]
    pub(crate) fn from_containers_json_for_tests(body: &str) -> Self {
        Self::from_containers_json(body)
    }

    /// Build the index from a raw `/containers/json` response body. Pure and
    /// total: malformed JSON yields an empty index rather than an error.
    fn from_containers_json(body: &str) -> Self {
        let containers: Vec<RawContainer> = serde_json::from_str(body).unwrap_or_default();
        let mut by_port = HashMap::new();

        for container in containers {
            let container_name = container
                .names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();
            let compose_project = nonempty(container.labels.get(LABEL_COMPOSE_PROJECT));
            let compose_service = nonempty(container.labels.get(LABEL_COMPOSE_SERVICE));
            let working_dir =
                nonempty(container.labels.get(LABEL_COMPOSE_WORKDIR)).map(PathBuf::from);

            for mapping in &container.ports {
                let Some(public_port) = mapping.public_port else {
                    continue; // not published to the host
                };
                // The daemon lists each published port once per host interface
                // (IPv4 + IPv6); first writer wins so the map stays 1:1.
                by_port
                    .entry(public_port)
                    .or_insert_with(|| DockerEndpoint {
                        container_name: container_name.clone(),
                        image: container.image.clone(),
                        compose_project: compose_project.clone(),
                        compose_service: compose_service.clone(),
                        working_dir: working_dir.clone(),
                    });
            }
        }

        Self { by_port }
    }
}

fn nonempty(value: Option<&String>) -> Option<String> {
    value.filter(|s| !s.is_empty()).cloned()
}

/// Resolve the Docker socket path, honoring `$DOCKER_HOST` (unix:// only) and
/// the rootless `$XDG_RUNTIME_DIR/docker.sock` before the system default.
fn resolve_socket() -> Option<PathBuf> {
    // An explicit unix:// DOCKER_HOST is authoritative. Unsupported schemes
    // (e.g. tcp://) are ignored so we fall through to local defaults rather than
    // disabling enrichment entirely.
    if let Some(path) = std::env::var("DOCKER_HOST")
        .ok()
        .and_then(|host| host.strip_prefix("unix://").map(PathBuf::from))
    {
        return path.exists().then_some(path);
    }

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let rootless = PathBuf::from(runtime_dir).join("docker.sock");
        if rootless.exists() {
            return Some(rootless);
        }
    }

    let default = PathBuf::from(DEFAULT_SOCKET);
    default.exists().then_some(default)
}

/// Issue `GET /containers/json` over the unix socket and return the response
/// body. HTTP/1.0 + `Connection: close` keeps the daemon from chunking, so the
/// body is everything after the header terminator. Returns `None` on any I/O
/// failure, non-200 status, or timeout.
fn fetch_containers_json(socket: &Path) -> Option<String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket).ok()?;
    stream.set_read_timeout(Some(SOCKET_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT)).ok()?;

    let request = "GET /containers/json HTTP/1.0\r\n\
                   Host: localhost\r\n\
                   Accept: application/json\r\n\
                   Connection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).ok()?;

    // Cap the read so a wedged or misbehaving daemon can't exhaust memory in the
    // TUI refresh loop; the container list is a few KB even on busy hosts.
    const MAX_RESPONSE: u64 = 4 * 1024 * 1024;
    let mut response = Vec::new();
    stream.take(MAX_RESPONSE).read_to_end(&mut response).ok()?;

    parse_http_body(&response)
}

/// Split an HTTP/1.0 response into status + body and return the body when the
/// status is 200. Lenient by design — we control the request and only ever
/// talk to the Docker daemon.
fn parse_http_body(response: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(response);
    let (head, body) = text.split_once("\r\n\r\n")?;
    // Docker answers HTTP/1.0 requests with a raw, unchunked body, so the body
    // is everything past the header terminator — no chunk decoding required.
    let status_line = head.lines().next()?;
    if status_line.split_whitespace().nth(1) != Some("200") {
        return None;
    }
    Some(body.to_string())
}

#[derive(Debug, serde::Deserialize)]
struct RawContainer {
    #[serde(default, rename = "Names")]
    names: Vec<String>,
    #[serde(default, rename = "Image")]
    image: String,
    #[serde(default, rename = "Labels")]
    labels: HashMap<String, String>,
    #[serde(default, rename = "Ports")]
    ports: Vec<RawPortMapping>,
}

#[derive(Debug, serde::Deserialize)]
struct RawPortMapping {
    #[serde(rename = "PublicPort")]
    public_port: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {
            "Names": ["/automem-flask-api-1"],
            "Image": "automem-flask-api",
            "Labels": {
                "com.docker.compose.project": "automem",
                "com.docker.compose.service": "flask-api",
                "com.docker.compose.project.working_dir": "/home/zrk/code/automem"
            },
            "Ports": [
                {"IP": "0.0.0.0", "PrivatePort": 8001, "PublicPort": 8001, "Type": "tcp"},
                {"IP": "::", "PrivatePort": 8001, "PublicPort": 8001, "Type": "tcp"}
            ]
        },
        {
            "Names": ["/redis"],
            "Image": "redis:7",
            "Labels": {},
            "Ports": [
                {"IP": "127.0.0.1", "PrivatePort": 6379, "PublicPort": 6379, "Type": "tcp"}
            ]
        },
        {
            "Names": ["/internal-only"],
            "Image": "busybox",
            "Labels": {},
            "Ports": [
                {"PrivatePort": 9000, "Type": "tcp"}
            ]
        }
    ]"#;

    #[test]
    fn indexes_compose_container_by_published_port() {
        // Arrange / Act
        let index = DockerIndex::from_containers_json(SAMPLE);

        // Assert
        let ep = index.get(8001).expect("port 8001 should be indexed");
        assert_eq!(ep.container_name, "automem-flask-api-1");
        assert_eq!(ep.compose_project.as_deref(), Some("automem"));
        assert_eq!(ep.compose_service.as_deref(), Some("flask-api"));
        assert_eq!(
            ep.working_dir.as_deref(),
            Some(Path::new("/home/zrk/code/automem"))
        );
    }

    #[test]
    fn dedupes_ipv4_and_ipv6_entries_for_same_port() {
        let index = DockerIndex::from_containers_json(SAMPLE);
        // 8001 appears twice (v4 + v6) in the fixture but maps once.
        assert_eq!(
            index.get(8001).unwrap().container_name,
            "automem-flask-api-1"
        );
    }

    #[test]
    fn standalone_container_falls_back_to_container_name_as_project() {
        let index = DockerIndex::from_containers_json(SAMPLE);
        let ep = index.get(6379).expect("redis should be indexed");
        assert_eq!(ep.compose_project, None);
        assert_eq!(ep.project_label().as_deref(), Some("redis"));
    }

    #[test]
    fn skips_containers_with_no_published_port() {
        let index = DockerIndex::from_containers_json(SAMPLE);
        assert!(index.get(9000).is_none());
    }

    #[test]
    fn compose_project_takes_precedence_in_label() {
        let index = DockerIndex::from_containers_json(SAMPLE);
        assert_eq!(
            index.get(8001).unwrap().project_label().as_deref(),
            Some("automem")
        );
    }

    #[test]
    fn malformed_json_yields_empty_index() {
        let index = DockerIndex::from_containers_json("not json at all");
        assert!(index.is_empty());
    }

    #[test]
    fn parses_http_body_only_on_200() {
        let ok = b"HTTP/1.0 200 OK\r\nContent-Type: application/json\r\n\r\n[]";
        assert_eq!(parse_http_body(ok).as_deref(), Some("[]"));

        let err = b"HTTP/1.0 500 Internal Server Error\r\n\r\noops";
        assert_eq!(parse_http_body(err), None);
    }
}
