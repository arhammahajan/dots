# GitHub Copilot CLI

`settings.json` contains portable terminal preferences: the GitHub theme, a
one-time startup banner, Markdown rendering, and streaming responses. Model
selection uses Copilot's default; change it with `/model` inside the CLI.

To link the config on another machine, back up any existing settings first,
then run from the dotfiles checkout:

```sh
copilot_dir="${COPILOT_HOME:-$HOME/.copilot}"
mkdir -p "$copilot_dir"
ln -s "$PWD/harnesses/copilot/settings.json" "$copilot_dir/settings.json"
```

Run `copilot` and use `/login` to authenticate if needed. Preferences changed
through `/settings` or `copilot config` update the linked file in this repo.

Only link `settings.json`, not the entire Copilot directory. Authentication,
trusted folders, saved permissions, installed plugins, session history, and logs
stay outside Git. In particular, `config.json` is automatically managed state;
current Copilot CLI versions store user preferences in `settings.json` instead.
MCP server definitions and custom provider credentials are configured separately.

See the [Copilot CLI configuration reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference)
for available settings and directory contents.
