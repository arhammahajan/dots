# Keep user-installed tools available in interactive and noninteractive Zsh.
export FNM_PATH="${XDG_DATA_HOME:-$HOME/.local/share}/fnm"
export PNPM_HOME="${XDG_DATA_HOME:-$HOME/.local/share}/pnpm"

typeset -U path PATH
path=(
  "$HOME/.local/bin"
  "$HOME/bin"
  "$HOME/.opencode/bin"
  "$FNM_PATH"
  "$PNPM_HOME/bin"
  "$PNPM_HOME"
  $path
)

# Use the active CUDA toolkit selected by the system alternatives.
if [[ -d /usr/local/cuda/bin ]]; then
  path=(/usr/local/cuda/bin $path)
fi
export PATH

# Harnesses use client settings; only the router reads upstream credentials.
if [[ -r "${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env" ]]; then
  . "${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env"
fi
