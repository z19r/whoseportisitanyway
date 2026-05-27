# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2025-05-27

### Added
- TUI interface for viewing active ports with real-time scanning
- Port classification with watched port configuration
- Parent process and framework detection
- `snapshot` command for point-in-time port dumps (text and JSON)
- `why` command to investigate specific ports
- `list` command for JSON/plain port listing
- Cross-platform scanner support (Linux via /proc/net, macOS via lsof)
- Configurable watched ports via `~/.config/whoseportisitanyway/config.toml`
- CI and release workflows
- Install script for curl-pipe-bash installation
