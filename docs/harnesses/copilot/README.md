# GitHub Copilot CLI

The `harnesses/copilot/` directory stores tracked configuration for GitHub Copilot CLI. It covers terminal preferences, shared skill discovery, browser MCP access, and Spaces lifecycle hooks.

## Tracked files

`harnesses/setup.sh` resolves `$COPILOT_HOME` and defaults to `~/.copilot` when that variable is unset. It symlinks three files from this repository into that directory:

| Repository path | Symlink destination | Purpose |
| --- | --- | --- |
| `harnesses/copilot/settings.json` | `$COPILOT_HOME/settings.json` | Terminal preferences and shared skill directories |
| `harnesses/copilot/mcp-config.json` | `$COPILOT_HOME/mcp-config.json` | Local `agent-browser` MCP server entry |
| `harnesses/copilot/hooks/zellij.json` | `$COPILOT_HOME/hooks/zellij.json` | Spaces session and tool lifecycle hooks |

## Settings and shared skills

`harnesses/copilot/settings.json` sets these terminal preferences:

- `theme` is `"github"`.
- `banner` is `"once"`, so Copilot CLI shows the startup banner a single time.
- `renderMarkdown` is `true`.
- `stream` is `true`.
- `skillDirectories` points at `~/dev/dots/harnesses/shared/skills`, so Copilot CLI loads the shared skills catalog directly without a separate skills symlink in `$HOME`.
- `hooks` is an empty object because lifecycle hooks live in `hooks/zellij.json`.

Running `copilot` directly keeps Copilot CLI's default provider and model selection. Change the active model inside a session with `/model`. Running `harness copilot` routes requests through the local model-router over Responses and defaults to `gpt-6-luna`. See [the agent CLI overview](../README.md) for model-router details.

Preference edits made through `/settings` or `copilot config` write through the symlink into `harnesses/copilot/settings.json` in this repository.

## Browser MCP server

`harnesses/copilot/mcp-config.json` defines one local MCP server named `agent-browser` with `"tools": ["*"]`. It launches `${HOME}/.local/bin/agent-browser` with the arguments `mcp --tools core,debug --config ${HOME}/.local/share/helium-tools/agent-browser/config.json`.

`harnesses/setup.sh` links this file directly to `$COPILOT_HOME/mcp-config.json`. If you already keep other MCP servers in `$COPILOT_HOME/mcp-config.json`, merge the `agent-browser` entry into your existing file instead of replacing it. See [browser MCP setup](../browser-mcp.md) for Helium remote debugging and launcher installation steps.

## Spaces lifecycle hooks

`harnesses/copilot/hooks/zellij.json` registers observational command hooks with a 5-second `timeoutSec`. Each command redirects output to `/dev/null` and ends with `|| true` so hook failures never block Copilot CLI.

- `sessionStart` runs `spaces session-hook copilot start`.
- `userPromptSubmitted` runs `spaces session-hook copilot prompt`.
- `sessionEnd`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `permissionRequest`, `notification`, `subagentStart`, `subagentStop`, `preCompact`, `stop`, and `errorOccurred` each run `spaces hook copilot <event>`.

`harnesses/setup.sh` links `spaces-hooks.service`. To keep the hook listener active, enable that user service:

```sh
systemctl --user daemon-reload
systemctl --user enable --now spaces-hooks.service
```

See [the Zellij documentation](../../zellij.md) for how these hooks report pane status and record session IDs for resume.

## Setup and authentication

Run the setup script from the repository root to create or verify all links:

```sh
./harnesses/setup.sh
./harnesses/setup.sh --check
```

`harnesses/setup.sh` moves any existing file at a destination path into `~/.local/share/dots-backups/` before creating the symlink.

To link only `settings.json` manually on another machine, back up any existing file first, then run these commands from the repository root:

```sh
copilot_dir="${COPILOT_HOME:-$HOME/.copilot}"
mkdir -p "$copilot_dir"
ln -s "$PWD/harnesses/copilot/settings.json" "$copilot_dir/settings.json"
```

After linking, start `copilot` and run `/login` to authenticate.

## Files kept outside Git

Link only `settings.json`, `mcp-config.json`, and `hooks/zellij.json`. Do not link the entire `$COPILOT_HOME` directory.

Authentication tokens, custom provider credentials, trusted folders, saved permissions, installed plugins, session history, and logs stay outside Git. `config.json` also stays outside Git because Copilot CLI manages it as local runtime state and reads user preferences from `settings.json`.

See the [Copilot CLI configuration reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference) for available settings and directory contents.
