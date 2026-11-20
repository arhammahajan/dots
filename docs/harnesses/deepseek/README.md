# DeepSeek Harness through model-router

This repository configures the published [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) release `@deepseek-ai/dsh@0.2.1-rc.5` to route all model requests through `model-router`. The install script places the package under `~/.local/share/deepseek-harness` and links `~/.local/bin/dsh` to `~/.local/share/deepseek-harness/bin/dsh`.

DeepSeek Harness requires Node 24.9.0 or newer. This machine runs Node 24.22.1 through fnm. Both `harnesses/deepseek/setup.sh` and the Go `harness` CLI locate fnm's default Node when started outside an interactive shell.

## Running DeepSeek Harness

Run `dsh` directly in interactive Zsh, where `.zshrc` removes any `dsh` alias. Use `harness deepseek` when you want the launcher at `~/.local/bin/harness` to load `~/.config/ai/model-router.env` and locate fnm's Node outside an interactive shell.

`~/.config/ai/model-router.env` exports `MODEL_ROUTER_URL=http://127.0.0.1:4000` and `MODEL_ROUTER_API_KEY=unused`. Only `model-router` holds upstream credentials. The `model-router` provider in the DeepSeek Harness patch files connects to `http://127.0.0.1:4000/v1` and reads `MODEL_ROUTER_API_KEY`.

```sh
dsh                                      # start the Web UI and open a browser
dsh web --no-open                        # print the authenticated local URL
dsh headless "explain this repository"   # run one task in the current directory
dsh acp                                  # run ACP over stdio for compatible clients
```

The Web UI listens on loopback port 3080. Open the URL printed by `dsh`, because that URL carries a generated access token that sets the browser cookie. Unauthenticated requests to the bare URL return HTTP 401. Stop the server with Ctrl-C. DeepSeek Harness requires the `model-router` user service, and requires no extra compatibility proxy or background daemon.

## Profiles, models, and configuration

`harnesses/setup.sh` creates two directory symlinks so edits made through atomic file replacement stay tracked in this repository:

| Repository path | Symlink destination | Contents |
| --- | --- | --- |
| `harnesses/deepseek/profiles` | `~/.dsh/profiles` or `$DSH_HOME/profiles` | Profile manifests, root configs, and router patches |
| `harnesses/deepseek/tui` | `~/.dsh-tui` | Saved TUI language, model, effort, onboarding, and settings |

`harnesses/deepseek/profiles/` contains the `web`, `headless`, `acp`, `sdk`, `sdk-minimal`, and `dsh-tui` profiles. Git tracks `cordis.yml`, `cordis.patch.yml`, `package.json`, `pnpm-workspace.yaml`, and `pnpm-lock.yaml` in each profile directory. Both `profiles/.gitignore` and `tui/.gitignore` exclude `node_modules/` and runtime state. Credentials, sessions, and storage directories under `~/.dsh` stay machine-local.

Top-level template files live in `harnesses/deepseek/`:

- `cordis.patch.yml` holds the `model-router` provider definition, `agent-default-model`, and `mcp-agent-browser`.
- `acp.patch.yml` holds the `acp` fallback selection, which sets `provider` to `model-router` and `model` to `deepseek-v4.1-flash`.

The `web`, `headless`, `acp`, `sdk`, and `sdk-minimal` profiles share the `model-router` provider block and set `agent-default-model` to `gpt-6-luna` with `reasoningEffort: low`. The `acp` profile also appends the `acp` fallback entry that points to `deepseek-v4.1-flash` with no preset effort, so ACP starts at the provider default until a client picks an effort level.

The `dsh-tui` profile loads version `0.20.1` of `@deepseek-harness-tui/dsh-tui` with an empty `cordis.patch.yml`. `harnesses/deepseek/tui/` stores the tracked TUI preferences. `lang.json` and `settings.json` set the language to `en`, `model.json` selects `deepseek-v4.1-flash` on `model-router`, `effort.json` sets effort to `high`, `onboarding.json` marks onboarding complete, and `migrations.json` records completed UI migrations.

In the Web UI, choose another model or effort level from the model picker. Saved defaults belong to that profile, while existing conversations keep their own selection. Reinstalling does not overwrite tracked profile patches.

### Model inventory and protocol behavior

Each patched profile declares 28 models under `providers.model-router`. The list combines the 27 models advertised by `/v1/models` on 2026-11-04 with `local-llama`. The `local-llama` entry uses the display name `Local (llama.cpp)` and configures a 64,000-token context window, 16,384 max output tokens, text input, and the `none`, `minimal`, `low`, `medium`, `high`, and `xhigh` effort levels. Context windows, max output token limits, image input support, and effort lists come from `metadata.router`. Models with unknown effort lists omit `reasoningEfforts` so the upstream uses its default. DeepSeek Harness accepts text and image inputs only. For `gpt-5.6-luna`, `gpt-6-luna`, `hy3`, and `hy4-preview`, the patch shows `off` in the UI and maps it back to `none` on the wire.

All 28 models use `api: openai-responses` over `transport: sse`, with `supportsDeveloperRole: false` and `supportsStrictMode: false`. `model-router` translates Responses requests to Chat, Anthropic, or Gemini upstreams where needed, as it does for Muse. DeepSeek's Responses adapter sends its durable session ID in `session_id` and `prompt_cache_key`, and `model-router` uses that ID for upstream affinity. Sessions do not share an install-wide or process-wide ID. Using Responses ingress also avoids the installed adapter's inability to set native Chat or Anthropic affinity headers for a custom provider. Nonportable Responses controls follow the router's lossy compatibility policy.

The five patched profiles also configure stdio MCP clients for `agent-browser`, `playwright`, and `browser-use` via `@deepseek-ai/dsh-mcp-client`. See [browser-mcp.md](../browser-mcp.md) for the shared browser MCP setup.

Running Fetch available models under Settings and Models in the Web UI adds new model IDs, but does not pull the router's capability metadata. Update `harnesses/deepseek/cordis.patch.yml` and the tracked profile patches when the router inventory changes so personal preferences stay intact. Any new profile needs the same provider patch, because an unpatched shipped profile keeps DeepSeek's official provider defaults.

## Reinstalling

With `model-router` already configured, run:

```sh
~/dev/dots/harnesses/deepseek/setup.sh
```

`harnesses/deepseek/setup.sh` runs four steps:

1. Prepends `${XDG_DATA_HOME:-$HOME/.local/share}/fnm/aliases/default/bin` to `PATH` and checks for Node 24.9.0 or newer.
2. Installs `@deepseek-ai/dsh@0.2.1-rc.5` globally under `~/.local/share/deepseek-harness`, with `--allow-scripts=@deepseek-ai/dsh-subprocess-local,koffi,node-pty,protobufjs` so required native dependencies build without changing the global npm policy.
3. Runs `harnesses/setup.sh` to create or verify the `~/.dsh/profiles` and `~/.dsh-tui` symlinks without overwriting tracked profiles or TUI preferences.
4. Symlinks `~/.local/bin/dsh` to `~/.local/share/deepseek-harness/bin/dsh`, and exits with an error if a different file or symlink already occupies that path.

## T3 Code ACP provider

`harnesses/deepseek/t3-provider.json` defines the `acpRegistry_dsh` provider instance with `displayName` set to `DeepSeek Harness`. It configures a local ACP command that runs `/home/arhammahajan/.local/bin/harness` with `commandArgs: ["deepseek", "acp"]` and `distribution: "auto"`. Because `harness` loads `~/.config/ai/model-router.env` and locates fnm's Node, T3 Code needs no provider environment overrides, ACP registry install, or DeepSeek login.

`harnesses/setup.sh` links `harnesses/t3code/settings.json` to `~/.t3/userdata/settings.json`, and that tracked file does not include `acpRegistry_dsh` by default. To register DeepSeek Harness in T3 Code on a machine, run `harnesses/deepseek/setup.sh`, adjust `commandPath` in `harnesses/deepseek/t3-provider.json` if the home directory differs, and merge that entry into `providerInstances` in `~/.t3/userdata/settings.json`.

When registered, T3 Code discovers all 28 router models and their display names from the ACP session. ACP model IDs encode the provider and model pair as a JSON array, so keep those IDs intact when editing T3 settings programmatically. The agent exposes reasoning choices for the active model through ACP config options. T3's initial catalog can repeat the current model's choices for other models until you switch models and the agent publishes the options for that model.

## Validation

Setup validation checked the installed version, composed profiles, shell-independent launcher, Web UI token and cookie access, live requests across DeepSeek V4.1 Flash, Qwen 3.8 Flash, and Gemini 4 Argon, and a two-request DeepSeek shell tool round trip. ACP validation checked T3's live provider catalog without restarting the app, ACP initialization, exact router inventory comparison, effort selection, a streamed response that returned `DSH_T3_ACP_OK`, and session close. Validation sampled live inference across upstream protocols rather than running against every model.

```sh
dsh --version
dsh web --dump-config
systemctl --user status model-router
```
