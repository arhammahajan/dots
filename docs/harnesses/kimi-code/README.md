# Kimi Code

Kimi Code connects to the local `model-router` service through three protocol providers defined in `harnesses/kimi-code/config.toml`. See the [shared setup](../README.md) for router services, launcher usage, and common paths.

## Symlinks and environment

`harnesses/setup.sh` creates three symlinks for Kimi Code.

| Source | Target |
| --- | --- |
| `harnesses/kimi-code/config.toml` | `~/.kimi-code/config.toml` |
| `harnesses/kimi-code/mcp.json` | `~/.kimi-code/mcp.json` |
| `harnesses/shared/skills` | `~/.kimi-code/skills` |

`harnesses/setup.sh` also links `harnesses/shared/model-router.env` to `${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env`. `.zshenv` sources that file and exports `MODEL_ROUTER_API_KEY="unused"` by default. All three provider blocks in `config.toml` read `MODEL_ROUTER_API_KEY` from the environment, while `model-router` holds the real upstream credentials.

`.zshrc` prepends `~/.kimi-code/bin` to `PATH`. Kimi Code saves active config updates as regular files and can replace the `~/.kimi-code/config.toml` symlink, so run `harnesses/setup.sh --check` if local edits drift from the repository copy.

## Providers and models

`harnesses/kimi-code/config.toml` sets `default_model = "model-router/deepseek-v4.1-flash"`, enables `[thinking]`, and registers three providers against `127.0.0.1:4000`.

| Provider | Type | Base URL |
| --- | --- | --- |
| `model-router` | `openai` | `http://127.0.0.1:4000/v1` |
| `model-router-responses` | `openai_responses` | `http://127.0.0.1:4000/v1` |
| `model-router-anthropic` | `anthropic` | `http://127.0.0.1:4000` |

Model entries in `config.toml` are a manual snapshot of the router catalog. When the router catalog changes, update each `[models."..."]` entry from router metadata for `max_context_size`, `max_output_size`, `capabilities`, `support_efforts`, `off_effort`, and `reasoning_key`. Tracked capabilities cover `tool_use`, `thinking`, `image_in`, `video_in`, and `audio_in`. See the [context and effort audit](../model-capabilities.md) for client limit details.

## Skills, MCP, and hooks

`config.toml` lists `~/dev/dots/harnesses/shared/skills` in `extra_skill_dirs` alongside the `~/.kimi-code/skills` symlink.

`harnesses/kimi-code/mcp.json` registers the `agent-browser` MCP server through `${HOME}/.local/bin/agent-browser` with `mcp --tools core,debug --config ${HOME}/.local/share/helium-tools/agent-browser/config.json`. See [shared MCP servers](../browser-mcp.md) for launcher setup.

The `[[hooks]]` blocks in `config.toml` run `spaces hook kimi <Event>` with a 10-second timeout for eight lifecycle events: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `Stop`, and `StopFailure`. See the [Zellij setup](../../zellij.md) for hook listener details.

## Validation

Run `kimi doctor` to check the Kimi Code installation and active configuration.
