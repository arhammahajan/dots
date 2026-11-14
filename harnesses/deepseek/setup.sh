#!/bin/sh
# Install the published harness and managed router profiles for this user.
set -eu

setup_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_prefix="$HOME/.local/share/deepseek-harness"
PATH="${XDG_DATA_HOME:-$HOME/.local/share}/fnm/aliases/default/bin:$PATH"
export PATH

node -e 'const [major, minor] = process.versions.node.split(".").map(Number); if (major < 24 || (major === 24 && minor < 9)) { console.error("DeepSeek Harness requires Node >=24.9.0"); process.exit(1); }'
npm install --global --prefix "$install_prefix" \
    --allow-scripts=@deepseek-ai/dsh-subprocess-local,koffi,node-pty,protobufjs \
    @deepseek-ai/dsh@0.2.1-rc.5

# The shared setup links the saved profiles and TUI preferences; never reset
# those settings to a copied template during reinstall.
"$setup_dir/../setup.sh"

mkdir -p "$HOME/.local/bin"
launcher="$(dirname "$setup_dir")/harness"
for name in harness dsh; do
    destination="$HOME/.local/bin/$name"
    executable=$launcher
    if [ "$name" = dsh ]; then executable="$install_prefix/bin/dsh"; fi
    if [ ! -L "$destination" ] || [ "$(readlink "$destination")" != "$executable" ]; then
        if [ -e "$destination" ] || [ -L "$destination" ]; then
            printf 'Refusing to replace an existing executable: %s\n' "$destination" >&2
            exit 1
        fi
        ln -s "$executable" "$destination"
    fi
done

printf 'Installed DeepSeek Harness. Run harness deepseek for the Web UI, or harness deepseek headless "your task".\n'
