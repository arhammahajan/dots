#!/bin/sh
# Install the published harness and managed router profiles for this user.
set -eu

setup_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_prefix="$HOME/.local/share/deepseek-harness"
dsh_home="${DSH_HOME:-$HOME/.dsh}"
backup_dir="$HOME/.local/share/dots-backups/deepseek-$(date +%Y%m%d-%H%M%S)"
PATH="${XDG_DATA_HOME:-$HOME/.local/share}/fnm/aliases/default/bin:$PATH"
export PATH

node -e 'const [major, minor] = process.versions.node.split(".").map(Number); if (major < 24 || (major === 24 && minor < 9)) { console.error("DeepSeek Harness requires Node >=24.9.0"); process.exit(1); }'
npm install --global --prefix "$install_prefix" \
    --allow-scripts=@deepseek-ai/dsh-subprocess-local,koffi,node-pty,protobufjs \
    @deepseek-ai/dsh@0.2.1-rc.5

# Initialize shipped profiles without starting an application or sending a request.
acp_template=$(mktemp)
trap 'rm -f "$acp_template"' EXIT HUP INT TERM
cat "$setup_dir/cordis.patch.yml" "$setup_dir/acp.patch.yml" >"$acp_template"
for profile in web headless acp sdk sdk-minimal; do
    "$install_prefix/bin/dsh" "$profile" --dump-default-config >/dev/null
    profile_patch="$dsh_home/profiles/$profile/cordis.patch.yml"
    profile_template="$setup_dir/cordis.patch.yml"
    if [ "$profile" = acp ]; then
        profile_template="$acp_template"
    fi
    if [ -e "$profile_patch" ] && ! cmp -s "$profile_template" "$profile_patch"; then
        mkdir -p "$backup_dir/$profile"
        cp -p "$profile_patch" "$backup_dir/$profile/cordis.patch.yml"
    fi
    cp "$profile_template" "$profile_patch"
done

mkdir -p "$HOME/.local/bin"
destination="$HOME/.local/bin/harness"
launcher="$(dirname "$setup_dir")/harness"
if [ ! -L "$destination" ] || [ "$(readlink "$destination")" != "$launcher" ]; then
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        mkdir -p "$backup_dir/bin"
        mv "$destination" "$backup_dir/bin/harness"
    fi
    ln -s "$launcher" "$destination"
fi

printf 'Installed DeepSeek Harness. Run harness deepseek for the Web UI, or harness deepseek headless "your task".\n'
