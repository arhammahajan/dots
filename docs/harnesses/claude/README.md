# Claude Code through model-router

`harnesses/claude/settings.json` points Anthropic Messages requests at `http://127.0.0.1:4000` and sets `deepseek-v4.1-flash` as the default model. Its picker replaces Claude Code's built-in options with every configured router model, including `local-llama`. `model-router` translates Claude Code's Anthropic Messages requests for Chat, Responses, Gemini, and local upstreams.

## Model picker and effort controls

Each entry in `modelPicker.options` records the router's context limit and published effort levels in its `description` field. The `CLAUDE_CODE_MODEL_CAPABILITIES` environment variable enables or disables `effort`, `xhigh_effort`, and `max_effort` for each model. Where supported, `modelSettings.<model>.maxEffortLevel` caps the effort selector, and selected models set a starting `effortLevel` of `low`.

Claude Code 2.1.312 does not read router context limits or effort enums for custom models. Picker entries accept no context-window field, so the built-in context meter uses its own fallback value. The picker descriptions state the router's real limits, but they do not change the meter. A global context override would become wrong whenever you switch models, so `settings.json` leaves it unset.

The effort selector cannot represent sparse effort sets such as `high` and `max`, and it cannot expose `none` or `minimal` as ordinary effort choices. Enabling `effort` keeps Claude Code's built-in `low`, `medium`, and `high` options. Full support for per-model context meters and sparse effort lists requires client support for custom-model metadata. See the [context and effort audit](../model-capabilities.md) for how other CLI tools handle these fields.

## Authentication and launching

`settings.json` sets `ANTHROPIC_AUTH_TOKEN` to `unused`, `ANTHROPIC_API_KEY` to an empty string, and `CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS` to `1`. The public `unused` token satisfies the SDK, while only `model-router` loads upstream credentials.

Direct `claude` invocations use the endpoint and placeholder token from `~/.claude/settings.json`. Running `harness claude` loads `~/.config/ai/model-router.env` and exports `ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN` for the Claude process alone.

To run Claude Code inside T3, merge `harnesses/claude/t3-provider.json` into your T3 provider configuration. That template configures the `claudeAgent` driver to launch `~/.local/bin/harness` with `HARNESS_AGENT=claude` in its environment. Preserve existing custom model entries during the merge. T3 maintains its own model picker separately from Claude Code's CLI picker.

## Files, symlinks, and hooks

`harnesses/setup.sh` respects `CLAUDE_CONFIG_DIR` and defaults to `~/.claude` when that variable is unset.

| Repository path | Destination | Installation |
| --- | --- | --- |
| `harnesses/claude/settings.json` | `~/.claude/settings.json` | `harnesses/setup.sh` symlinks this file |
| `harnesses/shared/skills` | `~/.claude/skills` | `harnesses/setup.sh` symlinks this directory |
| `harnesses/claude/mcp.json` | `~/.claude.json` under `mcpServers` | Merge manually or pass via `claude --mcp-config` |
| `harnesses/claude/t3-provider.json` | T3 `claudeAgent` provider settings | Merge manually and keep existing custom models |

Do not replace `~/.claude.json` with `harnesses/claude/mcp.json`, because `~/.claude.json` also stores Claude Code's machine-local state. See [browser MCP setup](../browser-mcp.md) for `agent-browser` details.

`settings.json` defines hooks for 16 Claude Code lifecycle events: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `Notification`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `PostCompact`, `TeammateIdle`, `Elicitation`, `ElicitationResult`, and `SessionEnd`. Each hook runs `spaces hook claude <Event> >/dev/null 2>&1 || true` with a 5-second timeout so the optional `spaces-hooks.service` listener tracks session state.

See the [shared setup](../README.md) for systemd services, shared environment variables, and model compatibility.
