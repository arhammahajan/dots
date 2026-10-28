# dots

Personal configuration files for **macOS**, **Arch Linux**, and **Fedora Linux**.

- **Editors:** Neovim (lazy.nvim), VSCodium, and Zed.
- **Terminal tools:** WezTerm, tmux, ranger, and lazygit.
- **Shell:** Zsh, Oh My Zsh, completions, autosuggestions, and syntax highlighting.
- **Linux desktop utilities:** polybar, rofi, redshift, zathura, flameshot, and neofetch.
- **Wallpapers:** ultrawide and portrait collections in `walls/`.

## Checkout

```sh
git clone git@github.com:arhammahajan/dots.git ~/Developer/dots
cd ~/Developer/dots
```

For an existing checkout, use `cd ~/Developer/dots`.
Run the setup commands below from the repo root. Back up existing configs before
creating symlinks, and keep the checkout in place after linking it.

## Layout

| Path in repo   | Config destination or use | Contents |
| -------------- | ------------------------- | -------- |
| `nvim/`        | `~/.config/nvim/`          | Lua config, lazy.nvim, LSP, Telescope, Treesitter |
| `vscodium/`    | [VSCodium setup](#vscodium) | Settings, snippets directory, extension list |
| `zed/`         | [Zed setup](#zed)          | Settings and custom themes directory |
| `harnesses/copilot/` | `~/.copilot/settings.json` | GitHub Copilot CLI terminal preferences ([setup](harnesses/copilot/README.md)) |
| `harnesses/grok/` | `~/.grok/config.toml` | Grok Build CLI model, marketplace, and terminal preferences ([setup](harnesses/grok/README.md)) |
| `harnesses/kimi-code/` | `~/.kimi-code/config.toml` | Kimi Code through the central model-router ([setup](harnesses/kimi-code/README.md)) |
| `harnesses/mcode/` | `~/.minimax/config.yaml` | MiniMax Code through the central model-router ([setup](harnesses/mcode/README.md)) |
| `harnesses/muse/` | `~/.config/muse/settings.json` | Muse Code directly through model-router ([setup](harnesses/muse/README.md)) |
| `harnesses/fx/` | `~/.fx/settings.json` and launcher/service | fx through the local model-router ([setup](harnesses/fx/README.md)) |
| `.wezterm.lua` | `~/.wezterm.lua`           | "coolnight" colors, JetBrainsMono Nerd Font, Super+drag |
| `.tmux.conf`   | `~/.tmux.conf`             | Prefix `Ctrl-\`, TPM plugins |
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

Claude, OpenCode, Pi, Qwen, Kimi, MCode, Muse, Antigravity CLI, and fx use the local
[model-router setup](harnesses/README.md). Upstream credentials stay in the router.
A second T3 Codex provider also uses the router. The primary Codex provider,
Grok, and Copilot keep their existing connections.

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

The settings reference BrowserOS MCP at `http://127.0.0.1:9010/mcp` and a local
OpenAI-compatible completion service at `http://localhost:9931/v1/completions`.
On another machine, run those services or adjust the corresponding settings.
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
