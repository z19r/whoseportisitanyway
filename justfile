default:
    @just --list --unsorted

build:
    cargo build

build-release:
    cargo build --release

check:
    cargo check

test:
    cargo test

test-verbose:
    cargo test -- --nocapture

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

lint: clippy fmt-check

release-check:
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

ci: release-check build

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

release LEVEL:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ ! "{{LEVEL}}" =~ ^(patch|minor|major)$ ]]; then
        echo "Usage: just release patch|minor|major"; exit 1
    fi
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Error: dirty working tree"; exit 1
    fi
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$BRANCH" != "main" ]]; then
        echo "Error: must be on main (currently on $BRANCH)"; exit 1
    fi
    git pull --ff-only origin main
    cargo set-version --bump {{LEVEL}}
    cargo check --quiet
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "$VERSION" > VERSION
    # Sync version into static site
    sed -i "s|// WHOSEPORTISITANYWAY · v[0-9]*\.[0-9]*\.[0-9]*|// WHOSEPORTISITANYWAY · v${VERSION}|" site/src/Hero.jsx
    sed -i "s|</span>v[0-9]*\.[0-9]*\.[0-9]*</span>|</span>v${VERSION}</span>|" site/src/Nav.jsx
    git checkout -b "release/v${VERSION}"
    git add Cargo.toml Cargo.lock VERSION site/src/Hero.jsx site/src/Nav.jsx
    git commit -m "release: v${VERSION}"
    git push -u origin "release/v${VERSION}"
    gh pr create \
        --title "release: v${VERSION}" \
        --body "Bump to v${VERSION} ({{LEVEL}} release)" \
        --base main
    echo "PR created. Merging triggers the verified release workflow."

init:
    git config core.hooksPath .githooks

clean:
    cargo clean

loc:
    @find src -name '*.rs' | xargs wc -l | tail -1

tree:
    @find src -name '*.rs' | sort
