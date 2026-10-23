# dots

Personal configuration files for **Arch Linux** and **macOS**, with a
Neovim/Tmux/WezTerm development environment, VSCodium, and desktop utilities.

```
Bar       polybar        Keys       sxhkd           Launcher    rofi
Terminal  wezterm        Multiplex  tmux            Editor      Neovim (lazy.nvim)
Files     ranger         PDF        zathura
```

---

## Layout

| Path in repo        | Installs to             | Notes                                                              |
| ------------------- | ----------------------- | ------------------------------------------------------------------ |
| `sxhkd/`            | `~/.config/sxhkd/`      | Keyboard shortcuts                                                 |
| `polybar/`          | `~/.config/polybar/`    | Bar: `config.ini`, `modules.ini`, `colors.ini`, `launch.sh`        |
| `rofi/`             | `~/.config/rofi/`       | `config.rasi` + Catppuccin theme                                   |
| `nvim/`             | `~/.config/nvim/`       | Lua config, `lazy.nvim`, LSP + Telescope + Treesitter              |
| `ranger/`           | `~/.config/ranger/`     | Includes vendored `ranger_devicons` plugin                         |
| `lazygit/`          | `~/.config/lazygit/`    | Placeholder (empty `config.yml`)                                   |
| `redshift/`         | `~/.config/redshift/`   | Manual location — edit `lat`/`lon`                                 |
| `zathura/`          | `~/.config/zathura/`    | Dracula theme                                                      |
| `flameshot/`        | `~/.config/flameshot/`  | Screenshot tool                                                    |
| `neofetch/`         | `~/.config/neofetch/`   | `config.conf` + custom `logo`                                      |
| `.tmux.conf`        | `~/.tmux.conf`          | Prefix `Ctrl-\`, TPM plugins                                       |
| `.wezterm.lua`      | `~/.wezterm.lua`        | "coolnight" colors, Nerd Font, Super+drag                          |
| `vscodium/`         | See below              | Settings, snippets, and an installed extension list                |

## VSCodium

On macOS, `~/Library/Application Support/VSCodium/User/settings.json` and
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

On Linux, use `${XDG_CONFIG_HOME:-$HOME/.config}/VSCodium/User` as `user_dir`.

Restore extensions from the repo root with:

```sh
while IFS= read -r extension; do
  codium --install-extension "$extension"
done < vscodium/extensions.txt
```

Refresh the list after installing or removing extensions with
`codium --list-extensions | sort > vscodium/extensions.txt`.
Extension binaries, caches, history, workspace state, and machine-specific
`~/.vscode-oss/argv.json` stay outside the repo.

---

## Key bindings

### Desktop shortcuts (via sxhkd)

| Shortcut                   | Action                                    |
| -------------------------- | ----------------------------------------- |
| `Super + d`                | Launch WezTerm                            |
| `Super + space`            | Rofi app launcher                         |
| `Super + p`                | Power menu                                |
| `Super + w`                | Random wallpaper                          |
| `Super + Escape + r`       | Reload sxhkd                              |

Full list: `sxhkd/sxhkdrc`.

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
- **sxhkd** — `sxhkdrc`, including his keybinding philosophy and the
  `~/bin/*` script hooks.
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
