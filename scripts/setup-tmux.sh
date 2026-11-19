#!/usr/bin/env bash
# Build/install tmux-spaces and link this repo's agent-multiplexer tmux config.
#
# Mirrors scripts/setup-zellij.sh: the tmux-spaces plugin and Go CLI live in
# the model-router monorepo (~/dev/model-router/plugins/tmux-spaces) and the
# zj/harness hook bridge lives in ~/dev/model-router; only the tmux config
# lives here.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${HOME}/.local/bin"
SPACES_REPO="${TMUX_SPACES_REPO:-$HOME/dev/model-router/plugins/tmux-spaces}"
HARNESS_BIN="${HARNESS_BIN:-$DEST/harness}"

die() { printf 'setup-tmux: %s\n' "$*" >&2; exit 1; }

command -v tmux >/dev/null 2>&1 || die "tmux not found on PATH"

mkdir -p "$DEST" "$HOME/.config/tmux"

if [[ ! -x "$SPACES_REPO/tmux-spaces" ]]; then
    [[ -x "$SPACES_REPO/build.sh" ]] || die "missing $SPACES_REPO/build.sh"
    printf 'Building tmux-spaces in %s...\n' "$SPACES_REPO"
    (cd "$SPACES_REPO" && PATH="$HOME/sdk/go/bin:$PATH" ./build.sh)
fi

[[ -x "$HARNESS_BIN" ]] ||
    printf 'setup-tmux: warning: harness CLI not found at %s (agent hooks/harnesses need it)\n' "$HARNESS_BIN" >&2

# Preserve real user files before replacing them with symlinks.
backup_dir="$HOME/.local/share/dots-backups/tmux-$(date +%Y%m%d%H%M%S)"
for path in "$HOME/.config/tmux/tmux.conf" "$DEST/tmux-spaces"; do
    if [[ -e "$path" && ! -L "$path" ]]; then
        mkdir -p "$backup_dir"
        mv "$path" "$backup_dir/"
        printf 'Backed up %s to %s\n' "$path" "$backup_dir"
    fi
done

ln -sfn "$ROOT/tmux/tmux.conf" "$HOME/.config/tmux/tmux.conf"
ln -sfn "$SPACES_REPO/tmux-spaces" "$DEST/tmux-spaces"

printf 'tmux version: %s\n' "$(tmux -V)"
printf 'Linked %s -> %s\n' "$HOME/.config/tmux/tmux.conf" "$ROOT/tmux/tmux.conf"
printf 'Linked %s -> %s\n' "$DEST/tmux-spaces" "$SPACES_REPO/tmux-spaces"
