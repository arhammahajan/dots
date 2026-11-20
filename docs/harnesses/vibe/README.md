# Mistral Vibe through model-router

Install Mistral Vibe with `uv tool install mistral-vibe`, and upgrade it with `uv tool upgrade mistral-vibe`. Run `./harnesses/setup.sh` to symlink `harnesses/vibe/config.toml` to `~/.vibe/config.toml`.

Interactive shells run `vibe` without the launcher because `.zshrc` clears any `vibe` alias. To launch Vibe with a per-invocation router session token, run `harness vibe` with the Go `harness` CLI at `~/.local/bin/harness`. That launcher executes the uv-managed `vibe` binary and sets `VIBE_ROUTER_SESSION` to a fresh public bearer token for `model-router`'s `bearer_session_compat` mode. Upstream API keys stay in `model-router` rather than in Vibe's config.

Start `model-router` on `127.0.0.1:4000` before launching Vibe. See the [shared setup](../README.md) for router service commands, `harness` CLI installation, and shared client settings.

## Configuration and model catalog

`harnesses/vibe/config.toml` defines one `model-router` provider at `http://127.0.0.1:4000/v1` with `api_style = "openai"` and `api_key_env_var = "VIBE_ROUTER_SESSION"`. Every router model in the file uses this OpenAI-compatible Chat provider, even when the upstream provider uses a different protocol. A resumed session receives a new router affinity token because the launcher creates a token per process.

The file sets these client defaults:

| Setting | Value |
| --- | --- |
| `active_model` | `deepseek-v4.1-flash` |
| `enable_telemetry` | `false` |
| `enable_auto_update` | `false` |
| `skill_paths` | `~/dev/dots/harnesses/shared/skills` |
| `[[mcp_servers]]` | `agent-browser` stdio server |

See [browser MCP setup](../browser-mcp.md) for the `agent-browser` server dependencies and options.

Switch models inside Vibe with `/model`, or pass `--model` on the command line:

```sh
vibe --model MODEL --prompt '...'
```

The 28 `[[models]]` entries in `harnesses/vibe/config.toml` are a static snapshot of the router inventory. Update the model entries and capability fields when the router inventory changes, then restart Vibe. Vibe also includes its own built-in Mistral model, which uses Mistral credentials rather than `model-router`.

Vibe may rewrite `~/.vibe/config.toml` when you change settings in the app. Copy any intended edits back into `harnesses/vibe/config.toml` and rerun `./harnesses/setup.sh` if Vibe replaces the symlink with a regular file.

## Context limits and reasoning

Each `[[models]]` entry records `max_context_length`, `max_output_tokens`, and `supports_images` from the router's published metadata.

Twenty-two of the 28 entries define `thinking_levels` with exact effort values. The entries `gpt-5.6-luna`, `gpt-6-luna`, `hy3`, and `hy4-preview` use `"off"` in `thinking_levels` for the router's `none` effort, and Vibe sends `"off"` as `none` on Chat requests. The `local-llama` entry lists `"none"` in `thinking_levels`. When the router publishes a separate thinking toggle without a `none` effort level, this OpenAI-compatible Chat provider does not represent that toggle.

Six entries omit `thinking_levels` because their upstream effort enums are unknown. Those models are `kimi-k2.7-code`, `mimo-v2.6-flash`, `mimo-v2.6-pro`, `minimax-m3`, `longcat-2.0`, and `qwen3.7-plus`. Vibe may display its own generic effort choices for those models, which are not verified capabilities.

The router does not publish default effort values. Three entries in `config.toml` set `thinking = "low"`. Those models are `deepseek-v4.1-flash`, `gemini-4-argon`, and `local-llama`. For all other models, Vibe picks its own default unless you set `thinking` in `config.toml`. Restart Vibe after updating the catalog.

## T3 Code ACP

`harnesses/vibe/t3-provider.json` stores the T3 Code provider template `acpRegistry_mistral_vibe` with the display name Mistral Vibe. It configures the `mistral-vibe` ACP Registry entry, points `commandPath` to the absolute path of `~/.local/bin/harness`, and sets `HARNESS_AGENT=vibe-acp` in `environment`.

When T3 Code starts this provider, `harness` executes Vibe's uv-managed `vibe-acp` binary with a fresh public router affinity token. The ACP process reads `~/.vibe/config.toml`, just like the CLI. Each ACP process receives a fresh affinity token, even when T3 Code resumes a session in a new process.

ACP reports the configured models and thinking choices, and changing the model inside a session updates the available thinking options. T3 Code's initial provider probe may show the default model's thinking choices for other models until you switch models in a session. Vibe also advertises its built-in Mistral model over ACP, which uses Mistral credentials rather than `model-router`.

To enable Mistral Vibe in T3 Code or set it up on another machine, install the `harness` CLI and Mistral Vibe as above. Then merge `harnesses/vibe/t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json` and update `commandPath` to the absolute path of `~/.local/bin/harness` on that machine.
