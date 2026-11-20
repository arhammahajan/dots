# Kilo Code CLI

Kilo CLI 7.8.18 connects directly to the local `model-router` at `http://127.0.0.1:4000/v1`. Its default model is `model-router/deepseek-v4.1-flash`. Upstream credentials stay inside `model-router`. Kilo reads `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` from `~/.config/ai/model-router.env`, which supplies the loopback URL and the public placeholder key `unused`. No Kilo account is needed for these connections.

## Configuration and model catalog

`harnesses/kilo/kilo.json` is the tracked global configuration, symlinked to `~/.config/kilo/kilo.json`. It sets `bash` permission to `allow`, loads shared skills from `~/dev/dots/harnesses/shared/skills`, and enables the local `agent-browser` MCP server documented in [browser MCP setup](../browser-mcp.md).

The file enables three `model-router` providers and defines 28 models as a static catalog:

| Provider ID | Adapter package | Models | Coverage |
| --- | --- | --- | --- |
| `model-router` | `@ai-sdk/openai-compatible` | 18 | Chat models, `gemini-4-argon`, and `local-llama` |
| `model-router-responses` | `@ai-sdk/openai` | 5 | `grok-4.7`, Muse Spark models, and GPT Luna models |
| `model-router-anthropic` | `@ai-sdk/anthropic` | 5 | `claude-haiku-5-5`, `minimax-m3`, and Qwen 3.7 and 3.8 models |

Each model entry records the router's context and output token limits, tool-calling support, reasoning effort variants, and supported input and output types across text, image, audio, video, and PDF. `local-llama` uses a 64,000-token context window and a 16,384-token output limit. Where the router does not publish effort levels, such as for `kimi-k2.7-code`, `mimo-v2.6-flash`, and `mimo-v2.6-pro`, `kilo.json` disables Kilo's built-in variants with `"disabled": true`.

Because the catalog is static, refresh entries in `harnesses/kilo/kilo.json` from `/v1/models` when router availability or model capabilities change. `kilo.json` does not configure model pricing, so costs that Kilo displays do not represent upstream billing.

## Usage and session affinity

Run Kilo directly in Zsh or pass flags to pick a provider group, model, or effort variant:

```sh
kilo
kilo models
kilo -m model-router/kimi-k3
kilo -m model-router-responses/gpt-6-luna
kilo -m model-router-anthropic/claude-haiku-5-5
kilo run --variant low 'Explain this project'
```

Run `/models` inside an active Kilo session to switch models. Check an installation with `kilo --version`, `kilo config check`, and `kilo models`.

`.zshenv` adds `~/.local/bin` to `PATH` and sources `~/.config/ai/model-router.env`. When you invoke `harness kilo` through the shared Go launcher, `harness` loads `~/.config/ai/model-router.env` and executes `~/.kilo/bin/kilo`.

Kilo sends `X-Session-Id` for each conversation and preserves that header when resuming a session. `model-router` reads `X-Session-Id` for upstream affinity, so Kilo needs no shared install-wide session token or compatibility shim.

## Installation and symlinks

Install the standalone binary with the [official installer](https://kilo.ai/cli/install) without editing shell startup files, then wire the symlinks from the `dots` checkout:

```sh
curl -fsSL https://kilo.ai/cli/install -o /tmp/kilo-install.sh
bash /tmp/kilo-install.sh --no-modify-path
mkdir -p "$HOME/.local/bin"
ln -s "$HOME/.kilo/bin/kilo" "$HOME/.local/bin/kilo"
./harnesses/setup.sh
```

`harnesses/setup.sh` backs up any conflicting targets and creates these symlinks:

| Source in repo | Target path |
| --- | --- |
| `harnesses/kilo/kilo.json` | `${XDG_CONFIG_HOME:-$HOME/.config}/kilo/kilo.json` |
| `harnesses/shared/model-router.env` | `${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env` |

Run `./harnesses/setup.sh --check` to verify the symlinks. Build the shared `harness` CLI from `~/dev/model-router` with `go build ./cmd/harness` and link it at `~/.local/bin/harness` as described in [the main harnesses guide](../README.md). Test it with `harness kilo --help`.

Update Kilo with `kilo upgrade`. The installed 7.8.18 release uses the `provider` schema in `kilo.json`; newer examples on Kilo's website may use a different schema.

## T3 Code ACP provider

`harnesses/kilo/t3-provider.json` stores the T3 Code provider instance `acpRegistry_kilo`, displayed as Kilo Code. It references the official [ACP Registry](https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json) entry with `agentId` set to `"kilo"` and `distribution` set to `"binary"`. The entry overrides `commandPath` to `/home/arhammahajan/.local/bin/harness` and sets `HARNESS_AGENT=kilo` in the provider environment. Because the registry entry supplies `acp`, T3 runs the equivalent of `harness kilo acp` against the same `~/.kilo/bin/kilo` binary used in the terminal.

The `harness` launcher loads `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` from `${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env` and falls back to `http://127.0.0.1:4000` and `unused` if the file is absent. T3 stores only `HARNESS_AGENT=kilo` and does not duplicate router URLs or keys.

In ACP mode, Kilo reads `~/.config/kilo/kilo.json` and reports its default model, catalog, and session options over ACP. T3's initial catalog probe can show the default model's effort choices on other models until you select a model. After you pick a model, Kilo updates the session's effort choices. Setup testing verified ACP initialization with the official binary, the router catalog in the model selector, and a live `session/prompt` request through `model-router`.

To enable Kilo in T3 Code on another machine:

1. Install the standalone Kilo binary, build and link `~/.local/bin/harness`, and run `./harnesses/setup.sh`.
2. Install the Kilo entry from T3's ACP Registry.
3. Merge `harnesses/kilo/t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json` and update `commandPath` to match your home directory. `harnesses/setup.sh` links that file to `harnesses/t3code/settings.json`.
