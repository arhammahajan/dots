# dots

Personal configuration files for **macOS**, **Arch Linux**, and **Fedora Linux**.

Repository documentation lives in `docs/`, mirroring the configuration directories.
Setup commands and configuration paths are relative to the repository root.
Runnable skill packages keep their Markdown instructions and references together
under `harnesses/shared/skills/` for harness discovery and relative resource loading.

- **Editors:** Neovim (lazy.nvim), VSCodium, and Zed.
- **Terminal tools:** WezTerm, Zellij (agent multiplexer), tmux, ranger, and lazygit.
- **Shell:** Zsh, Oh My Zsh, completions, autosuggestions, and syntax highlighting.
- **Linux desktop utilities:** polybar, rofi, redshift, zathura, flameshot, and neofetch.
- **Wallpapers:** ultrawide and portrait collections in `walls/`.

## Checkout

```sh
git clone git@github.com:arhammahajan/dots.git ~/dev/dots
cd ~/dev/dots
```

For an existing checkout, use `cd ~/dev/dots`.
Run the setup commands below from the repo root. Back up existing configs before
creating symlinks, and keep the checkout in place after linking it.

## Layout

| Path in repo   | Config destination or use | Contents |
| -------------- | ------------------------- | -------- |
| `docs/`        | Repository documentation | Setup guides and harness configuration references |
| `nvim/`        | `~/.config/nvim/`          | Lua config, lazy.nvim, LSP, Telescope, Treesitter |
| `vscodium/`    | [VSCodium setup](#vscodium) | Settings, snippets directory, extension list |
| `zed/`         | [Zed setup](#zed)          | Settings and custom themes directory |
| `harnesses/copilot/` | `~/.copilot/settings.json` | GitHub Copilot CLI terminal preferences ([setup](harnesses/copilot/README.md)) |
| `harnesses/grok/` | `~/.grok/config.toml` | Grok Build CLI model, marketplace, and terminal preferences ([setup](harnesses/grok/README.md)) |
| `harnesses/kimi-code/` | `~/.kimi-code/config.toml` | Kimi Code through the central model-router ([setup](harnesses/kimi-code/README.md)) |
| `harnesses/mcode/` | `~/.minimax/config.yaml` | MiniMax Code through the central model-router ([setup](harnesses/mcode/README.md)) |
| `harnesses/muse/` | `~/.config/muse/settings.json` | Muse Code directly through model-router ([setup](harnesses/muse/README.md)) |
| `harnesses/kilo/` | `~/.config/kilo/kilo.json` and launcher | Kilo Code CLI through the local model-router ([setup](harnesses/kilo/README.md)) |
| `harnesses/fx/` | `~/.fx/settings.json` and launcher/service | fx through the local model-router ([setup](harnesses/fx/README.md)) |
| `harnesses/vibe/` | `~/.vibe/config.toml` and launcher | Mistral Vibe through the local model-router ([setup](harnesses/vibe/README.md)) |
| `.wezterm.lua` | `~/.wezterm.lua`           | "coolnight" colors, JetBrainsMono Nerd Font, Super+drag |
| `.tmux.conf`   | `~/.tmux.conf`             | Prefix `Ctrl-\`, TPM plugins |
| `zellij/config.kdl` | `~/.config/zellij/config.kdl` | Agent multiplexer: OSC 7501 inbox, `Ctrl-b` leader ([setup](zellij.md)) |
| `zellij/zj`    | `~/.local/bin/zj`          | Session/agent control for Zellij ([setup](zellij.md)) |
| `zellij/plugins/` | `~/.config/zellij/plugins/` | `zellij-spaces` navigator WASM; source in `zellij/plugins/spaces/` ([setup](zellij.md)) |
| `.zshenv`      | `~/.zshenv`               | User tool paths, fnm and pnpm locations |
| `.zshrc`       | `~/.zshrc`                | Oh My Zsh, completion, suggestions, highlighting |
| `ranger/`      | `~/.config/ranger/`        | File manager with vendored `ranger_devicons` plugin |
| `lazygit/`     | `~/.config/lazygit/`       | Placeholder (empty `config.yml`) |
| `polybar/`     | `~/.config/polybar/`       | `config.ini`, `modules.ini`, `colors.ini`, `launch.sh` |
| `rofi/`        | `~/.config/rofi/`          | `config.rasi` and Catppuccin theme |
| `redshift/`    | `~/.config/redshift/`      | Manual location — edit `lat`/`lon` |
| `zathura/`     | `~/.config/zathura/`       | Dracula theme |
| `flameshot/`   | `~/.config/flameshot/`     | Screenshot tool |
| `neofetch/`    | `~/.config/neofetch/`      | `config.conf` and custom `logo` |
| `walls/`       | Wallpaper collection      | `uw/` for ultrawide, `vertical/` for portrait |

## Coding harnesses

Claude, OpenCode, Pi, Qwen, Kimi, MCode, Muse, Antigravity CLI, fx, and Kilo Code use the local
[model-router setup](harnesses/README.md). Upstream credentials stay in the router.
Second T3 Codex and Grok Build providers also use the router. The primary Codex
and Grok providers and Copilot keep their existing connections.

All coding harnesses include the shared Helium browser MCP servers:
agent-browser, Playwright, and Browser Use. See the [browser MCP setup](harnesses/browser-mcp.md)
for their config destinations and local launcher requirements.

## Zsh

Install Zsh and Git with your package manager (`sudo dnf install zsh git` on
Fedora, `sudo pacman -S zsh git` on Arch, or `brew install zsh git` on macOS).
Then run from this checkout:

```sh
bash scripts/setup-zsh.sh
```

The script clones [Oh My Zsh](https://github.com/ohmyzsh/ohmyzsh) into
`~/.oh-my-zsh`, installs the upstream `zsh-users/zsh-completions`,
`zsh-users/zsh-autosuggestions`, and `zsh-users/zsh-syntax-highlighting` plugins,
and symlinks `.zshenv` and `.zshrc` from this repo into your home directory.
Existing Zsh files and recovery copies of Bash configs are saved under
`~/.local/share/dots-backups/zsh-<timestamp>-<pid>/` (or `$XDG_DATA_HOME/dots-backups/`).
Oh My Zsh, downloaded plugins, history, and caches stay outside Git.

After checking shell startup, it uses `chsh` to make Zsh your login shell;
this may ask for your account password. Log out and back in afterward so
terminals and applications pick up the new shell. Bash remains installed for
scripts that require it. The WezTerm and tmux configs also explicitly start Zsh
for new terminals and panes.

The prompt shows the current directory and Git branch. Tab opens completion
choices; Right Arrow accepts a gray suggestion; Up/Down search history using
what you have typed. Commands are highlighted as you type. Built-in plugins
provide Git shortcuts, archive extraction with `extract`, and double-Escape
to prefix a command with `sudo`.

The existing fnm, pnpm, local CLI, and OpenCode paths are preserved, and fnm
switches Node versions when entering a directory with a version file. Put
machine-specific overrides or secrets in `~/.zshrc.local`.

## Zellij

Zellij 0.46 is configured as an agent terminal multiplexer modeled on Herdr.
Agent status uses the built-in Inbox and **OSC 7501** program-status protocol.
The **Spaces navigator** is a purpose-built plugin (`zellij/plugins/spaces/`,
tracked WASM at `zellij/plugins/zellij-spaces.wasm`) with the Herdr-inspired
Catppuccin UI: grouped spaces and agents, attention filters, direct jumps and
inline dialogs. No patched
Zellij binary and no sidebar are involved. The [Zellij guide](zellij.md) covers
the architecture, agent support matrix, verification results and limitations.

Install the pinned v0.46.0 release binary, Spaces navigator and helper:

```sh
bash scripts/setup-zellij.sh
zellij setup --check
```

The script downloads the release for the current platform, verifies its
SHA-256, installs `~/.local/bin/zellij` and the tracked navigator WASM, seeds its
permission cache, and symlinks `zellij/config.kdl` and `zellij/zj` into place.
Run it again to update within the pinned version. Rebuild the plugin with
`scripts/build-zellij-spaces.sh` after changing `zellij/plugins/spaces/`.

Sessions are Herdr spaces, tabs are agents or tasks. `Ctrl-b w` opens the Spaces
navigator as a popup: grouped spaces and agents, attention filters, direct jump,
and inline new/rename/close dialogs. `Ctrl-b b`/`i` opens the stock session
manager as before. `Ctrl-b Shift-n` opens the navigator on its new-space form.
`zj space list|new|switch|rename|close` exposes spaces from the shell, and
`zj doctor` shows config, spaces, target and client state. The `Ctrl-b` leader mirrors
Herdr's prefix (with `Ctrl-Space` as a fallback); the session starts locked so
agent TUIs receive every keystroke. `Ctrl-b o` jumps to whatever needs attention
across spaces.

## VSCodium

`vscodium/settings.json` enables autosave and Continue tab completion, selects
Rosé Pine Moon (no italics), and configures parent-folder Git repository detection.
`extensions.txt` records the installed extensions; `snippets/` is currently empty
apart from its Git placeholder.

On this Mac, `~/Library/Application Support/VSCodium/User/settings.json` and
`snippets` are symlinks to `vscodium/settings.json` and `vscodium/snippets`
in this checkout. Edits through VSCodium therefore update the files in `dots`.
Originals are backed up under `~/.local/share/dots-backups/vscodium-<timestamp>/`.

To link these configs on another Mac, back up any existing settings and snippets,
then run from the repo root:

```sh
user_dir="$HOME/Library/Application Support/VSCodium/User"
mkdir -p "$user_dir"
ln -s "$PWD/vscodium/settings.json" "$user_dir/settings.json"
ln -s "$PWD/vscodium/snippets" "$user_dir/snippets"
```

On Linux, replace the `user_dir` assignment with
`user_dir="${XDG_CONFIG_HOME:-$HOME/.config}/VSCodium/User"`.

With the `codium` CLI available, restore extensions from the repo root with:

```sh
while IFS= read -r extension; do
  codium --install-extension "$extension"
done < vscodium/extensions.txt
```

Refresh the list after installing or removing extensions with
`codium --list-extensions | sort > vscodium/extensions.txt`.
Extension binaries, caches, history, workspace state, and machine-specific
`~/.vscode-oss/argv.json` stay outside the repo.

## Zed

`zed/settings.json` contains Vim mode, one-second autosave, font sizes, light/dark
theme selections, telemetry preferences, and agent configuration. `zed/themes/`
is currently empty apart from its Git placeholder.

On this Mac, `~/.config/zed/settings.json` and `themes` are symlinks to
`zed/settings.json` and `zed/themes` in this checkout. Edits through Zed therefore
update the files in `dots`. Originals are backed up under
`~/.local/share/dots-backups/zed-<timestamp>/`.

To link these configs on another computer, back up any existing settings and
themes, then run from the repo root:

```sh
zed_dir="$HOME/.config/zed"
mkdir -p "$zed_dir"
ln -s "$PWD/zed/settings.json" "$zed_dir/settings.json"
ln -s "$PWD/zed/themes" "$zed_dir/themes"
```

The prompt database, installed extensions, conversations, and application state
stay outside the repo.

The settings reference a local OpenAI-compatible completion service at
`http://localhost:9931/v1/completions`. On another machine, run that service or
adjust the corresponding settings.
Install the selected themes and configure agent providers separately.

---

## GitHub Copilot CLI

`harnesses/copilot/settings.json` contains the CLI's terminal preferences.
See the [Copilot setup instructions](harnesses/copilot/README.md) to link it into
`~/.copilot/` (or `$COPILOT_HOME`). Authentication and runtime state stay outside
the repo.

---

## Key bindings

### tmux

| Shortcut            | Action                                    |
| ------------------- | ----------------------------------------- |
| `Ctrl-\`            | Prefix (replaces `Ctrl-b`)                |
| `Prefix + \|` / `-` | Split horizontal / vertical (current dir) |
| `Prefix + c`        | New window (current dir)                  |
| `Prefix + h/j/k/l`  | Resize panes (`-r`, repeatable)           |
| `Prefix + m`        | Toggle zoom                               |
| `Prefix + r`        | Reload config                             |
| `Prefix + I`        | Install TPM plugins                       |

### Zellij

Leader is `Ctrl-b` (same as Herdr's default prefix). The session is locked by
default; every leader action returns to locked.

| Shortcut | Action |
| -------- | ------ |
| `Ctrl-b c` / `Ctrl-b Shift-x` | New tab / close tab |
| `Ctrl-b n` / `p` / `1..9` | Next / previous / numbered tab |
| `Ctrl-b v` / `-` | Split right / down |
| `Ctrl-b h/j/k/l` | Focus pane |
| `Ctrl-b x` / `z` | Close pane / zoom |
| `Ctrl-b b` / `i` | Inbox (stock session manager) |
| `Ctrl-b o` | Jump to next attention target |
| `Ctrl-b w` | Spaces navigator (grouped spaces + agents) |
| `Ctrl-b Shift-n` | Spaces navigator, opened on the new-space form |
| `Ctrl-b Shift-w` | Rename the current space |
| `Ctrl-b q` | Detach |
| `Ctrl-Space` | Fallback leader if `Ctrl-b` is swallowed by an outer multiplexer |

See the [Zellij guide](zellij.md) for the full map and `zj wait`, `zj status`,
`zj prompt` and `zj read` helpers.

### Neovim

Leader is `Space`. Highlights: `<leader>ff` Telescope find files, `<leader>fs`
live grep, `<leader>ee` file tree, `<leader>nh` clear highlights, `jk` to leave
insert mode. See `nvim/lua/arham/core/keymaps.lua` and each plugin file.

---

## Credits

Large parts of this repo are adapted from two excellent open-source projects.
All credit for the original structure, ideas, and configuration goes to their authors.

### [josean-dev/dev-environment-files](https://github.com/josean-dev/dev-environment-files) — Josean Martinez

The development-environment side of this repo is based on Josean's setup:

- **Neovim** — the `lua/arham/` layout, `lazy.nvim` bootstrap, and the plugin
  selection (Telescope, nvim-cmp + LuaSnip, Treesitter, nvim-tree, lualine,
  gitsigns, which-key, trouble, conform, nvim-lint, auto-session, and friends)
  all follow his config. `colorscheme.lua` is a lightly customised version of
  his `tokyonight.nvim` theme, and LSP servers/formatters via `mason.nvim`
  mirror his list.
- **WezTerm** — `.wezterm.lua`, including the "coolnight" palette.
- **tmux** — `.tmux.conf` derives from his (sensible defaults, TPM,
  `vim-tmux-navigator`, `tmux-resurrect` / `tmux-continuum`, `tmux-tokyo-night`),
  with a custom `Ctrl-\` prefix and extra keybindings.

### [Zproger/bspwm-dotfiles](https://github.com/Zproger/bspwm-dotfiles) — ZProger

The desktop utility configs are based on ZProger's Arch Linux setup:

- **polybar** — `config.ini`, `modules.ini`, `colors.ini`, `launch.sh`.
- **rofi** — `config.rasi` and the bundled Catppuccin theme.
- **ranger, redshift, zathura, flameshot** — configs carried over and adapted
  from his `config/`.

If you like this setup, please go star and follow those repositories — they are
the reason it exists.

### Other third-party components

- [`ranger_devicons`](https://github.com/alexanderjeurissen/ranger_devicons) by
  Alexander Jeurissen — vendored under `ranger/plugins/ranger_devicons/`
  (MIT / Nerd Font licenses included alongside it).
- [Catppuccin](https://github.com/catppuccin) — rofi theme.
- [tokyonight.nvim](https://github.com/folke/tokyonight.nvim),
  [lazy.nvim](https://github.com/folke/lazy.nvim) and the many plugins listed in
  `nvim/lazy-lock.json` — each under its own license.
- Dracula — zathura color palette.

---

## License

No blanket license is applied to this repository. Configuration adapted from the
projects above remains subject to **their** licenses; check each upstream repo
before redistributing. Personal scripts here are shared as-is,
with no warranty.

## Shared coding-agent skills

Personal skills live in `harnesses/shared/skills/<skill-name>/SKILL.md`, with any scripts,
references, and assets beside that file. This directory is the single source
for the connected coding harnesses on this machine, including Gemini CLI.

There are no per-harness skill folders. Harnesses with an explicit skills
setting point directly at `~/dev/dots/harnesses/shared/skills`; harnesses with a
fixed user skills directory link that directory to the same place. Codex links
it at `~/.codex/skills/shared` so its vendor-managed `.system` directory stays
separate, and Muse keeps its store-owned directory with per-entry links. See the
[per-harness skills table](harnesses/README.md#shared-skills) for each
destination and setting. The DeepSeek Harness release installed here has no
skill discovery, so it is the only harness without a skills path.

Add or edit skills in `shared/skills`, run `harnesses/setup.sh`, then reload or
restart active harness sessions to refresh discovery. Every harness sees the new
skill directly from the shared catalog. Required MCP tools, plugins, and
credentials remain configured per harness. These links are local to this machine;
rerun `harnesses/setup.sh` if the checkout moves.

Relative links between skill files are resolved from the file that contains
them, so a reference written for an upstream plugin root can miss once the skills
are split into sibling directories. The workflow skills (HyperFrames, media-use,
motion-graphics, and the two video routes) were repaired on that basis: a path
that named a shared file is now relative to the skill that holds it, and paths
into files those releases never shipped are left untouched. The three vendored
catalogs are documented separately in
[`shared/skills/UPSTREAM.md`](../harnesses/shared/skills/UPSTREAM.md).
