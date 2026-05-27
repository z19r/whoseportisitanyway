# Whetstone (Claude Code stack)

This project was set up with Whetstone: Headroom, RTK, and ICM for
token-efficient Claude Code sessions.

## Quick Start

```bash
whetstone              # Start Claude with Headroom proxy
whetstone claude       # Same as above
```

## Tools

| Tool | Purpose | Savings |
|------|---------|---------|
| Headroom | HTTP proxy compresses context before API | 50-90% |
| RTK | Hook rewrites CLI output before entering context | 60-90% |
| ICM | Embedded SQLite memory, zero dependencies | persistent context |

## Configuration

| File | Purpose |
|------|---------|
| `~/.claude/settings.json` | Hook registrations (global) |
| `.claude/config.local.json` | Project config |

## Uninstall

Per-project: `whetstone uninstall`
