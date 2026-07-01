# Architecture

`whoseportisitanyway` is a small pipeline: scan the OS for listening/established sockets, enrich each one with process and project metadata, classify it, then render the result as a TUI, a one-shot table, or JSON.

```
                 ┌────────────┐
                 │  scanner   │  OS-specific: reads /proc (Linux) or
                 │            │  process/socket tables (macOS)
                 └─────┬──────┘
                       │ Vec<RawPort>
                       ▼
                 ┌────────────┐
                 │  docker    │  Optional enrichment via the Docker
                 │ (optional) │  daemon socket (container/compose labels)
                 └─────┬──────┘
                       │
                       ▼
                 ┌────────────┐
                 │ classifier │  project detection, framework detection,
                 │            │  ownership assessment, dedup/collapse
                 └─────┬──────┘
                       │ Vec<PortEntry>
                       ▼
            ┌──────────┴───────────┐
            ▼                      ▼
       ┌─────────┐            ┌─────────┐
       │   tui   │            │   cli   │  snapshot / why / list
       └─────────┘            └─────────┘
```

## Data model (`src/model.rs`)

- **`RawPort`** — what the scanner produces: port, protocol, pid, process name/command line, socket state, local/remote address, parent process info, cwd, uid/user. This is the raw, unclassified fact the OS gives us.
- **`PortEntry`** — what everything downstream consumes: `RawPort` plus `Classification`, an optional `Project` (name, root directory, detected `Framework`), and `Ownership`.
- **`Classification`** — `DevServer`, `Database`, `Docker`, `BuildTool`, `LanguageServer`, `Proxy`, `Browser`, `MessageQueue`, `SshTunnel`, `System`, `Unknown`.
- **`Framework`** — a flat enum of ~25 frameworks/tools it can recognize (Vite, Next.js, Rails, Django, Axum, etc.), detected from the process's command line or its project's manifest file.
- **`Ownership`** — `Owned` / `Blocked` / `Untracked`, derived from the `watched_ports` list in [config](#configuration-config).

## Scanning (`src/scanner/`)

```rust
pub trait Scanner {
    fn scan(&self) -> Result<Vec<RawPort>, ScanError>;
}

pub fn create_scanner() -> Box<dyn Scanner>;
```

`create_scanner` is compiled per-target: `scanner::linux::LinuxScanner` on Linux, `scanner::macos::MacosScanner` on macOS, selected with `#[cfg(target_os = ...)]`. Everything above the `Scanner` trait is platform-agnostic — adding a new OS backend means implementing the trait, not touching the classifier, TUI, or CLI.

`ScanError` distinguishes I/O failures, parse failures, and `PermissionDenied` (surfaced to the user as a hint to re-run elevated, since some other users' processes are only fully inspectable with more privilege — see [Docker enrichment](#docker-enrichment-srcdockerrs) for how this is avoided for containerized ports specifically).

## Docker enrichment (`src/docker.rs`)

Without root, the scanner can't read another user's `/proc/<pid>/...`, so root-owned `docker-proxy` listeners normally surface with no process, type, or project information. `docker.rs` closes that gap by querying the Docker daemon directly:

- Speaks a minimal, dependency-free HTTP/1.0 client over the daemon's Unix socket (`/var/run/docker.sock`), which is readable by any member of the `docker` group — no elevation needed.
- Issues a single read-only `GET /containers/json` with a 400ms timeout and a 4MB response cap.
- Builds a `DockerIndex` mapping published host port → `{ container name, compose project, compose service, working dir }`, sourced from container labels (`com.docker.compose.project`, etc.) — which is more detail than `/proc` alone would give even with root.
- If the socket is absent, unreadable, or the daemon doesn't respond in time, every lookup degrades to `None` and behavior is identical to not having Docker enrichment at all. This module never mutates daemon state.

`classify_all` threads an `Option<&DockerIndex>` through so classification stays a pure function of `(RawPort, Option<DockerIndex lookup>)`.

## Classification (`src/classifier/`)

Runs as a short pipeline over the raw scan results:

1. **`project.rs`** — walks upward from a process's `cwd` looking for one of `project_markers` (`package.json`, `Cargo.toml`, `go.mod`, `Gemfile`, `.git`, etc., configurable — see [Configuration](../README.md#configuration)) to find the project root and derive a display name.
2. **`framework.rs`** — given the process's command line, its parent's command line, and the detected `Project`, matches against known framework signatures (e.g. `vite`, `next dev`, `rails server`) to set `Project.framework`. Results are cached per project root (`HashMap<PathBuf, Option<Framework>>`) so repeated scans don't redo the same filesystem/string work for ports belonging to the same project.
3. **`assess.rs`** — sets `Ownership` by checking the entry's port against the configured `watched_ports`: not in the list → `Untracked`; in the list and present → `Owned`; in the list but the state doesn't match what's expected → `Blocked`.
4. **`collapse.rs`** — some processes bind the same port multiple times (e.g. dual-stack IPv4/IPv6, or multiple threads). `collapse` groups `RawPort`s by `(port, pid)` and merges them into a single `PortEntry` with all addresses attached (`PortEntry.all_addrs`), so the UI shows one row per logical service.

## Configuration (`src/config/`)

`Config` is a `#[serde(default)]` struct loaded from an optional TOML file (via the `dirs` crate to find the platform config directory); any field absent from the file falls back to the constant in `src/config/defaults.rs`. There is no required configuration — the tool is fully usable with zero setup.

## CLI and TUI (`src/cli/`, `src/tui/`)

`src/run.rs` is the single entrypoint shared by every binary alias (`whoseportisitanyway` and `whose-port`, see [`src/bin/whose-port.rs`](../src/bin/whose-port.rs)) — it parses arguments with `clap` and dispatches:

- No subcommand → `tui::run(&config)`, an interactive `ratatui`/`crossterm` app (`src/tui/mod.rs` holds `App`/`View` state; `table.rs`, `detail.rs`, `confirm.rs` render each view; `keybindings.rs` maps key events to state transitions, including sending `SIGTERM` to kill a selected process via `nix`).
- `snapshot [--json]` → one-shot table or JSON dump (`src/cli/snapshot.rs`).
- `why <port>` → prints a plain-English explanation of a single port (`src/cli/why.rs`).
- `list [--plain]` → JSON or tab-separated listing, meant for piping into other tools (`src/cli/list.rs`).

Keeping the parse/dispatch logic in the library crate (`src/run.rs`) rather than in `main.rs` is what lets both binary entrypoints stay a one-line shim while `clap` still reports whichever name was actually invoked.
