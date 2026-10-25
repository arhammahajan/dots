# Claude Code

`settings.json` contains the OpenCode Go endpoint, model mappings, and CLI model
menu. Link it to `~/.claude/settings.json`; credentials stay outside this repo.

The shared private file `~/.config/ai/credentials.env` contains:

```sh
OPENCODE_API_KEY='your-key'
ANTHROPIC_AUTH_TOKEN="${OPENCODE_API_KEY}"
```

Use your existing OpenCode key. Set permissions with
`chmod 700 ~/.config/ai` and `chmod 600 ~/.config/ai/credentials.env`.
The repo's `.zshenv` already sources this file with automatic exporting, so
new Zsh sessions pass the token to Claude Code. Do not put the token back in
`settings.json`: that would override the environment.

To link the config on another machine, back up any existing settings first,
then run from the dotfiles checkout:

```sh
mkdir -p ~/.claude
ln -s "$PWD/harnesses/claude/settings.json" ~/.claude/settings.json
```

For T3 Code, set the Claude provider's **Binary path** to the absolute path of
`harnesses/claude/claude-with-env` in this checkout. It loads the same credentials
file before executing `claude`, including when T3 starts from the desktop.
The installed Claude binary must be on `PATH`. The launcher forwards every
argument, so the CLI and Claude Agent SDK can both use it.

T3's custom model names are configured separately in T3's provider settings;
the CLI's `modelPicker` does not populate T3's selector.

After editing credentials, open a new shell (or source `~/.zshenv`). For T3,
start a new Claude session so the launcher reloads the file.
