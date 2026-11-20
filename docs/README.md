# dots

Personal configuration files for macOS, Arch Linux, and Fedora Linux.

Repository documentation lives in `docs/` and mirrors the configuration directories. All setup commands and configuration paths below are relative to the repository root. Runnable skill packages keep their Markdown instructions and references under `harnesses/shared/skills/` so coding CLIs can discover them and resolve relative files.

| Area | Included configs |
| ---- | ---------------- |
| Editors | Neovim (lazy.nvim), VSCodium, and Zed |
| Terminal tools | Kitty, tmux, Zellij, Yazi, ranger, and lazygit |
| Shell | Zsh, Oh My Zsh, Starship, zoxide, completions, autosuggestions, and syntax highlighting |
| Linux desktop | Niri, Fuzzel, Quickshell, Waypaper, Wallflow, polybar, rofi, redshift, zathura, flameshot, and neofetch |
| Coding CLIs | Shared router config, MCP setup, hooks, and skills under `harnesses/` |
| Wallpapers | `16:9/`, `uw/`, and `vertical/` collections in `walls/` |

## Checkout

```sh
git clone git@github.com:arhammahajan/dots.git ~/dev/dots
cd ~/dev/dots
```

For an existing checkout, run `cd ~/dev/dots`. Run all setup commands from the repository root. Back up existing configs before creating symlinks, and keep this checkout in place after linking it.

## Repository layout

| Path in repo | Config destination or use | Contents |
| ------------ | ------------------------- | -------- |
| `docs/` | Repository documentation | Setup guides and coding CLI references |
| `nvim/` | `~/.config/nvim/` | Lua config, lazy.nvim, LSP, Telescope, Treesitter |
| `vscodium/` | [VSCodium setup](#vscodium) | Settings, snippets directory, extension list |
| `zed/` | [Zed setup](#zed) | Settings and custom themes directory |
| `harnesses/` | Linked by `harnesses/setup.sh` | Configs, hooks, services, and shared skills ([overview](harnesses/README.md)) |
| `harnesses/copilot/` | `~/.copilot/settings.json` | GitHub Copilot CLI terminal preferences ([setup](harnesses/copilot/README.md)) |
| `harnesses/grok/` | `~/.grok/config.toml` | Grok Build CLI model, marketplace, and terminal preferences ([setup](harnesses/grok/README.md)) |
| `harnesses/kimi-code/` | `~/.kimi-code/config.toml` | Kimi Code config for model-router ([setup](harnesses/kimi-code/README.md)) |
| `harnesses/mcode/` | `~/.minimax/config.yaml` | MiniMax Code config for model-router ([setup](harnesses/mcode/README.md)) |
| `harnesses/muse/` | `~/.config/muse/settings.json` | Muse Code config for model-router ([setup](harnesses/muse/README.md)) |
| `harnesses/kilo/` | `~/.config/kilo/kilo.json` | Kilo Code CLI config for model-router ([setup](harnesses/kilo/README.md)) |
| `harnesses/fx/` | `~/.fx/settings.json` | fx config and MCP file for model-router ([setup](harnesses/fx/README.md)) |
| `harnesses/vibe/` | `~/.vibe/config.toml` | Mistral Vibe config for model-router ([setup](harnesses/vibe/README.md)) |
| `kitty/kitty.conf` | `~/.config/kitty/kitty.conf` | JetBrainsMono Nerd Font, obsidian and slate-blue palette, 0.55 opacity |
| `tmux/tmux.conf` | `~/.config/tmux/tmux.conf` | Agent multiplexer config with `Ctrl-Space` leader and `tmux-spaces` |
| `zellij/config.kdl` | `~/.config/zellij/config.kdl` | Agent multiplexer config with OSC 7501 inbox and `Ctrl-b` leader ([setup](zellij.md)) |
| `~/dev/model-router/plugins/zellij-spaces` | `~/.local/bin/zj`, `~/.config/zellij/plugins/` | Rust Spaces navigator WASM plugin and Go `zj` symlink, installed by `scripts/setup-zellij.sh` ([setup](zellij.md)) |
| `~/dev/model-router/plugins/tmux-spaces` | `~/.local/bin/tmux-spaces` | Go Spaces navigator and plugin for tmux, installed by `scripts/setup-tmux.sh` |
| `.zshenv` | `~/.zshenv` | User tool paths, fnm, pnpm, CUDA, `EDITOR=nvim`, `TERMINAL=kitty`, router env |
| `.zshrc` | `~/.zshrc` | Oh My Zsh, completion, autosuggestions, syntax highlighting, Starship, zoxide, Yazi wrapper |
| `starship/starship.toml` | `~/.config/starship.toml` | Prompt layout with Nerd Font icons, Git status, runtime versions, and clock |
| `yazi/` | `~/.config/yazi/` | `yazi.toml`, `init.lua`, `theme.toml`, and `theme.tmTheme` |
| `ranger/` | `~/.config/ranger/` | File manager config with vendored `ranger_devicons` plugin |
| `lazygit/` | `~/.config/lazygit/` | Empty `config.yml` placeholder |
| `niri/` | `~/.config/niri/` | Modular KDL config for the Niri Wayland compositor |
| `fuzzel/` | `~/.config/fuzzel/` | Launcher config and menus for clipboard, idle time, monitor layout, and power profile |
| `quickshell/` | `~/.config/quickshell/` | Bar, display menu, lockscreen, and power menu modules |
| `waypaper/` | `~/.config/waypaper/` | `config.ini` and `style.css` wallpaper picker config |
| `wallflow/` | `~/.config/wallflow/` | `config.yml` wallpaper downloader config |
| `scripts/` | Run from repo root or Niri binds | Setup scripts (`setup-zsh.sh`, `setup-zellij.sh`, `setup-tmux.sh`) and desktop helpers |
| `polybar/` | `~/.config/polybar/` | `config.ini`, `modules.ini`, `colors.ini`, and `launch.sh` |
| `rofi/` | `~/.config/rofi/` | `config.rasi` and `catppuccin.rasi` |
| `redshift/` | `~/.config/redshift/` | `redshift.conf` with manual `lat` and `lon` coordinates |
| `zathura/` | `~/.config/zathura/` | `zathurarc` with Dracula colors |
| `flameshot/` | `~/.config/flameshot/` | `flameshot.ini` screenshot config |
| `neofetch/` | `~/.config/neofetch/` | `config.conf` and custom `logo` |
| `walls/` | Wallpaper collection | `16:9/`, `uw/`, `vertical/`, and downloaded backgrounds |

## Coding CLI setup

Claude Code, Cline CLI, OpenCode, Pi, Qwen Code, Kimi Code, MCode, Muse, Antigravity CLI, Gemini CLI, fx, DeepSeek, Mistral Vibe, and Kilo Code route requests through the local [model-router setup](harnesses/README.md). Upstream credentials stay in the router. Second T3 Codex and Grok Build providers also use the router. The primary Codex and Grok providers and GitHub Copilot CLI keep their direct connections.

Run `./harnesses/setup.sh` from the repository root to link configs, systemd user units, and fixed-path skills directories. Run `./harnesses/setup.sh --check` to verify every symlink without modifying files.

Every coding CLI configuration includes the shared Helium `agent-browser` MCP server. See the [browser MCP setup](harnesses/browser-mcp.md) for config destinations and launcher requirements.

GitHub Copilot CLI terminal preferences live in `harnesses/copilot/settings.json`. See the [Copilot setup instructions](harnesses/copilot/README.md) to link that file into `~/.copilot/` (or `$COPILOT_HOME`). Authentication and runtime state stay outside the repository.

## Zsh

Install Zsh and Git with your system package manager. Use `sudo dnf install zsh git` on Fedora, `sudo pacman -S zsh git` on Arch, or `brew install zsh git` on macOS. Then run the setup script from the repository root:

```sh
bash scripts/setup-zsh.sh
```

`scripts/setup-zsh.sh` performs these steps:

1. Clones [Oh My Zsh](https://github.com/ohmyzsh/ohmyzsh) into `~/.oh-my-zsh`.
2. Clones `zsh-users/zsh-completions`, `zsh-users/zsh-autosuggestions`, and `zsh-users/zsh-syntax-highlighting` into `~/.oh-my-zsh/custom/plugins/`.
3. Copies `.bashrc`, `.bash_profile`, and `.profile` and moves any existing `.zshenv`, `.zshrc`, or `starship.toml` into `~/.local/share/dots-backups/zsh-<timestamp>-<pid>/` (or `$XDG_DATA_HOME/dots-backups/`).
4. Checks `.zshenv` and `.zshrc` with `zsh -n`, then symlinks `.zshenv`, `.zshrc`, and `starship/starship.toml` into place.
5. Verifies interactive login startup and runs `chsh -s` to make Zsh the login shell.

Oh My Zsh, downloaded plugins, shell history, and caches stay outside Git. `chsh` may prompt for your account password. Log out and back in so terminals and desktop apps pick up Zsh. Bash stays installed for scripts that require it.

`.zshrc` disables Oh My Zsh themes (`ZSH_THEME=""`) and initializes Starship when installed. Starship prints the current directory, Git branch and status, command duration, active Python/Node/Go/Rust versions, and time. Tab opens the completion menu, Right Arrow accepts an autosuggestion, and Up or Down searches history by the current prefix. Built-in plugins add Git aliases, `extract` for archives, history helpers, and double-Escape to prepend `sudo`. When installed, `zoxide` provides `z` and `zi`, and the `y` shell function opens Yazi and changes into its last directory on exit.

`.zshenv` sets `EDITOR=nvim`, `VISUAL=nvim`, and `TERMINAL=kitty`, and sources `~/.config/ai/model-router.env`. It also prepends `~/.local/bin`, `~/bin`, `~/.opencode/bin`, fnm, pnpm, and `/usr/local/cuda/bin` (when present) to `PATH`. `.zshrc` runs `fnm env --use-on-cd --shell zsh` to switch Node versions when you enter a directory with a version file. It then puts `~/.local/bin`, `~/.opencode/bin`, and `$PNPM_HOME/bin` back ahead of Node's bundled shims. Interactive hooks emit OSC 133 prompt markers, OSC 7 working directory updates, and OSC 7501 state priming for coding CLIs inside Zellij. Put machine-specific overrides and secrets in `~/.zshrc.local`.

## Zellij

Zellij 0.46 is configured as an agent terminal multiplexer modeled on Herdr. Status tracking uses Zellij's built-in Inbox and the OSC 7501 program-status protocol. The Spaces navigator plugin is built from Rust source in `~/dev/model-router/plugins/zellij-spaces`, and the `zj` helper is a symlink to the Go `harness` binary from the same monorepo. The navigator uses a Catppuccin Mocha theme with grouped spaces and agents, attention filters, direct jumps, and inline dialogs. No patched Zellij binary or sidebar is required.

For Gemini, Cursor, older Qwen, Cline, Antigravity, and Kilo, which do not emit OSC 7501, the navigator reads pane viewports through `zj __space-snapshot`. The [Zellij guide](zellij.md) documents the architecture, agent support matrix, verification checks, and limitations.

Install the pinned v0.46.0 release binary, Spaces navigator, and `zj` symlink:

```sh
bash scripts/setup-zellij.sh
zellij setup --check
```

`scripts/setup-zellij.sh` downloads the v0.46.0 archive for the current platform, checks its SHA-256 digest, and installs `~/.local/bin/zellij` and `~/.config/zellij/plugins/zellij-spaces.wasm`. It links `~/.local/bin/zj` to `~/.local/bin/harness`, seeds `~/.cache/zellij/permissions.kdl`, backs up replaced files to `~/.local/share/dots-backups/`, and symlinks `zellij/config.kdl` to `~/.config/zellij/config.kdl`. Set `ZJ_SPACES_REPO` to override the plugin directory. Run the script again to refresh files for the pinned version. After editing plugin source code, rebuild the WASM module with `bash ~/dev/model-router/plugins/zellij-spaces/build.sh`.

Sessions correspond to spaces, and tabs correspond to agents or tasks. `Ctrl-b` is the leader, with `Ctrl-Space` as a fallback, and sessions start locked so agent TUIs receive every keystroke. `Ctrl-b w` opens the Spaces navigator popup, `Ctrl-b b` or `Ctrl-b i` opens the stock session manager, `Ctrl-b Shift-n` opens the new-space form, and `Ctrl-b o` jumps to the next attention target across spaces. From the shell, `zj space list|new|switch|rename|close` manages spaces and `zj doctor` reports config, spaces, target, and client state.

## tmux

`tmux/tmux.conf` adapts the Zellij agent-multiplexer workflow to tmux. It sets `Ctrl-Space` as the prefix and leaves bare Alt chords unbound so agent TUIs keep Alt keybindings. The config applies Catppuccin Mocha colors, enables vi copy mode with OSC 52 clipboard support, and loads `tmux-spaces` from `~/dev/model-router/plugins/tmux-spaces`.

Install the `tmux-spaces` binary and link the config:

```sh
bash scripts/setup-tmux.sh
```

`scripts/setup-tmux.sh` builds `tmux-spaces` if the binary is missing, backs up replaced files to `~/.local/share/dots-backups/tmux-<timestamp>/`, symlinks `tmux/tmux.conf` to `~/.config/tmux/tmux.conf`, and symlinks `tmux-spaces` to `~/.local/bin/tmux-spaces`. Set `TMUX_SPACES_REPO` to override the plugin directory.

## VSCodium

`vscodium/settings.json` enables delayed autosave, Continue tab autocomplete, experimental modern UI, the Rosé Pine Moon (no italics) color theme, and parent-folder Git repository detection. `vscodium/extensions.txt` lists installed extensions. `vscodium/snippets/` is empty apart from its Git placeholder.

To link these files on macOS, back up any existing settings and snippets, then run from the repository root:

```sh
user_dir="$HOME/Library/Application Support/VSCodium/User"
mkdir -p "$user_dir"
ln -s "$PWD/vscodium/settings.json" "$user_dir/settings.json"
ln -s "$PWD/vscodium/snippets" "$user_dir/snippets"
```

On Linux, replace the first line with `user_dir="${XDG_CONFIG_HOME:-$HOME/.config}/VSCodium/User"`.

Restore extensions when `codium` is on `PATH`:

```sh
while IFS= read -r extension; do
  codium --install-extension "$extension"
done < vscodium/extensions.txt
```

Refresh the tracked list after adding or removing extensions with `codium --list-extensions | sort > vscodium/extensions.txt`. Extension binaries, caches, history, workspace state, and `~/.vscode-oss/argv.json` stay outside the repository.

## Zed

`zed/settings.json` configures Vim mode, one-second autosave, UI font size 16, buffer font size 15, system light and dark themes using Gruvbox Light Hard and Paraíso Dark, disabled telemetry, and agent server entries. `zed/themes/` is empty apart from its Git placeholder.

Back up any existing settings and themes, then link them from the repository root:

```sh
zed_dir="$HOME/.config/zed"
mkdir -p "$zed_dir"
ln -s "$PWD/zed/settings.json" "$zed_dir/settings.json"
ln -s "$PWD/zed/themes" "$zed_dir/themes"
```

The prompt database, installed extensions, conversations, and application state stay outside the repository.

`zed/settings.json` points edit predictions at `http://localhost:9931/v1/completions`. On another machine, run that local OpenAI-compatible completion service or update `edit_predictions`. Install the selected themes and configure agent providers separately.

## Shared coding-agent skills

Shared skills live in `harnesses/shared/skills/<skill-name>/SKILL.md`, with any scripts, references, and assets beside each file. Every connected coding CLI reads from this catalog, including Gemini CLI.

There are no per-tool skill folders. Tools with an explicit skills setting point directly at `~/dev/dots/harnesses/shared/skills`. Tools with a fixed user skills directory get a symlink to that directory from `harnesses/setup.sh`. Codex links the directory at `~/.codex/skills/shared` so its vendor-managed `.system` folder stays separate. Muse owns its store directory, so `harnesses/setup.sh` links each skill entry inside `~/.config/muse/skills/`. The installed DeepSeek release has no skill discovery and has no skills path. See the [shared skills table](harnesses/README.md#shared-skills) for every destination and config key.

After adding or editing a skill in `harnesses/shared/skills/`, run `./harnesses/setup.sh` and reload or restart active sessions to refresh discovery. Required MCP tools, plugins, and credentials remain configured per tool. Rerun `./harnesses/setup.sh` if this checkout moves.

Relative links inside skill files resolve from the file that contains them, so references copied from an upstream plugin root can break when skills are placed in sibling folders. The vendored pstack catalog and its upstream sync steps are documented in [`harnesses/shared/skills/UPSTREAM.md`](../harnesses/shared/skills/UPSTREAM.md).

## Key bindings

### Zellij

Leader is `Ctrl-b`. The session starts locked, and every leader action returns to locked.

| Shortcut | Action |
| -------- | ------ |
| `Ctrl-b c` / `Ctrl-b Shift-x` | New tab / close tab |
| `Ctrl-b n` / `p` / `1..9` | Next / previous / numbered tab |
| `Ctrl-b v` / `-` | Split right / down |
| `Ctrl-b h/j/k/l` | Focus pane |
| `Ctrl-b x` / `z` | Close pane / toggle zoom |
| `Ctrl-b b` / `i` | Inbox (stock session manager) |
| `Ctrl-b o` | Jump to next attention target |
| `Ctrl-b a` | Agents selector (filterable list of coding CLIs; opens in a new tab) |
| `Ctrl-b w` | Spaces navigator (grouped spaces and agents) |
| `Ctrl-b Shift-n` | Spaces navigator on the new-space form |
| `Ctrl-b Shift-w` | Rename the current space |
| `Ctrl-b q` / `d` | Detach |
| `Ctrl-Space` | Fallback leader when an outer multiplexer consumes `Ctrl-b` |

See the [Zellij guide](zellij.md) for the full key map and the `zj wait`, `zj status`, `zj prompt`, and `zj read` commands.

### tmux

Prefix is `Ctrl-Space`.

| Shortcut | Action |
| -------- | ------ |
| `Ctrl-Space c` / `X` | New window / close window |
| `Ctrl-Space n` / `b` / `1..9` | Next / previous / numbered window |
| `Ctrl-Space v` / `-` | Split right / down |
| `Ctrl-Space h/j/k/l` | Focus pane |
| `Ctrl-Space p` / `t` / `r` | Pane cycling / tab cycling / resize mode |
| `Ctrl-Space x` / `z` | Close pane / toggle zoom |
| `Ctrl-Space w` / `i` / `a` | Spaces navigator / inbox / agents selector |
| `Ctrl-Space N` / `W` | New space / rename current space |
| `Ctrl-Space o` | Jump to next attention target |
| `Ctrl-Space e` / `[` | Edit scrollback in `$EDITOR` / enter copy mode |
| `Ctrl-Space q` / `d` / `R` | Detach / reload `tmux.conf` |

### Neovim

Leader is `Space`. `<leader>ff` runs Telescope find files, `<leader>fs` runs live grep, `<leader>ee` toggles the file tree, `<leader>nh` clears search highlights, and `jk` exits insert mode. See `nvim/lua/arham/core/keymaps.lua` and the plugin files under `nvim/lua/arham/plugins/` for the rest.

## Credits

Parts of this repository are adapted from two open-source setups.

### [josean-dev/dev-environment-files](https://github.com/josean-dev/dev-environment-files) by Josean Martinez

The Neovim configuration follows Josean's `lua/arham/` layout, `lazy.nvim` bootstrap, and `mason.nvim` LSP and formatter list. It keeps his plugin set of Telescope, nvim-cmp, LuaSnip, Treesitter, nvim-tree, lualine, gitsigns, which-key, trouble, conform, nvim-lint, and auto-session. `colorscheme.lua` adapts his `tokyonight.nvim` setup to an obsidian and slate-blue palette.

### [Zproger/bspwm-dotfiles](https://github.com/Zproger/bspwm-dotfiles) by ZProger

ZProger's Arch Linux setup is the basis for `polybar` (`config.ini`, `modules.ini`, `colors.ini`, `launch.sh`), `rofi` (`config.rasi` and `catppuccin.rasi`), `ranger`, `redshift`, `zathura`, and `flameshot`.

### Other third-party components

- `ranger/plugins/ranger_devicons/` vendors [`ranger_devicons`](https://github.com/alexanderjeurissen/ranger_devicons) by Alexander Jeurissen along with its MIT and Nerd Font licenses.
- Rofi, tmux, and Zellij use [Catppuccin](https://github.com/catppuccin) palettes.
- [tokyonight.nvim](https://github.com/folke/tokyonight.nvim), [lazy.nvim](https://github.com/folke/lazy.nvim), and the plugins listed in `nvim/lazy-lock.json` each carry their own licenses.
- Zathura uses the Dracula color palette.

## License

No blanket license applies to this repository. Configs adapted from upstream projects remain subject to their original licenses; check each upstream repository before redistributing. Personal scripts come as-is without warranty.
