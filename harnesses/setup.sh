#!/bin/sh
# Wire this checkout's harness configs and skills into a machine using GNU Stow.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
repo=$(CDPATH= cd -- "$root/.." && pwd -P)

case ${1:-} in
    "")
        "$repo/stow.sh" harnesses
        printf '\nNext steps:\n'
        printf '  systemctl --user daemon-reload\n'
        printf '  systemctl --user enable --now model-router\n'
        printf '  systemctl --user enable --now spaces-hooks.service  # optional: keep the Spaces hook listener running\n'
        printf '  merge claude/mcp.json and codex/config.toml MCP entries as described in docs/harnesses/browser-mcp.md\n'
        printf '  run harnesses/deepseek/setup.sh separately if the DeepSeek Harness is installed\n'
        printf '  install the harness CLI from ~/dev/model-router (go build ./cmd/harness) and symlink it to ~/.local/bin/harness\n'
        ;;
    --check)
        exec "$repo/stow.sh" --check harnesses
        ;;
    *)
        printf 'usage: %s [--check]\n' "$0" >&2
        exit 2
        ;;
esac
