# Oh My Zsh itself and third-party plugins are installed outside this repo.
export ZSH="$HOME/.oh-my-zsh"
export ZSH_CUSTOM="$ZSH/custom"
ZSH_THEME=""

ZSH_CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/oh-my-zsh"
mkdir -p "$ZSH_CACHE_DIR"
ZSH_COMPDUMP="$ZSH_CACHE_DIR/.zcompdump-${HOST}-${ZSH_VERSION}"
zstyle ':omz:update' mode reminder

# Extra completion definitions must be on fpath before Oh My Zsh runs compinit.
fpath=("$ZSH_CUSTOM/plugins/zsh-completions/src" $fpath)
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'
zstyle ':completion:*' menu select
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"

HISTFILE="$HOME/.zsh_history"
HISTSIZE=50000
SAVEHIST=50000
ZSH_AUTOSUGGEST_STRATEGY=(history completion)

# Preserve the existing Node setup, using fnm's native Zsh integration.
if (( $+commands[fnm] )); then
  eval "$(fnm env --use-on-cd --shell zsh)"
fi

# fnm prepends its Node bin directory. Restore the Bash setup's CLI priority,
# including the standalone pnpm installation ahead of Node's bundled shim.
path=("$HOME/.local/bin" "$HOME/.opencode/bin" "$PNPM_HOME/bin" $path)

# Keep syntax highlighting last so it sees the other plugins' widgets.
plugins=(
  git
  sudo
  extract
  history
  zsh-autosuggestions
  zsh-syntax-highlighting
)

if [[ -r "$ZSH/oh-my-zsh.sh" ]]; then
  source "$ZSH/oh-my-zsh.sh"
else
  print -u2 'Oh My Zsh is missing. Run scripts/setup-zsh.sh from the dots repo.'
fi

setopt HIST_IGNORE_ALL_DUPS HIST_REDUCE_BLANKS HIST_IGNORE_SPACE SHARE_HISTORY
setopt AUTO_CD

bindkey -e
autoload -Uz up-line-or-beginning-search down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search
bindkey '^[[A' up-line-or-beginning-search
bindkey '^[[B' down-line-or-beginning-search
bindkey '^[OA' up-line-or-beginning-search
bindkey '^[OB' down-line-or-beginning-search
bindkey '^[[H' beginning-of-line
bindkey '^[[F' end-of-line
bindkey '^[[3~' delete-char

# OSC 133 semantic prompt markers and OSC 7 cwd reporting. Zellij (0.45+), and
# terminals such as WezTerm, use these to know when a command starts, finishes
# and fails, and to open new panes in the current directory. Agents that report
# OSC 7501 are handled by Zellij itself. Direct agent launches are also primed
# into the idle state here, so a new or resumed chat is visible in the Inbox.
if [[ -o interactive ]]; then
  # A directly launched coding agent should enter Zellij as idle, not working,
  # and should stay visible in the Inbox while idle. The short working record
  # gives Zellij a status to remember; the idle record is the final state.
  __zj_prime_agent() {
    emulate -L zsh
    [[ -n "${ZELLIJ_SESSION_NAME:-}" && -w /dev/tty ]] || return 0
    local line=$1
    # Use the last simple command in a `cd … && agent` style line.
    line=${line##*&& }
    line=${line##*; }
    line=${line##*| }
    local -a words
    words=(${(z)line})
    (( ${#words} )) || return 0
    local exe=${words[1]:t} sub app
    case "$exe" in
      claude) app=claude-code ;;
      opencode|codex|pi|qwen|copilot|gemini|mcode|muse|vibe|cline|cursor-agent|grok|crush|omp|kilo|antigravity|deepseek|fx) app=$exe ;;
      agy) app=antigravity ;;
      dsh) app=deepseek ;;
      kimi|kimi-cli|kimi-code) app=kimi ;;
      harness)
        (( ${#words} >= 2 )) || return 0
        sub=${words[2]:t}
        if [[ "$sub" == "profile" ]]; then
          (( ${#words} >= 3 )) || return 0
          sub=${words[3]:t}
        fi
        case "$sub" in
          claude*) app=claude-code ;;
          codex*) app=codex ;;
          opencode*) app=opencode ;;
          pi*) app=pi ;;
          qwen*) app=qwen ;;
          copilot*) app=copilot ;;
          gemini*) app=gemini ;;
          kimi*) app=kimi ;;
          mcode*|minimax*) app=mcode ;;
          muse*) app=muse ;;
          vibe*) app=vibe ;;
          cline*) app=cline ;;
          cursor*) app=cursor-agent ;;
          grok*) app=grok ;;
          crush*) app=crush ;;
          omp*) app=omp ;;
          kilo*) app=kilo ;;
          antigravity*|agy*) app=antigravity ;;
          deepseek*|dsh*) app=deepseek ;;
          fx*) app=fx ;;
          *) return 0 ;;
        esac ;;
      *) return 0 ;;
    esac
    case "$1" in
      *" --version"*|*" --help"*|*" -h"*|*" -v"*) return 0 ;;
    esac
    printf '\033]7501;state=working:app=%s\033\\' "$app" >/dev/tty
    printf '\033]7501;state=idle:app=%s\033\\' "$app" >/dev/tty
  }
  __zj_osc133_precmd() {
    local exit_code=$?
    if [[ -w /dev/tty ]]; then
      printf '\033]133;D;%d\033\\' "$exit_code" >/dev/tty
      printf '\033]7;file://%s%s\033\\' "$HOST" "$PWD" >/dev/tty
      printf '\033]133;A\033\\' >/dev/tty
    fi
  }
  __zj_osc133_preexec() {
    if [[ -w /dev/tty ]]; then
      printf '\033]133;C\033\\' >/dev/tty
    fi
  }
  # Run first, while $? still holds the last command's status.
  if (( ! ${precmd_functions[(Ie)__zj_osc133_precmd]} )); then
    precmd_functions=(__zj_osc133_precmd $precmd_functions)
  fi
  if (( ! ${preexec_functions[(Ie)__zj_prime_agent]} )); then
    preexec_functions=(__zj_prime_agent $preexec_functions)
  fi
  if (( ! ${preexec_functions[(Ie)__zj_osc133_preexec]} )); then
    preexec_functions=(__zj_osc133_preexec $preexec_functions)
  fi
fi

# Machine-specific settings and secrets stay outside version control.
[[ ! -r "$HOME/.zshrc.local" ]] || source "$HOME/.zshrc.local"

# OpenCode's background service broadcasts OSC 7501 status to every TUI
# connected to it, so three panes light up when one works. A private server per
# pane keeps statuses independent. Subcommands and explicit --server /
# --standalone invocations are left alone.
opencode() {
  local first=${1:-}
  case "$first" in
    ""|mini|-*) ;;
    upgrade|update|uninstall|acp|api|debug|auth|mcp|plugin|models|stats|run|session|service|reload|pair|serve|completions|help)
      command opencode "$@"
      return
      ;;
  esac
  local arg
  for arg in "$@"; do
    case "$arg" in
      --standalone|--server|--server=*) command opencode "$@"; return ;;
    esac
  done
  command opencode --standalone "$@"
}

# Use the native Antigravity CLI with Google sign-in.
unalias agy 2>/dev/null || true
agy() {
  env -u GEMINI_API_KEY -u GOOGLE_API_KEY -u GOOGLE_GEMINI_BASE_URL \
    "$HOME/.local/libexec/antigravity/agy" \
    --settings "${XDG_CONFIG_HOME:-$HOME/.config}/ai/antigravity/settings.json" "$@"
}

# Coding agents run directly by default; `harness <agent>` is explicit/opt-in.
unalias cline dsh fx gemini kilo kimi-cli muse vibe 2>/dev/null || true

# Portable aliases recovered from the previous Ubuntu installation.
if command -v nvim >/dev/null 2>&1 && ! alias nv >/dev/null 2>&1; then
    alias nv='nvim'
fi

# >>> grok installer >>>
export PATH="$HOME/.grok/bin:$PATH"
fpath=(~/.grok/completions/zsh $fpath)
autoload -Uz compinit && compinit -C
# <<< grok installer <<<

# kimi-code
export PATH="/home/arhammahajan/.kimi-code/bin:$PATH"

# Kimi Code (a Pi derivative) suppresses OSC 7501 reporting inside multiplexers
# unless this is set; Zellij's Inbox uses it for agent state.
export PI_PROGRAM_STATUS=1

# MiniMax Code CLI
export PATH="/home/arhammahajan/.minimax-code/bin:$PATH"

# Qwen Code PATH block begin
export PATH='/home/arhammahajan/.local/bin':$PATH
# Qwen Code PATH block end

# Initialize Starship prompt.
if (( $+commands[starship] )); then
  eval "$(starship init zsh)"
fi

# Initialize zoxide (smart cd via `z` and `zi`).
if (( $+commands[zoxide] )); then
  eval "$(zoxide init zsh)"
fi

# Yazi wrapper: `y` launches yazi and cd's to the final directory on exit (`q`).
if (( $+commands[yazi] )); then
  y() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
    yazi "$@" --cwd-file="$tmp"
    IFS= read -r -d '' cwd < "$tmp"
    [ -n "$cwd" ] && [ "$cwd" != "$PWD" ] && builtin cd -- "$cwd"
    rm -f -- "$tmp"
  }
fi
