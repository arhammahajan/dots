# Oh My Zsh itself and third-party plugins are installed outside this repo.
export ZSH="$HOME/.oh-my-zsh"
export ZSH_CUSTOM="$ZSH/custom"
ZSH_THEME="robbyrussell"

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

# Machine-specific settings and secrets stay outside version control.
[[ ! -r "$HOME/.zshrc.local" ]] || source "$HOME/.zshrc.local"

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

# MiniMax Code CLI
export PATH="/home/arhammahajan/.minimax-code/bin:$PATH"

# Qwen Code PATH block begin
export PATH='/home/arhammahajan/.local/bin':$PATH
# Qwen Code PATH block end
