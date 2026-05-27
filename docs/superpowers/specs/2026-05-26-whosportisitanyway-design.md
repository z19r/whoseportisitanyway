# whoseportisitanyway — Design Spec

A cross-platform TUI that answers: which local ports are in use, who owns them, and is it my dev server or something blocking it?

Rust + ratatui. Linux + macOS in v1.

## Data Model

```
PortEntry {
    port: u16,
    protocol: TCP | UDP,
    pid: u32,
    process_name: String,
    command_line: String,
    state: Listen | Established,
    classification: Classification,
    project: Option<Project>,
}

Project {
    name: String,
    root: PathBuf,
    framework: Option<Framework>,
}

enum Classification { DevServer, Docker, SSHTunnel, System, Unknown }

enum Framework { Vite, Next, Expo, Storybook, Nest, Rails, Django, Flask, ... }
```

### Classification Heuristics

- **DevServer:** process is node/python/ruby/go AND CWD resolves to a project root
- **Docker:** process is `docker-proxy` or `com.docker.*`
- **SSHTunnel:** process is `ssh` with `-L`/`-R`/`-D` flags in argv
- **System:** pid 1, root-owned, or well-known system process
- **Unknown:** fallback

### Project Detection

Walk up from the process's CWD looking for marker files: `.git`, `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, `Gemfile`, `pom.xml`, `build.gradle`. Read the project name from the manifest when possible (e.g. `"name"` field in `package.json`).

### Framework Detection

Match against process argv and the detected project manifest:

| Framework  | Signal                                              |
|------------|-----------------------------------------------------|
| Next.js    | `next dev` or `next start` in argv                  |
| Vite       | `vite` in argv or vite in devDependencies           |
| Expo       | `expo start` in argv                                |
| Storybook  | `storybook dev` in argv                             |
| Nest       | `nest start` in argv or `@nestjs/core` in deps      |
| Rails      | `rails server` or `puma` in argv + `Gemfile` marker |
| Django     | `manage.py runserver` in argv                       |
| Flask      | `flask run` in argv                                 |

Extensible — new frameworks are a pattern match addition, not an architectural change.

## Architecture

Monolithic binary. Platform-specific code behind `#[cfg(target_os)]`. Single scan-classify pipeline shared by both TUI and CLI.

```
src/
├── main.rs              # clap arg parsing, dispatch to TUI or subcommand
├── scanner/
│   ├── mod.rs           # Scanner trait: fn scan() -> Vec<RawPort>
│   ├── linux.rs         # /proc/net/tcp + /proc/{pid}/ parsing
│   └── macos.rs         # libproc + netstat-style scanning
├── classifier/
│   ├── mod.rs           # classify(RawPort) -> PortEntry
│   ├── project.rs       # walk-up-from-CWD project root detection
│   └── framework.rs     # argv/process heuristics → Framework enum
├── model.rs             # PortEntry, Project, Classification, Framework
├── config/
│   ├── mod.rs           # Config struct, TOML loading, CLI flag merge
│   └── defaults.rs      # default watched ports, refresh interval, etc.
├── tui/
│   ├── mod.rs           # App struct, event loop, tick-based rescan
│   ├── table.rs         # port table widget (main view)
│   ├── detail.rs        # detail pane for selected port
│   ├── confirm.rs       # kill confirmation dialog
│   └── keybindings.rs   # key → action mapping
└── cli/
    ├── mod.rs           # subcommand dispatch
    ├── snapshot.rs      # one-shot table print to stdout
    ├── why.rs           # "why is port X in use?" drill-down
    └── list.rs          # machine-readable JSON/plain output
```

### Key Boundaries

- **scanner** enumerates ports + PIDs from the OS. Returns raw data. Platform-specific code lives only here.
- **classifier** enriches raw scan output — project roots, framework matching, classification assignment. All business logic lives here.
- **tui** is a thin rendering layer. Calls scan+classify on a tick, holds display state, renders via ratatui.
- **cli** is equally thin. Calls the same pipeline, formats output, exits.

## TUI Design

Three states:

### Main View

Full-width ratatui `Table` widget, sorted by port number by default. Status bar at bottom shows port count, project count, last refresh time, and key hints.

Columns: Port, Proto, PID, Process, Project, Framework, Type

### Detail Pane

Toggled with `Enter` or `d`. Shows full command line, project root path, CWD, process owner, listening address (0.0.0.0 vs 127.0.0.1), and available actions.

### Confirm Dialog

Triggered by `x`. Modal overlay: `Kill PID {pid} ({process})? This will stop {project}. [y/N]`

### Key Bindings

| Key          | Action                              |
|--------------|-------------------------------------|
| `j`/`k`, arrows | Navigate rows                   |
| `Enter`/`d`  | Toggle detail pane                  |
| `x`          | Kill selected (opens confirm)       |
| `s`          | Cycle sort (port, project, type, PID) |
| `f`          | Filter by type                      |
| `/`          | Search/filter by text               |
| `r`          | Force refresh                       |
| `q`/`Esc`    | Quit (or close detail/dialog)       |

### Refresh Strategy

Configurable tick interval (default 2s). The table diffs against the previous scan — only changed rows trigger a re-render to avoid flicker.

## CLI Subcommands

Bare invocation (`whoseportisitanyway`) launches the TUI. Subcommands:

### `snapshot`

Prints the port table to stdout and exits. Flags: `--json`, `--port <range>`.

### `why <port>`

Single-port drill-down. Prints PID, process, command, project, framework, type, listening address.

### `list`

Machine-oriented. Defaults to JSON, supports `--plain` for tab-separated. Designed for piping into `jq`, scripts, CI checks.

## Configuration

File: `~/.config/whoseportisitanyway/config.toml` (XDG convention). Works with no config file — all values have defaults.

```toml
refresh_interval = 2
watched_ports = [3000, 3001, 5173, 8080, 8081, 4200, 5432]
ignored_ports = []
sort_by = "port"
filter = "all"
project_markers = [
  ".git", "package.json", "Cargo.toml", "go.mod",
  "pyproject.toml", "Gemfile", "pom.xml", "build.gradle"
]
```

Loading order: built-in defaults → config file → CLI flags.

## Platform Support

### Linux

- Port enumeration: parse `/proc/net/tcp` and `/proc/net/tcp6`
- Process info: read `/proc/{pid}/cmdline`, `/proc/{pid}/cwd` (symlink), `/proc/{pid}/status`
- Requires no elevated privileges for processes owned by the current user. Root-owned process details may be limited.

### macOS

- Port enumeration: `libproc` APIs or parsing `lsof -iTCP -sTCP:LISTEN -P -n`
- Process info: `libproc::proc_pidpath`, `proc_pidinfo`
- CWD detection: `proc_pidinfo` with `PROC_PIDVNODEPATHINFO`

### Process Termination

Send `SIGTERM` (graceful) via `kill(pid, SIGTERM)`. Always behind a confirmation prompt. Never sends `SIGKILL` in v1.

## Dependencies

| Crate       | Purpose                          |
|-------------|----------------------------------|
| ratatui     | TUI framework                    |
| crossterm   | Terminal backend for ratatui     |
| clap        | CLI argument parsing (derive)    |
| serde       | Config/JSON serialization        |
| toml        | Config file parsing              |
| serde_json  | JSON output for list/snapshot    |
| nix         | Unix signal sending (SIGTERM)    |
| dirs        | XDG config directory resolution  |

## Out of Scope for v1

- Windows support
- UDP port scanning (focus on TCP LISTEN)
- Remote/networked port scanning
- Daemon mode
- Port forwarding/remapping
- Conflict auto-resolution
