# Grok Build CLI

`config.toml` mirrors this computer's Grok Build settings: Grok 4.7 with low
reasoning effort, the official xAI plugin marketplace, and terminal preferences.

To link the config on another machine, back up any existing config first,
then run from the dotfiles checkout:

```sh
grok_dir="${GROK_HOME:-$HOME/.grok}"
mkdir -p "$grok_dir"
ln -s "$PWD/harnesses/grok/config.toml" "$grok_dir/config.toml"
```

Only link `config.toml`. OAuth credentials, sessions, caches, installed plugins,
and CLI binaries stay outside the repo.
