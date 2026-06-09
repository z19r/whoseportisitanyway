# whoseportisitanyway

Cross-platform TUI for discovering which ports are in use, who owns them, and what they're for.

![MIT License](https://img.shields.io/crates/l/whoseportisitanyway)
![crates.io](https://img.shields.io/crates/v/whoseportisitanyway)

## Install

```bash
cargo install whoseportisitanyway
```

Or download a prebuilt binary from the [releases page](https://github.com/z19r/whoseportisitanyway/releases).

## Usage

Launch the interactive TUI:

```bash
whoseportisitanyway
```

### Subcommands

```bash
whoseportisitanyway snapshot          # one-shot port table
whoseportisitanyway snapshot --json   # JSON output
whoseportisitanyway why 3000          # explain what's using port 3000
whoseportisitanyway list              # list all ports (JSON)
whoseportisitanyway list --plain      # tab-separated plain text
```

### TUI Features

- Group ports by process, framework, or category
- Hide system/ephemeral ports to focus on dev services
- Search and filter by port, process, or description
- Kill processes directly from the TUI

## Supported Platforms

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)

## License

MIT
