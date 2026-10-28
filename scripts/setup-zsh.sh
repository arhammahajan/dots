#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
zsh_bin="$(command -v zsh || true)"
if [[ -z "$zsh_bin" ]]; then
  printf '%s\n' 'Install Zsh first (Fedora: sudo dnf install zsh), then rerun this script.' >&2
  exit 1
fi
command -v git >/dev/null

clone_if_missing() {
  local url="$1" destination="$2"
  if [[ ! -e "$destination" ]]; then
    git clone --depth 1 "$url" "$destination"
  elif [[ ! -d "$destination/.git" ]]; then
    printf 'Refusing to overwrite an existing non-Git directory: %s\n' "$destination" >&2
    exit 1
  fi
}

clone_if_missing https://github.com/ohmyzsh/ohmyzsh.git "$HOME/.oh-my-zsh"
for plugin in zsh-autosuggestions zsh-syntax-highlighting zsh-completions; do
  clone_if_missing "https://github.com/zsh-users/$plugin.git" "$HOME/.oh-my-zsh/custom/plugins/$plugin"
done

backup_dir="${XDG_DATA_HOME:-$HOME/.local/share}/dots-backups/zsh-$(date +%Y%m%d-%H%M%S)-$$"
mkdir -p "$backup_dir"
chmod 700 "$backup_dir"
# Keep recovery copies of the old Bash setup without changing it.
for config in .bashrc .bash_profile .profile; do
  if [[ -e "$HOME/$config" || -L "$HOME/$config" ]]; then
    cp -a "$HOME/$config" "$backup_dir/$config"
  fi
done

for config in .zshenv .zshrc; do
  "$zsh_bin" -n "$repo_dir/$config"
  if [[ -L "$HOME/$config" && "$(readlink "$HOME/$config")" == "$repo_dir/$config" ]]; then
    continue
  fi
  if [[ -e "$HOME/$config" || -L "$HOME/$config" ]]; then
    mv "$HOME/$config" "$backup_dir/$config"
  fi
  ln -s "$repo_dir/$config" "$HOME/$config"
done
printf 'Zsh configs linked to %s\nBackups: %s\n' "$repo_dir" "$backup_dir"

# Check startup before making this the account's login shell.
"$zsh_bin" -lic '(( $+functions[compdef] && $+functions[_zsh_autosuggest_start] && $+functions[_zsh_highlight] ))'
if ! grep -Fxq "$zsh_bin" /etc/shells; then
  printf '%s must be added to /etc/shells by an administrator before changing your login shell.\n' "$zsh_bin" >&2
  exit 1
fi
current_shell="${SHELL:-}"
if command -v getent >/dev/null; then
  current_shell="$(getent passwd "$(id -un)" | cut -d: -f7)"
fi
if [[ "$current_shell" != "$zsh_bin" ]]; then
  chsh -s "$zsh_bin"
fi
printf '%s\n' 'Setup complete. Log out and back in to use Zsh everywhere.'
