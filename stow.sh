#!/usr/bin/env bash
# Manage dotfile and harness symlinks using GNU Stow.
set -euo pipefail

repo="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
stow_dir="$repo/stow"
target="${HOME:?HOME is not set}"
mode="stow"
declare -a req=()

usage() {
    cat <<'EOF'
Usage: ./stow.sh [options] [group|package ...]

Options:
  --check, -n        Verify Stow symlinks without modifying files
  --delete, -D       Unstow the selected packages
  --target, -t DIR   Override target directory (default: $HOME)
  --list             List available package groups and packages
  -h, --help         Show this help

Groups:
  all        shell + editors + desktop + harnesses (default)
  shell      shell tmux zellij kitty yazi
  editors    nvim zed vscodium
  desktop    niri fuzzel quickshell waypaper wallflow neofetch
  x11        polybar rofi redshift zathura flameshot
  extras     ranger lazygit
  harnesses  router + all coding CLI harnesses (with OS-specific codex)

Direct GNU Stow also works from the repo root via .stowrc:
  cd ~/dev/dots && stow <package>
EOF
}

codex_pkg() {
    if [ "$(uname -s)" = "Darwin" ]; then
        printf '%s' "codex-macos"
    else
        printf '%s' "codex"
    fi
}

expand_group() {
    case "$1" in
        all)
            expand_group shell
            expand_group editors
            expand_group desktop
            expand_group harnesses
            ;;
        shell)
            printf '%s\n' shell tmux zellij kitty yazi
            ;;
        editors)
            printf '%s\n' nvim zed vscodium
            ;;
        desktop)
            printf '%s\n' niri fuzzel quickshell waypaper wallflow neofetch
            ;;
        x11)
            printf '%s\n' polybar rofi redshift zathura flameshot
            ;;
        extras)
            printf '%s\n' ranger lazygit
            ;;
        harnesses)
            printf '%s\n' \
                router antigravity claude cline "$(codex_pkg)" copilot cursor \
                deepseek fx gemini grok kilo kimi-cli kimi-code mcode muse \
                opencode pi qwen t3code vibe
            ;;
        *)
            printf '%s\n' "$1"
            ;;
    esac
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --check|-n)
            mode="check"
            shift
            ;;
        --delete|-D)
            mode="delete"
            shift
            ;;
        --target|-t)
            [ "$#" -ge 2 ] || { printf 'stow.sh: --target requires a directory\n' >&2; exit 2; }
            target="$2"
            shift 2
            ;;
        --list)
            printf 'Groups: all shell editors desktop x11 extras harnesses\n'
            printf 'Packages:\n'
            (cd "$stow_dir" && ls -1) | sed 's/^/  /'
            exit 0
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            while [ "$#" -gt 0 ]; do
                req+=("$1")
                shift
            done
            ;;
        -*)
            printf 'stow.sh: unknown option: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
        *)
            req+=("$1")
            shift
            ;;
    esac
done

command -v stow >/dev/null 2>&1 || {
    printf 'stow.sh: GNU stow is not installed\n' >&2
    exit 1
}

if [ "${#req[@]}" -eq 0 ]; then
    req=(all)
fi

declare -a pkgs=()
declare -A seen=()
for item in "${req[@]}"; do
    while IFS= read -r pkg; do
        [ -n "$pkg" ] || continue
        if [ ! -d "$stow_dir/$pkg" ]; then
            printf 'stow.sh: unknown package: %s\n' "$pkg" >&2
            exit 2
        fi
        if [ -z "${seen[$pkg]+x}" ]; then
            seen["$pkg"]=1
            pkgs+=("$pkg")
        fi
    done < <(expand_group "$item")
done

target="$(cd "$target" && pwd -P)"
data_home="${XDG_DATA_HOME:-$target/.local/share}"
case "$data_home" in
    "$target"/*) ;;
    *) data_home="$target/.local/share" ;;
esac
backup_dir="$data_home/dots-backups/stow-$(date +%Y%m%d-%H%M%S)-$$"
backed_up=0

backup_path() {
    local path="$1"
    local rel="${path#"$target"/}"
    mkdir -p "$(dirname "$backup_dir/$rel")"
    mv "$path" "$backup_dir/$rel"
    backed_up=$((backed_up + 1))
}

# Migrate pre-existing manual symlinks into Stow ownership and back up any
# conflicting real files/directories before running stow.
prepare_pkg_targets() {
    local pkg="$1"
    local pkg_root="$stow_dir/$pkg"
    local entry rel dest dest_real src_real
    while IFS= read -r -d '' entry; do
        rel="${entry#"$pkg_root"/}"
        dest="$target/$rel"
        if [ ! -e "$dest" ] && [ ! -L "$dest" ]; then
            continue
        fi
        if [ -L "$dest" ]; then
            dest_real="$(readlink -f "$dest" 2>/dev/null || true)"
            src_real="$(readlink -f "$entry" 2>/dev/null || true)"
            # Already a Stow link pointing into stow/<pkg>/.
            case "$(readlink "$dest")" in
                *"/stow/$pkg/$rel"|*"stow/$pkg/$rel") continue ;;
            esac
            # Old direct symlink into this dots repo (e.g. from legacy setup.sh):
            # replace cleanly so Stow owns the link.
            if [ -n "$dest_real" ] && [ "$dest_real" = "$src_real" ]; then
                rm "$dest"
                continue
            fi
            backup_path "$dest"
        else
            backup_path "$dest"
        fi
    done < <(find "$pkg_root" \( -type f -o -type l \) -print0)
}

check_pkg_targets() {
    local pkg="$1"
    local pkg_root="$stow_dir/$pkg"
    local entry rel dest dest_real src_real
    local errs=0
    while IFS= read -r -d '' entry; do
        rel="${entry#"$pkg_root"/}"
        dest="$target/$rel"
        if [ ! -L "$dest" ]; then
            printf 'not a symlink: %s (expected from stow/%s)\n' "$dest" "$pkg" >&2
            errs=$((errs + 1))
            continue
        fi
        dest_real="$(readlink -f "$dest" 2>/dev/null || true)"
        src_real="$(readlink -f "$entry" 2>/dev/null || true)"
        if [ -z "$src_real" ] || [ "$dest_real" != "$src_real" ]; then
            printf 'wrong target: %s -> %s (expected %s)\n' "$dest" "$(readlink "$dest")" "$src_real" >&2
            errs=$((errs + 1))
        fi
    done < <(find "$pkg_root" \( -type f -o -type l \) -print0)
    return "$errs"
}

prepare_muse_skills() {
    local dest_dir="$target/.config/muse/skills"
    local src_root="$repo/harnesses/shared/skills"
    mkdir -p "$dest_dir"
    local src name entry dest_real src_real
    for src in "$src_root"/*; do
        [ -d "$src" ] || continue
        name="$(basename "$src")"
        entry="$dest_dir/$name"
        if [ -L "$entry" ]; then
            case "$(readlink "$entry")" in
                *"harnesses/shared/skills/$name")
                    # Old absolute symlink from setup.sh -> let stow replace with relative link.
                    if [ "${entry:0:1}" = "/" ] && [[ "$(readlink "$entry")" == /* ]]; then
                        rm "$entry"
                    fi
                    continue
                    ;;
            esac
            dest_real="$(readlink -f "$entry" 2>/dev/null || true)"
            src_real="$(readlink -f "$src" 2>/dev/null || true)"
            if [ "$dest_real" = "$src_real" ]; then
                rm "$entry"
            else
                backup_path "$entry"
            fi
        elif [ -e "$entry" ]; then
            backup_path "$entry"
        fi
    done
}

check_muse_skills() {
    local dest_dir="$target/.config/muse/skills"
    local src_root="$repo/harnesses/shared/skills"
    local src name entry dest_real src_real errs=0
    for src in "$src_root"/*; do
        [ -d "$src" ] || continue
        name="$(basename "$src")"
        entry="$dest_dir/$name"
        dest_real="$(readlink -f "$entry" 2>/dev/null || true)"
        src_real="$(readlink -f "$src" 2>/dev/null || true)"
        if [ ! -L "$entry" ] || [ "$dest_real" != "$src_real" ]; then
            printf 'missing or wrong Muse skill link: %s (expected %s)\n' "$entry" "$src_real" >&2
            errs=$((errs + 1))
        fi
    done
    return "$errs"
}

sync_pi_extension() {
    local ext_src="$target/dev/model-router/integrations/pi/model-router.ts"
    local ext_dst="$target/.pi/agent/extensions/model-router.ts"
    [ -f "$ext_src" ] || return 0
    if [ "$mode" = "check" ]; then
        if [ ! -L "$ext_dst" ] || [ "$(readlink -f "$ext_dst" 2>/dev/null || true)" != "$(readlink -f "$ext_src")" ]; then
            printf 'missing or wrong Pi extension link: %s (expected %s)\n' "$ext_dst" "$ext_src" >&2
            return 1
        fi
        return 0
    elif [ "$mode" = "delete" ]; then
        if [ -L "$ext_dst" ] && [ "$(readlink -f "$ext_dst" 2>/dev/null || true)" = "$(readlink -f "$ext_src")" ]; then
            rm "$ext_dst"
        fi
        return 0
    fi
    mkdir -p "$(dirname "$ext_dst")"
    if [ -L "$ext_dst" ] && [ "$(readlink "$ext_dst")" = "$ext_src" ]; then
        return 0
    fi
    if [ -e "$ext_dst" ] || [ -L "$ext_dst" ]; then
        backup_path "$ext_dst"
    fi
    ln -s "$ext_src" "$ext_dst"
}

cleanup_legacy_gemini() {
    local legacy="$target/.config/ai/gemini/router-acp.mjs"
    if [ -L "$legacy" ] && [ "$(readlink "$legacy")" = "$repo/harnesses/gemini/router-acp.mjs" ]; then
        if [ "$mode" = "check" ]; then
            printf 'obsolete Gemini runtime link: %s\n' "$legacy" >&2
            return 1
        fi
        rm "$legacy"
    fi
    return 0
}

has_pkg() {
    [ -n "${seen[$1]+x}" ]
}

case "$mode" in
    check)
        errors=0
        for pkg in "${pkgs[@]}"; do
            if ! check_pkg_targets "$pkg"; then
                errors=$((errors + 1))
            fi
        done
        if ! stow --dir="$stow_dir" --target="$target" --no-folding --restow --simulate "${pkgs[@]}" >/dev/null 2>&1; then
            stow --dir="$stow_dir" --target="$target" --no-folding --restow --simulate "${pkgs[@]}" || true
            errors=$((errors + 1))
        fi
        if has_pkg muse; then
            check_muse_skills || errors=$((errors + 1))
        fi
        if has_pkg pi; then
            sync_pi_extension || errors=$((errors + 1))
        fi
        if has_pkg gemini; then
            cleanup_legacy_gemini || errors=$((errors + 1))
        fi
        if [ "$errors" -gt 0 ]; then
            exit 1
        fi
        printf 'All %d Stow package(s) verified cleanly against %s.\n' "${#pkgs[@]}" "$target"
        ;;
    delete)
        stow --dir="$stow_dir" --target="$target" --no-folding --delete "${pkgs[@]}"
        if has_pkg muse && [ -d "$target/.config/muse/skills" ]; then
            (cd "$repo/harnesses/shared" && stow --dir=. --target="$target/.config/muse/skills" --delete skills)
        fi
        if has_pkg pi; then
            sync_pi_extension
        fi
        printf 'Unstowed %d package(s) from %s.\n' "${#pkgs[@]}" "$target"
        ;;
    stow)
        for pkg in "${pkgs[@]}"; do
            prepare_pkg_targets "$pkg"
        done
        stow --dir="$stow_dir" --target="$target" --no-folding --restow "${pkgs[@]}"
        if has_pkg muse; then
            prepare_muse_skills
            (cd "$repo/harnesses/shared" && stow --dir=. --target="$target/.config/muse/skills" --restow skills)
        fi
        if has_pkg pi; then
            sync_pi_extension
        fi
        if has_pkg gemini; then
            cleanup_legacy_gemini
        fi
        printf 'Stowed %d package(s) from %s into %s' "${#pkgs[@]}" "$stow_dir" "$target"
        if [ "$backed_up" -gt 0 ]; then
            printf ' (backed up %d existing path(s) to %s)' "$backed_up" "$backup_dir"
        fi
        printf '.\n'
        ;;
esac
