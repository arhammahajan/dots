# Antigravity CLI through model-router

Run `harness antigravity` or `harness agy` to start the Antigravity CLI against the local `model-router`. Build the `harness` binary from `~/dev/model-router` with `go build ./cmd/harness` and link it at `~/.local/bin/harness`.

Interactive Zsh defines an `agy` function in `.zshrc` for native Google sign-in. That function unsets `GEMINI_API_KEY`, `GOOGLE_API_KEY`, and `GOOGLE_GEMINI_BASE_URL`, then runs `~/.local/libexec/antigravity/agy` with `--settings "${XDG_CONFIG_HOME:-$HOME/.config}/ai/antigravity/settings.json"`. Use `harness antigravity` whenever you want requests routed through `model-router`.

Pass `--model <router-model-id>` to choose a model. When you omit `--model`, `harness` reads `model` from `~/.gemini/antigravity-cli/settings.json`, falls back to `gemini-4-argon` if the field is missing, and passes `--model <default>` to the binary because this CLI does not always apply custom defaults from settings alone. Explicit `--model` flags take precedence. When you omit `--settings`, `harness` also passes `--settings '{"modelProvider":"gemini"}'`. All 28 tracked router models in `harnesses/antigravity/cli/settings.json` appear in `/model`.

## CLI configuration and router connection

The CLI uses API-key mode with `modelProvider` set to `gemini`, `GEMINI_API_KEY` set to a per-launch `router-session-<UUIDv4>` token, and `GOOGLE_GEMINI_BASE_URL` pointed at `model-router`. See the official [Antigravity CLI settings reference](https://antigravity.google/docs/settings?tab=cli). No upstream credentials live in any Antigravity config file.

Each custom model in `harnesses/antigravity/cli/settings.json` sets `modelName` to `gemini-api://<model-id>` and relies on `GOOGLE_GEMINI_BASE_URL` for the endpoint. This CLI version sent the longer endpoint-bearing URI format from upstream docs as a literal model ID. Each entry sets `toolFormatterType` to `TOOL_FORMATTER_TYPE_NONE` to use native function calling.

`harness antigravity` loads the router URL from `~/.config/ai/model-router.env`, defaults to `http://127.0.0.1:4000`, sets `GOOGLE_GEMINI_BASE_URL`, and generates a fresh public session token in `GEMINI_API_KEY` on each launch. It unsets `GOOGLE_API_KEY`, `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_CLOUD_PROJECT`, `ANTIGRAVITY_APP_DATA_DIR`, and `ANTIGRAVITY_CONFIG_REL_DIR`.

Enable `[server] bearer_session_compat = true` on `model-router` with loopback client authentication disabled. `model-router` reads the public token as session identity, sets the upstream session header, and authenticates upstream with its own credentials from `~/.config/ai/credentials.env`. Resuming a conversation in a later invocation gets a new affinity ID. No local HTTP proxy process runs alongside the CLI.

To keep router sessions separate from desktop keyrings and Google accounts, `harness antigravity` isolates state under `~/.model-router/antigravity-router-cli`. It sets `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `XDG_CACHE_HOME`, `ANTIGRAVITY_GEMINI_DIR`, `GEMINI_HOME`, `DBUS_SESSION_BUS_ADDRESS=disabled:`, and a scoped `ANTIGRAVITY_CREDENTIAL_SCOPE`. It also links shared skills into `~/.model-router/antigravity-router-cli/.gemini/config/skills`.

The standalone desktop app, the interactive `agy` function, and OAuth-based ACP connections keep their own authentication settings.

## Tracked files and machine setup

Run `./harnesses/setup.sh` from the repository checkout to symlink the tracked Antigravity configs and shared skills. Before replacing an existing file or symlink, `setup.sh` moves it to `~/.local/share/dots-backups/harnesses-<timestamp>-<pid>/`.

| Repo file | Destination | Purpose |
| --- | --- | --- |
| `harnesses/antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` | CLI settings, `agent-browser` MCP entry, and 28 custom router models |
| `harnesses/antigravity/mcp_config.json` | `~/.gemini/config/mcp_config.json` | Antigravity IDE `agent-browser` MCP config |
| `harnesses/antigravity/acp/settings.json` | `~/.gemini/antigravity-acp/settings.json` | OAuth ACP settings with `agent-browser` MCP entry |
| `harnesses/shared/skills` | `~/.gemini/config/skills` | Shared agent skills directory |

Keep the Antigravity CLI binary at `~/.local/libexec/antigravity/agy`, which `harness antigravity` executes directly. If an installer places a new binary at `~/.local/bin/agy`, move it to `~/.local/libexec/antigravity/agy`.

If you copy `harnesses/antigravity/cli/settings.json` onto another machine instead of symlinking it, preserve that machine's `trustedWorkspaces` list. See [browser MCP setup](../browser-mcp.md) for the shared `agent-browser` server configuration.

## T3 Code ACP connection

`harnesses/antigravity/acp/t3-provider.json` defines provider instance `acpRegistry_antigravity_router` with display name "Antigravity (Router ACP)". It runs `~/.local/bin/harness` with `commandArgs: ["antigravity-acp"]`. Existing account-based Antigravity instances in `~/.t3/userdata/settings.json` stay available alongside it.

At startup, `harness antigravity-acp` locates `agy_acp_server.par` under `~/.model-router/tools/antigravity-acp/linux-x64`. It reads `releaseId` from `active.json` first and falls back to the newest executable under `versions/*/agy_acp_server.par`.

`harness antigravity-acp` enables gateway auth and configures the router connection through these environment variables:

- `AGY_ACP_ENABLE_GATEWAY_AUTH=1`
- `AGY_ACP_DEFAULT_MODEL` set to `gemini-4-argon` unless already present in the environment
- `AGY_LLM_GATEWAY_URL` set to the `model-router` base URL
- `AGY_LLM_GATEWAY_API_KEY` set to a fresh per-launch public session token
- `AGY_LLM_GATEWAY_MODELS` populated from `<router>/v1/models` with the default model first, or the default model alone if the fetch fails
- `AGY_ACP_FORCE_FILE_STORAGE=1` with shadow home `~/.model-router/antigravity-router-acp`, `DBUS_SESSION_BUS_ADDRESS=disabled:`, and a scoped `ANTIGRAVITY_CREDENTIAL_SCOPE`

Setting `AGY_LLM_GATEWAY_MODELS` to the fetched router inventory excludes built-in Gemini entries from this ACP connection's model picker.

With `bearer_session_compat` enabled, `model-router` consumes the ACP adapter's nonstandard `sessionId` body field, hashes its JSON-encoded string into an `agy-acp-<SHA-256>` identity, and strips the field before translation or upstream forwarding. Native session headers take priority. Requests without a conversation ID fall back to the per-launch token. No forwarding server or Python supervisor runs alongside the adapter.

The installed ACP adapter reuses one gateway `sessionId` across multiple ACP conversations in the same process. `model-router` uses whatever identity the adapter sends.

To set up this provider on another machine that has the ACP adapter under `~/.model-router/tools/antigravity-acp/linux-x64`, install `~/.local/bin/harness`, run `./harnesses/setup.sh`, and merge `harnesses/antigravity/acp/t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json`. Update `commandPath` to match that machine's home directory.

This ACP adapter is separate from the CLI and does not inherit `customModelsConfig`, context limits, or effort settings from `cli/settings.json`. Custom-model context and effort reporting in the ACP adapter remain limited.

## Context and reasoning

All 28 custom models in `harnesses/antigravity/cli/settings.json` set `maxTokens` from `metadata.router.limits.context_window.value` in `model-router`. Update `local-llama` when its `llama.cpp` server context allocation changes. The CLI schema defines `maxTokens` as total context including output tokens and exposes no separate output limit field for custom models.

`supportsThinking` follows the router's explicit reasoning flag and is `true` for all 28 tracked entries. `supportsImages` follows the router's explicit input-image flag, with 20 models set to `true` and 8 models set to `false` in the current tracked `cli/settings.json`. `supportsToolCalls` is `true` for all 28 models, while video, PDF, and audio flags are `false`.

This CLI does not support per-model effort enums for custom models. The settings schema has no effort-list or default-effort field. Running `harness antigravity --model deepseek-v4.1-flash -p /effort --output-format json` returns `adjustable: false`, and adding `--effort max` fails as unsupported. Setting `supportsThinking` to `true` does not unlock those controls. Reasoning uses the upstream destination's default behavior without a forced token budget or effort mapping.

See the [router capability reference](model-capabilities.md) for each model's router context window and effort vocabulary. Native per-model effort controls require Antigravity support for custom-model effort variants.
