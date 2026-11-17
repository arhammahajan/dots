#!/usr/bin/env bash
# Build the Spaces navigator plugin and refresh the tracked WASM artifact.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/zellij/plugins/spacesy"

if ! rustup target list --installed 2>/dev/null | grep -qx wasm32-wasip1; then
    printf 'build-zellij-spaces: installing the wasm32-wasip1 Rust target\n'
    rustup target add wasm32-wasip1
fi

cargo test --no-default-features
cargo build --release --target wasm32-wasip1
install -m 644 target/wasm32-wasip1/release/zellij-spaces.wasm \
    "$ROOT/zellij/plugins/zellij-spaces.wasm"
printf 'Built %s/zellij/plugins/zellij-spaces.wasm\n' "$ROOT"
