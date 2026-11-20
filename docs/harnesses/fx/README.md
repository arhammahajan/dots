# fx through model-router

Run `fx` directly or launch it through `harness fx` to use the local `model-router` custom connection. The tracked configuration sets `deepseek-v4.1-flash` as the default model and registers 28 router models in `/model` and `fx models`. Select a different model with `/model <id>` or `--model <id>`.

`harnesses/setup.sh` links `~/.fx/settings.json` to `harnesses/fx/settings.json`, and the fx fork follows symlinks when writing profile files. Personal workspace entries and runtime preferences stay in that tracked file alongside the router connection, static model metadata, active model selection, and update policy. No Vercel login or upstream credentials are stored in this configuration.

| Repository file | Destination | Purpose |
| --- | --- | --- |
| `harnesses/fx/settings.json` | `~/.fx/settings.json` | Router provider definition, model metadata, and runtime preferences |
| `harnesses/fx/mcp.json` | `~/.fx/mcp.json` | Shared `agent-browser` MCP server configuration |
| `harnesses/shared/skills` | `~/.fx/skills` | Shared agent skills directory |

See [browser MCP setup](../browser-mcp.md) for the `agent-browser` MCP server setup.

## Compatibility and services

fx's [custom connection](https://fx.sh/docs/configure-fx/custom-model-connections) supports OpenAI Chat Completions, bearer environment authentication, and static model metadata. It cannot send custom session headers, and it sends multiple leading system messages. Hosted OpenCode routes require session affinity, and the loaded llama.cpp template accepts only one leading system message.

fx connects directly to `model-router` at `http://127.0.0.1:4000/v1`. Enable `bearer_session_compat = true` under `[server]` in the router's unauthenticated local configuration. With that setting enabled, `model-router` combines consecutive leading plain-text system messages and maps the public bearer session token to the upstream session header. The main router handles streaming responses and model discovery, so no separate proxy service is required.

Run `fx` directly for regular terminal use, or invoke `harness fx` when you want the shared launcher environment. T3 runs `~/.local/bin/harness` with `HARNESS_AGENT=fx`, as recorded in `harnesses/fx/t3-provider.json`. The central `harness` launcher sets `FX_MODEL_ROUTER_SESSION` to a fresh `router-session-<UUID>` value on each invocation and executes `~/.local/libexec/fx/fx-router`. Every turn and permission review within one invocation shares that session ID. Resuming in a later invocation creates a new ID. The session token is a public session identifier, not an upstream credential.

```sh
fx status --json
fx models
fx doctor --json
systemctl --user status model-router
```

## Model metadata

`providers.model-router.model_metadata` in `harnesses/fx/settings.json` stores static metadata for 28 router models, initially captured from `/v1/models` on 2026-10-28. Context windows (`context_window`), tool support (`supports_tool_use`), and image support (`supports_vision`) come from `metadata.router`. fx does not discover these fields from `model-router` at runtime. When the router inventory changes, update `providers.model-router.model_metadata` in `harnesses/fx/settings.json` and keep existing runtime preferences and workspace entries intact.

The `local-llama` entry declares a 32,000-token `context_window` and omits `max_output_tokens` because llama.cpp does not report an output limit. Other models copy their published `max_output_tokens` values directly, except `grok-4.7` and `kimi-k2.7-code`. Their published output limits equal their full context windows, which fx's custom-provider validator rejects. Both entries set `max_output_tokens` to `32768`, which matches fx's bounded request policy for full-window output limits.

## Reasoning effort

fx reads `reasoning_efforts` from each model entry as a JSON object that maps level names to request fields. It builds the native `/model` and `/effort` menus from those keys and merges the selected level's fields into the Chat Completions request body. Each configured level carries `{"reasoning_effort": "<level>"}`, so fx sends `reasoning_effort` in the request. `model-router` receives that field and translates it for the upstream protocol. Selecting the default option omits `reasoning_effort` and lets the destination apply its own default. A model with `"reasoning_efforts": {}` exposes no effort levels.

In `harnesses/fx/settings.json`, 22 of the 28 models define exact effort levels published by `model-router`. The remaining 6 models (`kimi-k2.7-code`, `mimo-v2.6-flash`, `mimo-v2.6-pro`, `minimax-m3`, `longcat-2.0`, and `qwen3.7-plus`) set `"reasoning_efforts": {}` and keep their destination defaults because the router has no verified effort enum for them. Sparse lists remain sparse. For example, `local-llama` offers `none`, `minimal`, `low`, `medium`, `high`, and `xhigh`, while `deepseek-v4.1-flash` offers `low`, `high`, and `max`.

Restart any running fx process to load a rebuilt binary or updated metadata. Change effort interactively with `/effort` or the effort step in `/model`, or override it for a single request:

```sh
FX_MODEL=local-llama fx ask --effort none "Reply with OK"
```

## Building the fx fork

The client implementation lives in [arhammahajan/fx on branch `main`](https://github.com/arhammahajan/fx/tree/main), checked out at `~/dev/fx` by default. `harnesses/fx/` in this repository holds only the router integration files.

`harnesses/fx/build-fx.py` builds and installs `~/.local/libexec/fx/fx-router`. If the checkout directory does not exist, the script clones branch `main`. For an existing checkout, the script verifies that the checked-out branch is `main` and builds the current working tree without pulling or switching branches. Set `FX_SOURCE_DIR` to build from another checkout path.

The build script uses the Zig 0.17.0 Linux x86_64 compiler cached under `${XDG_DATA_HOME:-~/.local/share}/fx-router/toolchain`. If the compiler is absent, the script downloads it and verifies its SHA-256 checksum before extracting it. The script runs `zig fmt --check` on the modified provider and chat completions files, compiles with `zig build -Doptimize=ReleaseSafe -j4`, and atomically replaces `~/.local/libexec/fx/fx-router`. Build artifacts remain in the fork checkout.

```sh
python3 ~/dev/dots/harnesses/fx/build-fx.py
```

`auto_upgrade` is set to `false` in `harnesses/fx/settings.json`. Pull updates in the fork checkout and rerun `build-fx.py` to upgrade fx.

## Installation

Run `./harnesses/setup.sh` from the dots checkout to symlink `settings.json`, `mcp.json`, and the shared skills directory into `~/.fx/`. The script backs up any existing destination paths before creating links.

For a manual setup on another machine where `model-router` is already configured, back up any existing `~/.fx/settings.json` first, then build the binary and create the links:

```sh
mkdir -p ~/.fx ~/.local/libexec/fx ~/.config/systemd/user
python3 ~/dev/dots/harnesses/fx/build-fx.py
ln -s ~/dev/dots/harnesses/fx/settings.json ~/.fx/settings.json
ln -s ~/dev/dots/harnesses/fx/mcp.json ~/.fx/mcp.json
```

Install the shared `harness` CLI as described in [the main harnesses guide](../README.md), then verify the connection:

```sh
harness fx status --json
```

## Verification

Router compatibility tests live in `~/dev/model-router/internal/server/compat_seam_test.go` and `~/dev/model-router/internal/server/bearer_compat_review_test.go`. Native client tests live in the fx fork at `~/dev/fx`.

```sh
fx status --json
fx models
fx doctor --json
```
