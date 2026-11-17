#!/usr/bin/env bash
# Install the pinned Zellij release and link this repo's agent-multiplexer setup.
#
# Zellij 0.46.0 is the first release with the OSC 7501 inbox and attention
# system this configuration depends on.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="0.46.0"
DEST="${HOME}/.local/bin"

die() { printf 'setup-zellij: %s\n' "$*" >&2; exit 1; }

os="$(uname -s)"
arch="$(uname -m)"
case "${os}-${arch}" in
    Linux-x86_64)   target="x86_64-unknown-linux-musl" ;;
    Linux-aarch64)  target="aarch64-unknown-linux-musl" ;;
    Darwin-arm64)   target="aarch64-apple-darwin" ;;
    Darwin-x86_64)  target="x86_64-apple-darwin" ;;
    *) die "unsupported platform ${os}/${arch}" ;;
esac

checksum() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

base="https://github.com/zellij-org/zellij/releases/download/v${VERSION}"
archive="zellij-${target}.tar.gz"

printf 'Downloading Zellij %s (%s)...\n' "$VERSION" "$target"
curl -fL --retry 3 -o "$tmp/$archive" "$base/$archive"
curl -fL --retry 3 -o "$tmp/$archive.sha256sum" "$base/zellij-${target}.sha256sum"

expected="$(awk 'NR==1 {print $1}' "$tmp/$archive.sha256sum")"
[[ "$expected" =~ ^[0-9a-f]{64}$ ]] || die "could not read the release checksum"
tar xzf "$tmp/$archive" -C "$tmp"
actual="$(checksum "$tmp/zellij")"
[[ "$actual" == "$expected" ]] || die "checksum mismatch: expected $expected, got $actual"

mkdir -p "$DEST" "$HOME/.config/zellij/plugins" "$HOME/.cache/zellij"

spaces_plugin="$HOME/.config/zellij/plugins/zellij-spaces.wasm"
[[ -f "$ROOT/zellij/plugins/zellij-spaces.wasm" ]] ||
    die "missing zellij/plugins/zellij-spaces.wasm (run scripts/build-zellij-spaces.sh)"

# Preserve real user files before replacing them with symlinks.
backup_dir="$HOME/.local/share/dots-backups/zellij-$(date +%Y%m%d%H%M%S)"
for path in "$HOME/.config/zellij/config.kdl" "$DEST/zj" "$spaces_plugin"; do
    if [[ -e "$path" && ! -L "$path" ]]; then
        mkdir -p "$backup_dir"
        mv "$path" "$backup_dir/"
        printf 'Backed up %s to %s\n' "$path" "$backup_dir"
    fi
done

install -m 755 "$tmp/zellij" "$DEST/zellij"
install -m 644 "$ROOT/zellij/plugins/zellij-spaces.wasm" "$spaces_plugin"
# Aliases used by sessions started during the temporary spacesy rename.
ln -sfn "$spaces_plugin" "$HOME/.config/zellij/plugins/zellij-spacesy.wasm"
ln -sfn "$spaces_plugin" "$HOME/.config/zellij/plugins/zellij-spaces-panel.wasm"
ln -sfn "$ROOT/zellij/config.kdl" "$HOME/.config/zellij/config.kdl"
ln -sfn "$ROOT/zellij/zj" "$DEST/zj"

# Zellij caches plugin permission grants by absolute plugin path. Seed the
# grants so the navigator opens without a prompt. Keep any other plugin blocks.
python3 - "$HOME/.cache/zellij/permissions.kdl" "$spaces_plugin" <<'PY'
import re
import sys
from pathlib import Path

cache = Path(sys.argv[1])
plugin = str(Path(sys.argv[2]).expanduser().resolve())
permissions = [
    "ReadApplicationState",
    "ChangeApplicationState",
]

existing = cache.read_text() if cache.exists() else ""
pattern = re.compile(
    r'^"?%s"?\s*\{.*?^\}\s*$' % re.escape(plugin),
    re.MULTILINE | re.DOTALL,
)
existing = pattern.sub("", existing).strip()
block = '"%s" {\n%s\n}\n' % (
    plugin,
    "\n".join("    " + permission for permission in permissions),
)
cache.write_text((existing + "\n" if existing else "") + block)
PY

printf 'Installed %s/zellij (%s)\n' "$DEST" "$("$DEST/zellij" --version)"
printf 'Linked %s and %s\n' "$HOME/.config/zellij/config.kdl" "$DEST/zj"
printf 'Installed %s\n' "$spaces_plugin"
"$DEST/zellij" setup --check
