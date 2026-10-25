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

# Export shared AI credentials for tools such as OpenCode.
if [[ -r "$HOME/.config/ai/credentials.env" ]]; then
  set -a
  . "$HOME/.config/ai/credentials.env"
  set +a
fi

# Muse Code uses META_API_KEY with its configured OpenCode Go endpoint.
if [[ -n "${OPENCODE_API_KEY:-}" ]]; then
  export META_API_KEY="$OPENCODE_API_KEY"
fi

# OpenCode Go requires a stable routing ID for each Muse conversation.
muse() {
  local muse_headers="${MUSE_CUSTOM_HEADERS:-}"
  local muse_session_id="${MUSE_SESSION_ID:-}"
  if [[ "${(L)muse_headers}" != *x-opencode-session:* ]]; then
    if [[ -z "$muse_session_id" ]]; then
      muse_session_id="$(uuidgen)" || return
    fi
    [[ -z "$muse_headers" ]] || muse_headers+=$'\n'
    muse_headers+="x-opencode-session: $muse_session_id"
  fi
  MUSE_CUSTOM_HEADERS="$muse_headers" command muse "$@"
}
