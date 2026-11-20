# MCode with model-router

MCode connects to the local `model-router` service at `http://127.0.0.1:4000/v1` through three custom provider groups in `harnesses/mcode/config.yaml`. The default model is `custom_provider:model-router/deepseek-v4.1-flash` with a `1000000` token context window and `low` reasoning effort.

## Files and symlinks

`harnesses/setup.sh` links the tracked MCode files and the shared skills folder into `~/.minimax/`.

| Source in repo | Target path | Purpose |
| --- | --- | --- |
| `harnesses/mcode/config.yaml` | `~/.minimax/config.yaml` | Provider definitions, model metadata, and default settings |
| `harnesses/mcode/mcp.json` | `~/.minimax/mcp.json` | Helium `agent-browser` MCP server configuration |
| `harnesses/shared/skills` | `~/.minimax/skills` | Shared skills catalog |

`harnesses/mcode/config.yaml` uses JSON syntax, which MCode's YAML reader accepts. When MCode saves settings, it can replace the `~/.minimax/config.yaml` symlink with a regular file. Copy any changes you want to keep back into `harnesses/mcode/config.yaml`, then run `harnesses/setup.sh` to restore the symlink. Run `harnesses/setup.sh --check` to check whether the symlinks remain intact.

For `agent-browser` launcher requirements, see [browser MCP setup](../browser-mcp.md). For `model-router` service setup and shared launcher details, see the [shared setup](../README.md).

## Custom provider groups

All three custom provider groups target `http://127.0.0.1:4000/v1` with `authMode` set to `api-key` and `apiKey` set to `unused`. The built-in `provider.minimax` block keeps the placeholder `sk-xxx`, so the tracked config contains no upstream API keys.

| Group key | Display name | API protocol | Configured models |
| --- | --- | --- | --- |
| `model-router` | `Model Router (Chat)` | `openai-completions` | 18 models such as `deepseek-v4.1-flash`, `kimi-k3.1-pro`, `gemini-4-argon`, and `local-llama` |
| `model-router-responses` | `Model Router (Responses)` | `openai-responses` | `grok-4.7`, `muse-spark-1.2-contributor`, `muse-spark-1.3-contributor`, `gpt-5.6-luna`, `gpt-6-luna` |
| `model-router-anthropic` | `Model Router (Anthropic)` | `anthropic-messages` | `claude-haiku-5-5`, `minimax-m3`, `qwen3.8-max`, `qwen3.8-flash`, `qwen3.7-plus` |

## Sessions and model capabilities

MCode sends its durable conversation ID in the `X-Mavis-Session-Id` request header. `model-router` maps that header to the upstream provider's session header for session affinity.

Context limits, output limits, `capabilities.support_image` flags, and `thinking.effortOptions` reflect the metadata reported by `model-router`. MCode's `--effort` flag selects from the levels configured in `thinking.effortOptions` for the chosen model. Models with `thinking_config.mode` set to `switchable` allow turning thinking off, while models set to `forced_on` keep thinking enabled. See the [context and effort audit](../model-capabilities.md) for how these settings compare across clients.

## Updating model entries

Edit model entries in `harnesses/mcode/config.yaml` from current `model-router` metadata, and keep each model's `capabilities` and `thinking_config` fields intact. Restart MCode after editing the file so it reloads the catalog. If MCode replaced the `~/.minimax/config.yaml` symlink with a regular file, rerun `harnesses/setup.sh` so the updated tracked file takes effect.

Inspect the parsed provider configuration with:

```sh
mcode provider list --json
```
