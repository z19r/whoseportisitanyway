# Building from source

## Prerequisites

- Rust (stable toolchain — see [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install))
- [`just`](https://github.com/casey/just) — task runner used for all dev commands (`cargo install just`)

## Clone and build

```bash
git clone https://github.com/z19r/whoseportisitanyway.git
cd whoseportisitanyway
just build            # debug build
just build-release    # optimized build (LTO, stripped)
```

Run it directly without installing:

```bash
just run               # launches the TUI
just run snapshot       # forwards args, e.g. `cargo run -- snapshot`
just why 3000
```

## Common tasks

All commands are defined in the [`justfile`](../justfile); run `just` with no arguments to list them.

| Command             | What it does                                              |
|----------------------|------------------------------------------------------------|
| `just check`         | `cargo check`                                               |
| `just test`          | `cargo test`                                                 |
| `just test-verbose`  | `cargo test -- --nocapture`                                  |
| `just fmt`           | `cargo fmt`                                                   |
| `just fmt-check`     | `cargo fmt -- --check` (used in CI)                           |
| `just clippy`        | `cargo clippy --all-targets --all-features -- -D warnings`   |
| `just lint`          | `clippy` + `fmt-check`                                        |
| `just release-check` | `fmt-check` + `clippy` + `test` — the full local pre-merge gate |
| `just loc`           | Line count of `src/`                                          |
| `just tree`          | List all `src/**/*.rs` files                                  |

## Git hooks

```bash
just init
```

Points `core.hooksPath` at [`.githooks/`](../.githooks), which runs `cargo fmt --check` and `cargo clippy -- -D warnings` on every commit. This mirrors what CI enforces (see below), so failures surface locally before you push.

## Continuous integration

[`ci.yml`](../.github/workflows/ci.yml) runs on every push/PR to `main` as four independent jobs — `fmt`, `clippy`, `check`, and `test` — so a failure in one (e.g. a flaky test) doesn't block feedback from the others, and each shows up as its own status check on a PR.

## Releasing

Releases are cut from `main` via a version bump, not a manual tag push:

```bash
just release patch   # or: minor, major
```

This bumps `Cargo.toml`/`VERSION`, syncs the version string into the marketing site (`site/src/Hero.jsx`, `site/src/Nav.jsx`), opens a `release/vX.Y.Z` branch and PR. Merging that PR to `main` triggers [`release.yml`](../.github/workflows/release.yml), which re-verifies the build (`just release-check`), builds cross-platform binaries, and publishes the GitHub release.

See [Contributing](../CONTRIBUTING.md) for the day-to-day workflow (branching, PRs, code review expectations).
