# Grok Build CLI

This directory stores the Grok Build configuration, Spaces hook definitions, and the separate model-router setup for T3.

## Tracked files and symlinks

| Repository file | Destination | Setup method |
| --- | --- | --- |
| `harnesses/grok/config.toml` | `~/.grok/config.toml` | `./harnesses/setup.sh` |
| `harnesses/grok/hooks/zellij.json` | `~/.grok/hooks/zellij.json` | `./harnesses/setup.sh` |
| `harnesses/grok/router-config.toml` | `~/.model-router/grok/model-router/config.toml` | Manual symlink |
| `harnesses/grok/t3-router-provider.json` | `providerInstances` in `~/.t3/userdata/settings.json` | Manual merge |

`./harnesses/setup.sh` backs up existing destination files under `~/.local/share/dots-backups/` and links `config.toml` and `hooks/zellij.json` into `~/.grok`. `./harnesses/setup.sh --check` verifies those symlinks.

To link only the primary config into a custom `GROK_HOME` without running `harnesses/setup.sh`, back up any existing file first and run these commands from the repository root:

```sh
grok_dir="${GROK_HOME:-$HOME/.grok}"
mkdir -p "$grok_dir"
ln -s "$PWD/harnesses/grok/config.toml" "$grok_dir/config.toml"
```

Keep OAuth credentials, sessions, caches, installed plugins, and CLI binaries outside the repository. Only link `config.toml` and `hooks/zellij.json`.

## Primary configuration

`harnesses/grok/config.toml` configures the direct xAI installation with these settings:

- Default model `grok-4.7` with `low` reasoning effort
- Internal CLI installer with `auto_update = true`
- Shared skills loaded from `~/dev/dots/harnesses/shared/skills`
- Official xAI plugin marketplace at `https://github.com/xai-org/plugin-marketplace.git`, with default bundled skill installs purged
- UI settings `max_thoughts_width = 120`, `yolo = false`, and `compact_mode = false`
- MCP server `agent-browser` running `${HOME}/.local/bin/agent-browser mcp --tools core,debug --config ${HOME}/.local/share/helium-tools/agent-browser/config.json` with a 60-second startup timeout and a 120-second tool timeout

See [shared MCP servers](../browser-mcp.md) for Helium CDP setup.

`harnesses/grok/hooks/zellij.json` runs `spaces hook grok <event>` with a 5-second timeout on ten lifecycle events. It triggers on `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `Notification`, `Stop`, `StopCancelled`, `StopFailure`, and `SessionEnd`. See [Zellij setup](../../zellij.md) for the Spaces hook listener.

## Second provider through model-router in T3

T3 can run a second Grok Build provider named "Grok Build (Model Router)" alongside the default "Grok" provider. The default "Grok" entry in `harnesses/t3code/settings.json` keeps using `~/.grok` and its xAI login. The second instance, `grok_router`, launches `~/.local/bin/harness` with `HARNESS_AGENT=grok` and isolates its sessions, credentials, caches, and plugins under `~/.model-router/grok/model-router`.

### Launcher environment and session affinity

Build the Go `harness` CLI from `cmd/harness` in `~/dev/model-router` and link it to `~/.local/bin/harness`. That binary handles `harness grok` and T3 launches with `HARNESS_AGENT=grok`. It reads `~/.config/ai/model-router.env`, which defaults `MODEL_ROUTER_URL` to `http://127.0.0.1:4000` and `MODEL_ROUTER_API_KEY` to `unused`. Before executing `~/.local/bin/grok`, the `harness` binary sets five environment variables:

| Variable | Value |
| --- | --- |
| `GROK_HOME` | `~/.model-router/grok/model-router` |
| `GROK_MODELS_BASE_URL` | `${MODEL_ROUTER_URL}/v1` |
| `GROK_MODELS_LIST_URL` | `${MODEL_ROUTER_URL}/v1/models` |
| `XAI_API_KEY` | `MODEL_ROUTER_API_KEY` |
| `GROK_MODEL_ROUTER_SESSION` | Fresh `router-session-*` identifier generated per launch |

Every model override in `harnesses/grok/router-config.toml` sets `env_http_headers = { "x-session-id" = "GROK_MODEL_ROUTER_SESSION" }`. That header gives model-router a stable session identity for upstream routing affinity, which OpenCode Go requires. Upstream provider credentials stay inside model-router.

### Router model overrides

Grok Build discovers models from the router's `/v1/models` endpoint. `harnesses/grok/router-config.toml` sets `auto_update = false`, `use_leader = false`, and default model `deepseek-v4.1-flash` with `low` reasoning effort. It assigns `deepseek-v4.1-flash` to `session_summary` and `image_description`, and `grok-4.7` to `web_search`. It loads the same shared skills path and `agent-browser` MCP server as `config.toml`.

The file defines 28 `[model."..."]` overrides for the 27 models from the 2026-11-04 router inventory and `local-llama` for `llama.cpp`. Each override records `context_window`, `max_completion_tokens`, `vision`, `api_backend`, and either `reasoning_efforts` or `supports_reasoning_effort = false`. Models route through their native `chat_completions`, `responses`, or `messages` backend, except `gemini-4-argon`, which uses translated `chat_completions`. All 28 entries set `supports_backend_search = false` to disable Grok-hosted backend search on custom endpoints.

Update the model tables in `harnesses/grok/router-config.toml` and the `customModels` array in `harnesses/grok/t3-router-provider.json` together whenever the router inventory changes.

### Installing the router provider

Start the `model-router` user service and install the Go `harness` CLI at `~/.local/bin/harness`. See [shared setup](../README.md) for service and `harness` installation steps. Back up any existing `config.toml` under `~/.model-router/grok/model-router`, then run these commands from the repository root:

```sh
mkdir -p "$HOME/.model-router/grok/model-router"
chmod 700 "$HOME/.model-router/grok/model-router"
ln -s "$PWD/harnesses/grok/router-config.toml" "$HOME/.model-router/grok/model-router/config.toml"
harness grok models
```

Merge `harnesses/grok/t3-router-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json` and update `binaryPath` to match the machine's home directory. The `customModels` list seeds T3's initial model picker snapshot, and ACP discovery supplies context limits and reasoning effort menus at runtime. The `grok_router` provider does not require a Grok login or a copied OAuth token.

Verification ran on Grok Build 1.0.70 against the 27-model inventory from 2026-11-04. ACP discovery loaded all 27 models with their configured context limits and reasoning effort menus. Live ACP requests through DeepSeek V4.1 Flash returned `ROUTER_OK`, executed `read_file` against a fixture file, and generated session summaries through a configured router model.
