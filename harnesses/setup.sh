#!/bin/sh
# Wire this checkout's harness configs, launcher, and skills into a machine.
# Configs are linked except fx settings, which require a regular file.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
home=${HOME:?HOME is not set}
config_home=${XDG_CONFIG_HOME:-$home/.config}
data_home=${XDG_DATA_HOME:-$home/.local/share}
copilot_home=${COPILOT_HOME:-$home/.copilot}
claude_home=${CLAUDE_CONFIG_DIR:-$home/.claude}
codex_home=${CODEX_HOME:-$home/.codex}
grok_home=$home/.grok
shared=$root/shared/skills
backup_dir=$data_home/dots-backups/harnesses-$(date +%Y%m%d-%H%M%S)-$$

check=false
case ${1:-} in
    "") ;;
    --check) check=true ;;
    *)
        printf 'usage: %s [--check]\n' "$0" >&2
        exit 2
        ;;
esac

created=0
backed_up=0
errors=0

backup_path() {
    path=$1
    relative=${path#"$home"/}
    mkdir -p "$(dirname "$backup_dir/$relative")"
    mv "$path" "$backup_dir/$relative"
    backed_up=$((backed_up + 1))
}

link() {
    source=$1
    destination=$2
    if [ "$check" = true ]; then
        if [ ! -L "$destination" ]; then
            printf 'not a symlink: %s (expected %s)\n' "$destination" "$source" >&2
            errors=$((errors + 1))
        elif [ "$(readlink "$destination")" != "$source" ]; then
            printf 'wrong target: %s -> %s (expected %s)\n' \
                "$destination" "$(readlink "$destination")" "$source" >&2
            errors=$((errors + 1))
        fi
        return
    fi
    if [ -L "$destination" ] && [ "$(readlink "$destination")" = "$source" ]; then
        return
    fi
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        backup_path "$destination"
    fi
    mkdir -p "$(dirname "$destination")"
    ln -s "$source" "$destination"
    created=$((created + 1))
}

# fx rejects symlinked durable settings. Preserve existing local preferences.
copy_fx_settings() {
    source=$root/fx/settings.json
    destination=$home/.fx/settings.json
    if [ -f "$destination" ] && [ ! -L "$destination" ]; then
        return
    fi
    if [ "$check" = true ]; then
        printf 'not a regular settings file: %s\n' "$destination" >&2
        errors=$((errors + 1))
        return
    fi
    mkdir -p "$(dirname "$destination")"
    temporary=$(mktemp "$destination.XXXXXX")
    if [ -f "$destination" ]; then
        cp "$destination" "$temporary"
    else
        cp "$source" "$temporary"
    fi
    chmod 600 "$temporary"
    if [ -e "$destination" ] || [ -L "$destination" ]; then
        backup_path "$destination"
    fi
    mv "$temporary" "$destination"
    created=$((created + 1))
}

# Muse owns its skills directory as a store, so link entries rather than the
# directory itself. Other harnesses can take a whole-directory link.
link_muse_skills() {
    destination=$1
    if [ "$check" = true ]; then
        for source in "$shared"/*; do
            [ -d "$source" ] || continue
            entry=$destination/$(basename "$source")
            if [ ! -L "$entry" ] || [ "$(readlink "$entry")" != "$source" ]; then
                printf 'missing or wrong Muse link: %s (expected %s)\n' \
                    "$entry" "$source" >&2
                errors=$((errors + 1))
            fi
        done
        return
    fi
    if [ -L "$destination" ]; then
        backup_path "$destination"
    elif [ -e "$destination" ] && [ ! -d "$destination" ]; then
        backup_path "$destination"
    fi
    mkdir -p "$destination"
    for source in "$shared"/*; do
        [ -d "$source" ] || continue
        entry=$destination/$(basename "$source")
        if [ -L "$entry" ] && [ "$(readlink "$entry")" = "$source" ]; then
            continue
        fi
        if [ -e "$entry" ] || [ -L "$entry" ]; then
            backup_path "$entry"
        fi
        ln -s "$source" "$entry"
        created=$((created + 1))
    done
    for entry in "$destination"/*; do
        [ -L "$entry" ] || continue
        case $(readlink "$entry") in
            "$shared"/*) [ -e "$entry" ] || rm -f "$entry" ;;
        esac
    done
}

# Launcher, router client settings, and user service.
link "$root/harness" "$home/.local/bin/harness"
link "$root/shared/model-router.env" "$config_home/ai/model-router.env"
link "$root/shared/model-router.service" \
    "$config_home/systemd/user/model-router.service"

# Harness configurations.
link "$root/claude/settings.json" "$claude_home/settings.json"
link "$root/copilot/settings.json" "$copilot_home/settings.json"
link "$root/copilot/mcp-config.json" "$copilot_home/mcp-config.json"
link "$root/grok/config.toml" "$grok_home/config.toml"
link "$root/grok/router-config.toml" \
    "$home/.grok-t3/model-router/config.toml"
link "$root/opencode/opencode.json" "$config_home/opencode/opencode.json"
link "$root/opencode/opencode.jsonc" "$config_home/opencode/opencode.jsonc"
link "$root/opencode/cli.json" "$config_home/opencode/cli.json"
link "$root/pi/settings.json" "$home/.pi/agent/settings.json"
link "$root/pi/mcp.json" "$home/.pi/agent/mcp.json"
link "$root/qwen/settings.json" "$home/.qwen/settings.json"
link "$root/gemini/settings.json" "$home/.gemini/settings.json"
# Source builds supersede the old in-memory ACP loader. Remove only our link.
legacy_gemini_runtime=$config_home/ai/gemini/router-acp.mjs
if [ -L "$legacy_gemini_runtime" ] && \
    [ "$(readlink "$legacy_gemini_runtime")" = "$root/gemini/router-acp.mjs" ]; then
    if [ "$check" = true ]; then
        printf 'obsolete Gemini runtime link: %s\n' "$legacy_gemini_runtime" >&2
        errors=$((errors + 1))
    else
        rm "$legacy_gemini_runtime"
    fi
fi
link "$root/kimi-code/config.toml" "$home/.kimi-code/config.toml"
link "$root/kimi-code/mcp.json" "$home/.kimi-code/mcp.json"
link "$root/kimi-cli/config.toml" "$home/.kimi/config.toml"
link "$root/kimi-cli/mcp.json" "$home/.kimi/mcp.json"
link "$root/mcode/config.yaml" "$home/.minimax/config.yaml"
link "$root/mcode/mcp.json" "$home/.minimax/mcp.json"
link "$root/antigravity/cli/settings.json" \
    "$home/.gemini/antigravity-cli/settings.json"
link "$root/antigravity/mcp_config.json" \
    "$home/.gemini/config/mcp_config.json"
link "$root/antigravity/acp/settings.json" \
    "$home/.gemini/antigravity-acp/settings.json"
link "$root/antigravity/acp/router-settings.json" \
    "$config_home/ai/antigravity-router-acp/antigravity-acp/settings.json"
link "$root/muse/settings.json" "$config_home/muse/settings.json"
copy_fx_settings
link "$root/fx/mcp.json" "$home/.fx/mcp.json"
link "$root/kilo/kilo.json" "$config_home/kilo/kilo.json"
link "$root/vibe/config.toml" "$home/.vibe/config.toml"
link "$root/cline/cline_mcp_settings.json" \
    "$home/.cline/data/settings/cline_mcp_settings.json"

# T3 Code user settings. Window geometry stays in the machine-local
# desktop-settings.json.
link "$root/t3code/settings.json" "$home/.t3/userdata/settings.json"
link "$root/t3code/client-settings.json" "$home/.t3/userdata/client-settings.json"
link "$root/t3code/keybindings.json" "$home/.t3/userdata/keybindings.json"

# Fixed-directory skill paths.
link "$shared" "$claude_home/skills"
link "$shared" "$home/.cline/skills"
link "$shared" "$home/.minimax/skills"
link "$shared" "$home/.gemini/config/skills"
link "$shared" "$home/.gemini/skills"
link "$shared" "$home/.fx/skills"
link "$shared" "$home/.kimi-code/skills"
link "$shared" "$home/.kimi/skills"
link "$shared" "$codex_home/skills/shared"

link_muse_skills "$config_home/muse/skills"

# Optional Pi router extension from the model-router checkout.
pi_extension=$home/dev/model-router/integrations/pi/model-router.ts
if [ -f "$pi_extension" ]; then
    link "$pi_extension" "$home/.pi/agent/extensions/model-router.ts"
fi

if [ "$check" = true ]; then
    exit "$errors"
fi

printf 'Wired harness configuration from %s\n' "$root"
if [ "$created" -eq 0 ] && [ "$backed_up" -eq 0 ]; then
    printf 'Everything was already linked.\n'
else
    printf 'Created %d symlink(s)' "$created"
    if [ "$backed_up" -gt 0 ]; then
        printf '; moved %d existing path(s) to %s' "$backed_up" "$backup_dir"
    fi
    printf '.\n'
fi
printf '\nNext steps:\n'
printf '  systemctl --user daemon-reload\n'
printf '  systemctl --user enable --now model-router\n'
printf '  merge claude/mcp.json and codex/config.toml MCP entries as described in docs/harnesses/browser-mcp.md\n'
printf '  run harnesses/deepseek/setup.sh separately if the DeepSeek Harness is installed\n'
