#!/usr/bin/env bash
set -euo pipefail

REPO="z19r/whoseportisitanyway"
BIN_DIR="${HOME}/.local/bin"
BIN_NAME="whoseportisitanyway"

fail() {
    echo "Error: $1" >&2
    exit 1
}

os="$(uname -s)"
arch="$(uname -m)"

case "${os}" in
    Linux)
        case "${arch}" in
            x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
            *)       fail "unsupported Linux architecture: ${arch}" ;;
        esac
        ;;
    Darwin)
        case "${arch}" in
            x86_64)  TARGET="x86_64-apple-darwin" ;;
            arm64)   TARGET="aarch64-apple-darwin" ;;
            *)       fail "unsupported macOS architecture: ${arch}" ;;
        esac
        ;;
    *)
        fail "unsupported OS: ${os}"
        ;;
esac

ARCHIVE="${BIN_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"

echo "Downloading ${BIN_NAME} for ${TARGET}..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL -o "${TMPDIR}/${ARCHIVE}" "$DOWNLOAD_URL" || fail "download failed — check ${DOWNLOAD_URL}"
tar xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

mkdir -p "$BIN_DIR"
mv "${TMPDIR}/${BIN_NAME}" "${BIN_DIR}/${BIN_NAME}"
chmod +x "${BIN_DIR}/${BIN_NAME}"

echo "Installed ${BIN_NAME} to ${BIN_DIR}/${BIN_NAME}"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$BIN_DIR"; then
    echo ""
    echo "Warning: ${BIN_DIR} is not in your PATH."
    echo "Add it with:  export PATH=\"${BIN_DIR}:\$PATH\""
fi

echo ""
echo "Run '${BIN_NAME}' to get started."
