# File icons for ranger

The `ranger_devicons` plugin adds a ranger linemode that prefixes file and directory names with Nerd Font glyphs. This repository vendors the plugin under `ranger/plugins/ranger_devicons/`. The upstream project is [alexanderjeurissen/ranger_devicons](https://github.com/alexanderjeurissen/ranger_devicons).

## Prerequisites

The plugin renders icons with glyphs from a patched Nerd Font. Install a Nerd Font and set it as your terminal font so the glyphs display instead of missing-character boxes. You can download Source Code Pro and other patched fonts with install instructions from [ryanoasis/nerd-fonts](https://github.com/ryanoasis/nerd-fonts).

## Repository layout

When you link or copy `ranger/` to `~/.config/ranger/`, ranger loads the vendored plugin and enables the `devicons` linemode through `ranger/rc.conf`.

| Path | Purpose |
| --- | --- |
| `ranger/rc.conf` | Sets `default_linemode devicons`. |
| `ranger/plugins/ranger_devicons/__init__.py` | Registers the `devicons` and `filename` linemodes. |
| `ranger/plugins/ranger_devicons/devicons.py` | Maps directory names, file names, and file extensions to Nerd Font glyphs. |
| `ranger/plugins/ranger_devicons/LICENSE` | GNU General Public License v3 for the plugin. |
| `ranger/plugins/ranger_devicons/LICENSE_NERDFONT` | MIT License for the glyph mappings from `vim-devicons`. |

## Standalone installation

To install `ranger_devicons` outside this dotfiles repository, clone the upstream repository into your ranger plugins directory:

```bash
git clone https://github.com/alexanderjeurissen/ranger_devicons ~/.config/ranger/plugins/ranger_devicons
```

Enable the linemode in `~/.config/ranger/rc.conf` or in your custom `rc.conf` path:

```bash
echo "default_linemode devicons" >> "$HOME/.config/ranger/rc.conf"
```

## How the plugin maps icons

`ranger/plugins/ranger_devicons/__init__.py` registers two linemodes with `@ranger.api.register_linemode`. `DevIconsLinemode` registers the `devicons` linemode with `uses_metadata = False`, and `DevIconsLinemodeFile` overrides the `filename` linemode. Both linemodes call `devicon(file)` and prepend the returned glyph and a space to `file.relative_path`.

`ranger/plugins/ranger_devicons/devicons.py` defines `devicon(file)` with three lookup tables from [ryanoasis/vim-devicons](https://github.com/ryanoasis/vim-devicons):

1. For directories, `devicon(file)` looks up `file.relative_path` in `dir_node_exact_matches` and falls back to the default folder glyph.
2. For files, `devicon(file)` looks up `os.path.basename(file.relative_path)` in `file_node_exact_matches`.
3. If no exact file name matches, `devicon(file)` looks up `file.extension` in `file_node_extensions` and falls back to the default file glyph.
