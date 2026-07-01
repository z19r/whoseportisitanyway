# Contributing

Thanks for considering a contribution to `whoseportisitanyway`.

## Getting set up

See [Building from source](docs/building-from-source.md) for toolchain setup and the full list of `just` commands. The short version:

```bash
git clone https://github.com/z19r/whoseportisitanyway.git
cd whoseportisitanyway
just init    # wire up local git hooks (fmt + clippy on commit)
just test
```

## Workflow

1. **Open an issue first for anything non-trivial** — new subcommands, TUI behavior changes, new framework/classification signatures, or scanner changes. Small fixes and docs typos can go straight to a PR.
2. **Branch from `main`.** Name branches descriptively (`fix/...`, `feat/...`, `docs/...`).
3. **Keep changes focused.** One logical change per PR — easier to review, easier to revert if something's wrong.
4. **Run `just release-check` before opening a PR.** This is the same `fmt-check` + `clippy` + `test` gate CI runs (split into separate `fmt`/`clippy`/`check`/`test` jobs in CI — see [`ci.yml`](.github/workflows/ci.yml)) — running it locally first saves a round trip.
5. **Add tests for new behavior.** Classifier and model logic in particular should be covered by unit tests (see existing `#[cfg(test)] mod tests` blocks alongside the code they test, e.g. `src/tui/keybindings.rs`, `src/classifier/*.rs`).
6. **Open the PR against `main`** and describe what changed and why, not just what.

## Code style

- Formatting is enforced by `rustfmt` (`just fmt-check` in CI) — run `just fmt` before committing rather than hand-formatting.
- Lints are enforced by `clippy` with warnings-as-errors (`just clippy`) — fix warnings rather than suppressing them unless there's a specific, commented reason.
- Keep platform-specific code behind the existing `Scanner` trait / `#[cfg(target_os = ...)]` boundaries (see [Architecture → Scanning](docs/architecture.md#scanning)) rather than branching on OS deeper in the call stack.
- Prefer extending the existing `Classification`/`Framework` enums and the classifier pipeline (`src/classifier/`) over adding one-off special cases elsewhere.

## Reporting bugs / requesting features

Open a [GitHub issue](https://github.com/z19r/whoseportisitanyway/issues) with:
- Your OS and architecture (Linux/macOS, x86_64/aarch64)
- Output of `whoseportisitanyway snapshot --json` if relevant (redact anything sensitive)
- Steps to reproduce, and what you expected instead

## Where things live

If you're not sure where a change belongs, read [`docs/architecture.md`](docs/architecture.md) first — it maps the scan → enrich → classify → render pipeline to the modules under `src/`.
