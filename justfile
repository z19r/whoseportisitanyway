default:
    @just --list

build:
    cargo build

release:
    cargo build --release

check:
    cargo check

test:
    cargo test

test-verbose:
    cargo test -- --nocapture

clippy:
    cargo clippy -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

lint: clippy fmt-check

ci: fmt-check clippy test build

run *ARGS:
    cargo run -- {{ARGS}}

snapshot:
    cargo run -- snapshot

snapshot-json:
    cargo run -- snapshot --json

why PORT:
    cargo run -- why {{PORT}}

list:
    cargo run -- list

list-json:
    cargo run -- list --json

bar: clean build run

tag-release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    just ci
    git tag -a "v{{VERSION}}" -m "Release v{{VERSION}}"
    echo "Tagged v{{VERSION}}. Push with: git push origin v{{VERSION}}"

clean:
    cargo clean

loc:
    @find src -name '*.rs' | xargs wc -l | tail -1

tree:
    @find src -name '*.rs' | sort
