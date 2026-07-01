# whoseportisitanyway

Cross-platform TUI for discovering which ports are in use, who owns them, and what they're for.

![MIT License](https://img.shields.io/crates/l/whoseportisitanyway)
![crates.io](https://img.shields.io/crates/v/whoseportisitanyway)

You know the drill: something's already bound to `3000`, and you have no idea if it's yesterday's `next dev` you forgot to kill, a Docker container, or someone else's process. `whoseportisitanyway` scans every listening (and optionally established) socket, matches each one back to a process, a project directory, and — when it can tell — a framework or container, and gets out of your way.

## Install

```bash
cargo install whoseportisitanyway
```

Or download a prebuilt binary from the [releases page](https://github.com/z19r/whoseportisitanyway/releases).

Both `whoseportisitanyway` and the shorter `whose-port` are installed as aliases for the same binary — use whichever you'd rather type.

## Usage

Launch the interactive TUI:

```bash
whoseportisitanyway
```

```
┌ whoseportisitanyway ──────────────────────────────────────────────────────┐
│ PORT   PROC          TYPE        PROJECT               OWNERSHIP          │
│ 3000   node          DevServer   marketing-site (Vite)  Owned            │
│ 5432   postgres      Database    —                      Untracked        │
│ 6379   docker-proxy  Docker      redis (compose: api)   Untracked        │
│ 8080   java          BuildTool   legacy-service          Blocked         │
│ 631    cupsd         System      —                       —              │
│                                                                            │
│ j/k move · enter/d detail · x kill · s sort · f filter · h hide-system    │
└────────────────────────────────────────────────────────────────────────────┘
```

### Subcommands

```bash
whoseportisitanyway snapshot          # one-shot port table
whoseportisitanyway snapshot --json   # JSON output
whoseportisitanyway why 3000          # explain what's using port 3000
whoseportisitanyway list              # list all ports (JSON)
whoseportisitanyway list --plain      # tab-separated plain text
```

`why` is the fastest way to answer "what is this thing on my port":

```
$ whoseportisitanyway why 631
Port 631 is used by cupsd (PID 0)
  Type: System
  Command:
  Address: ::1:631
```

### TUI Features

- Group ports by process, framework, or category
- Hide system/ephemeral ports to focus on dev services
- Search and filter by port, process, or description
- Kill processes directly from the TUI

### Keybindings

| Key          | Action                                       |
|--------------|-----------------------------------------------|
| `j` / `↓`    | Move selection down                           |
| `k` / `↑`    | Move selection up                             |
| `g` / `G`    | Jump to first / last row                      |
| `Enter`, `d` | Open detail view for the selected port        |
| `x`          | Prompt to kill the selected process            |
| `y` / `n`    | Confirm / cancel kill                          |
| `o`          | Search the process online (from detail view)   |
| `s`          | Cycle sort order                               |
| `f`          | Cycle filter                                   |
| `h`          | Toggle hiding system/ephemeral ports           |
| `Tab`        | Cycle grouping (process / framework / category)|
| `r`          | Force refresh                                  |
| `q`, `Esc`   | Quit / back                                    |

## Configuration

`whoseportisitanyway` reads an optional TOML config file from your platform's config directory (e.g. `~/.config/whoseportisitanyway/config.toml` on Linux, `~/Library/Application Support/whoseportisitanyway/config.toml` on macOS). Every key is optional and falls back to a built-in default:

```toml
refresh_interval_secs = 2
default_sort = "port"
show_established = false
watched_ports = [3000, 5432, 6379]

project_markers = [
  "package.json", "Cargo.toml", "go.mod", "Gemfile",
  "requirements.txt", "pyproject.toml",
]
```

- `watched_ports` marks ports you explicitly care about (e.g. your own app's ports). Anything in the list is classified `Owned` while it's up and `Blocked` if something else has taken it; ports outside the list are `Untracked`.
- `project_markers` controls which files are used to walk up from a process's working directory to find its project root (used to label entries like `marketing-site (Vite)`).

See [`docs/architecture.md`](docs/architecture.md) for how these settings are consumed internally.

## Supported Platforms

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)

Linux and macOS have separate scanner backends behind a shared `Scanner` trait — see [Architecture → Scanning](docs/architecture.md#scanning).

## Documentation

- [Architecture](docs/architecture.md) — how scanning, classification, and Docker enrichment fit together
- [Building from source](docs/building-from-source.md) — toolchain setup, build/test/lint/release commands
- [Contributing](CONTRIBUTING.md) — workflow, coding conventions, and how to submit changes

## License

MIT
