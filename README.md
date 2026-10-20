# dotfiles

Personal Linux configuration files — an **Arch Linux + bspwm** window-manager setup
with a Neovim/Tmux/WezTerm development environment.

```
WM        bspwm          Bar        polybar         Compositor  picom
Keys      sxhkd          Launcher   rofi            Notifs      dunst
Terminal  wezterm        Multiplex  tmux            Editor      Neovim (lazy.nvim)
Files     ranger         PDF        zathura
```

---

## Layout

| Path in repo        | Installs to             | Notes                                                              |
| ------------------- | ----------------------- | ------------------------------------------------------------------ |
| `alacritty/`        | `~/.config/alacritty/`  | Terminal (gruvbox, 0.7 opacity + blur)                             |
| `bspwm/`            | `~/.config/bspwm/`      | `bspwmrc`, `dunstrc`, `picom_configurations/`, `scripts/launch.sh` |
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

---

## Key bindings

### bspwm (via sxhkd)

| Shortcut                   | Action                                    |
| -------------------------- | ----------------------------------------- |
| `Super + d`                | Launch WezTerm                            |
| `Super + space`            | Rofi app launcher                         |
| `Super + Return`           | Toggle floating / tiled                   |
| `Super + {h,j,k,l}`        | Focus window (`+ Shift` to swap)          |
| `Super + {1-9,0}`          | Switch desktop (`+ Shift` to move window) |
| `Super + {t, Ctrl+t, f}`   | Tiled / pseudo-tiled / fullscreen         |
| `Super + Ctrl + {h,j,k,l}` | Resize window                             |
| `Super + g`                | Swap with biggest window                  |
| `Super + Ctrl + {1-9}`     | Preselect split ratio                     |
| `Super + c`                | Close window                              |
| `Super + p`                | Power menu                                |
| `Super + w`                | Random wallpaper                          |
| `Super + Escape + r`       | Reload sxhkd                              |
| `Ctrl + Shift + {q,r}`     | Quit / restart bspwm                      |

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
- **WezTerm** — `.wezterm.lua`, including the "coolnight" palette that also
  appears in his Alacritty themes.
- **tmux** — `.tmux.conf` derives from his (sensible defaults, TPM,
  `vim-tmux-navigator`, `tmux-resurrect` / `tmux-continuum`, `tmux-tokyo-night`),
  with a custom `Ctrl-\` prefix and extra keybindings.
- **Alacritty** — trimmed derivative of his terminal config.

### [Zproger/bspwm-dotfiles](https://github.com/Zproger/bspwm-dotfiles) — ZProger

The window-manager "rice" is based on ZProger's Arch + bspwm build:

- **bspwm** — `bspwmrc` (borders, gaps, pointer actions, rules, autostart) and
  the `picom_configurations/`.
- **polybar** — `config.ini`, `modules.ini`, `colors.ini`, `launch.sh`.
- **sxhkd** — `sxhkdrc`, including his keybinding philosophy and the
  `~/bin/*` script hooks.
- **rofi** — `config.rasi` and the bundled Catppuccin theme.
- **dunst** — `dunstrc`.
- **ranger, redshift, zathura, flameshot** — configs carried over and adapted
  from his `config/`; the Alacritty window opacity/blur settings also come from
  his terminal config.

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
- Dracula — zathura and picom color palettes.

---

## License

No blanket license is applied to this repository. Configuration adapted from the
projects above remains subject to **their** licenses; check each upstream repo
before redistributing. Personal scripts here are shared as-is,
with no warranty.
