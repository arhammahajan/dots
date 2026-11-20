# Coding agents through model-router

Every agent configuration in `harnesses/` includes the shared Helium `agent-browser` MCP server. See [browser MCP setup](browser-mcp.md) for config destinations, launcher dependencies, and interaction visibility.

Claude Code, Cline CLI, OpenCode, Pi, Qwen Code, Kimi Code, legacy Kimi CLI, MCode, Muse, Antigravity CLI, Gemini CLI, fx, DeepSeek Harness, Mistral Vibe, and Kilo Code connect directly to the local model-router at `http://127.0.0.1:4000`. Second T3 Code instances for Codex and Grok Build also use the router, and `harness copilot` routes GitHub Copilot CLI through it. The primary Codex, Grok Build, and Copilot connections keep their upstream accounts. The router checkout lives at `~/dev/model-router`.

Only model-router reads upstream credentials from `~/.config/ai/credentials.env`. Clients send the public placeholder `unused` or public per-launch session tokens, because the local router disables client authentication on loopback. `harnesses/setup.sh` links `harnesses/shared/model-router.env` to `~/.config/ai/model-router.env`, which `.zshenv` and the `harness` CLI load. Never copy upstream API keys into an agent config.

Only `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` are exported globally. Claude, Muse, Antigravity, Gemini, Copilot, Kimi CLI, fx, and Vibe map those settings to their own variables in their configs or through the `harness` launcher. Pi runs through its managed launcher and loads the router extension at `~/dev/model-router/integrations/pi/model-router.ts` to discover the router inventory without inheriting placeholders for other providers.

| Agent | Protocol and discovery |
| --- | --- |
| [Codex (Model Router)](codex/README.md) | Responses; `harness codex` defaults to `gpt-6-luna` |
| [Grok Build (Model Router)](grok/README.md#second-provider-through-model-router-in-t3) | Native Chat, Responses, and Anthropic; router inventory with static capability overrides |
| [Claude](claude/README.md) | Anthropic Messages; `claude/settings.json` defaults to `deepseek-v4.1-flash` with full router picker |
| [Cline CLI](cline/README.md) | OpenAI-compatible Chat; selects any router model with `-m` |
| OpenCode | Full inventory across Chat, Responses, and Anthropic provider groups |
| Pi | Router extension discovers all models, including `local-llama` |
| Qwen | OpenAI-compatible and Anthropic provider groups in `qwen/settings.json` with `openai-responses` auth |
| [Kimi Code](kimi-code/README.md) | Full inventory across Chat, Responses, and Anthropic provider groups |
| [Kimi CLI](kimi-cli/README.md) | Legacy Python CLI on Chat ingress; static inventory and per-launch session token |
| [MCode](mcode/README.md) | Full inventory across Chat, Responses, and Anthropic provider groups |
| [Muse](muse/README.md) | Responses directly; model-router serves the catalog at `/v1/muse-code/models` |
| [Antigravity CLI](antigravity/README.md) | Gemini ingress; CLI and router ACP default to `gemini-4-argon` |
| [Gemini CLI](gemini/README.md) | Gemini ingress; source build in `~/dev/gemini-cli` with one ACP entry per model, separate effort controls, and live context limits |
| [fx](fx/README.md) | Chat directly; static inventory in `fx/settings.json` |
| [DeepSeek Harness](deepseek/README.md) | Responses for all models to keep durable session affinity; tracked profiles default to `deepseek-v4.1-flash` |
| [GitHub Copilot CLI](copilot/README.md) | Responses through `harness copilot`; defaults to `gpt-6-luna` |
| [Kilo Code CLI](kilo/README.md) | Chat, Responses, and Anthropic provider groups; native conversation affinity; official ACP in T3 Code |
| [Mistral Vibe](vibe/README.md) | Chat; static router inventory with per-launch affinity; official ACP in T3 Code |

Every client sees the full inventory from `/v1/models`, including Gemini and discovered local models. Claude always sends Anthropic Messages, and Muse and DeepSeek always send Responses. Model-router translates those requests to each model's upstream protocol. Clients with multiple protocol adapters prefer a model's native route and use Chat ingress for Gemini.

fx, Cline, Vibe, and legacy Kimi CLI rely on model-router's `bearer_session_compat` setting to map per-invocation public session tokens and merge consecutive leading system messages. Antigravity CLI and Antigravity Router ACP use the same setting for Gemini API-key session tokens and ACP `sessionId` normalization. Gemini CLI follows the same public API-key token convention.

Model-router applies each provider's strict or lossy translation policy. It reports supported approximations in the `X-Model-Router-Loss` header and rejects requests that require a same-protocol route. Codex and Copilot default to `gpt-6-luna` on Responses because their reasoning requests can require that protocol. A provider-native model can still reject a specific upstream tool schema.

Session affinity comes from per-client headers or per-launch tokens. Pi sends its durable session ID in `x-session-id`. Qwen expands `${session_id}` into `x-session-id`. MCode sends `X-Mavis-Session-Id`, and Kilo sends `X-Session-Id`. DeepSeek sends `session_id` and `prompt_cache_key`. The `harness` launcher generates a fresh per-invocation token for fx, Cline, Vibe, legacy Kimi CLI, Antigravity, and Gemini CLI. None of these configs store a shared install-wide session ID.

## Central launcher

The Go `harness` CLI lives in `~/dev/model-router` under `cmd/harness` and replaces the older shell script that this repository used to link. Build it from `~/dev/model-router` and symlink the binary to `~/.local/bin/harness`:

```sh
cd ~/dev/model-router
go build -o harness ./cmd/harness
mkdir -p "$HOME/.local/bin"
ln -s "$PWD/harness" "$HOME/.local/bin/harness"
```

Pass the agent name as the first argument:

```sh
harness claude
harness codex --help
harness cline -i -m local-llama
harness antigravity --model deepseek-v4.1-flash
harness gemini --model deepseek-v4.1-flash
harness deepseek headless 'your task'
harness vibe-acp
harness --help
```

Run `harness --help` for the complete list of agents and subcommands. `agy`, `dsh`, and `kimi-code` are aliases for `antigravity`, `deepseek`, and `kimi`. `kimi-cli` runs the legacy Python CLI. Calling `harness deepseek` without extra arguments starts its Web UI. All arguments after the agent name pass through unchanged, including flags such as `--help`.

`harness` also provides per-agent profiles via `harness <agent> profile <name>`, along with `harness router`, `harness models`, `harness status`, `harness usage`, `harness doctor`, and `harness spaces`. Installing the same binary under the `spaces` entry name exposes the Spaces hook and session commands directly.

`harness codex`, `harness grok`, and `harness copilot` apply their model-router setups. Plain `codex`, `grok`, and `copilot` keep their primary connections. OpenCode, Pi, Qwen, Kimi Code, Kilo, and MCode run their PATH executables and read their own provider settings.

The launcher reads `~/.config/ai/model-router.env` while respecting `XDG_CONFIG_HOME`. If that file is absent, it falls back to `http://127.0.0.1:4000` and `unused`. Provider-specific variables apply only to the launched child process. In interactive Zsh, `.zshrc` runs `cline`, `dsh`, `fx`, `gemini`, `kilo`, `kimi-cli`, `muse`, `vibe`, and the native `agy` function directly by default. Use `harness <agent>` explicitly when you want the shared launcher environment or when calling an agent from a script.

### T3 Code provider settings

Providers that need the shared launcher point `binaryPath` or `commandPath` to the expanded absolute path of `~/.local/bin/harness`. Native T3 Code providers accept a binary path but cannot prepend an agent name before probes and SDK commands, so set `HARNESS_AGENT` in the provider's `environment` array instead:

```json
{
  "environment": [
    { "name": "HARNESS_AGENT", "value": "claude", "sensitive": false }
  ],
  "config": { "binaryPath": "/home/arhammahajan/.local/bin/harness" }
}
```

Set `HARNESS_AGENT` to `claude`, `codex`, `grok`, `muse`, or `fx` for native providers. `harness` removes `HARNESS_AGENT` from the child process environment so nested `harness` calls can pick a different agent, and it skips the interactive access picker on `HARNESS_AGENT` launches. Keep existing custom models, home paths, and other fields when merging these settings.

Local ACP providers pass the agent selector in `commandArgs`, such as `["antigravity-acp"]` or `["deepseek", "acp"]`. ACP Registry overrides keep the registry's own arguments and ignore `commandArgs`, so Vibe, Kilo, legacy Kimi CLI, and Gemini CLI set `commandPath` to `.../harness` and set `HARNESS_AGENT` to `vibe-acp`, `kilo`, `kimi-cli`, or `gemini`. The registry supplies Kilo's and Kimi's `acp` argument, Gemini's `--acp`, and its own environment variables. Provider JSON templates live in each agent's directory under `harnesses/`.

## User services

`harnesses/shared/model-router.service` defines the systemd user service `model-router.service`. It runs `~/dev/model-router/model-router-go` with `~/.config/model-router/router-go.toml`, listens on `127.0.0.1:4000`, and starts at user login. Muse reads `/v1/muse-code/models` on the same listener.

`harnesses/shared/spaces-hooks.service` defines the optional `spaces-hooks.service` user unit, which runs `~/.local/bin/spaces hook-server --foreground` for the Spaces navigator.

```sh
systemctl --user status model-router
systemctl --user restart model-router
curl http://127.0.0.1:4000/readyz
```

`model-router.service` assumes the router checkout is at `~/dev/model-router`. Edit the unit path if you keep the checkout elsewhere.

## Model names, limits, and effort controls

Router connections use model IDs from `/v1/models` and human labels from each model's `display_name` metadata in `~/dev/model-router`. Pi and Muse read names and capability metadata directly from the router. Other clients store static labels and limits in their tracked configs, while fx's custom-provider catalog displays raw router IDs. Client-required provider prefixes wrap the same underlying IDs. The local model uses ID `local-llama` and display name `Local (llama.cpp)`.

See the [capability audit](model-capabilities.md) for supported fields and client limits. Update static model entries together when upstream limits, image and text support, or reasoning effort lists change.

Restart active agent sessions and open a new shell to pick up environment updates. When removing old global provider exports, log out and back in, then reopen T3 Code and Pi to clear variables inherited from the desktop session and refresh model lists. T3 Code model pickers read their own settings rather than CLI menus.

## Machine setup

Run `harnesses/setup.sh` from this repository checkout:

```sh
./harnesses/setup.sh          # create or refresh the symlinks
./harnesses/setup.sh --check  # verify destinations without writing
```

`harnesses/setup.sh` links the config files, hooks, standalone MCP configs, user service units, and fixed skills directories listed below. Before replacing an existing destination, it moves the old path to `~/.local/share/dots-backups/harnesses-<timestamp>-<pid>/`. It also deletes the obsolete `~/.config/ai/gemini/router-acp.mjs` symlink if that link still points into this checkout. It does not install agent binaries, build the Go `harness` CLI, or enable systemd services.

| Repo path (under `harnesses/`) | Destination |
| --- | --- |
| `shared/model-router.env` | `~/.config/ai/model-router.env` |
| `shared/model-router.service` | `~/.config/systemd/user/model-router.service` |
| `shared/spaces-hooks.service` | `~/.config/systemd/user/spaces-hooks.service` |
| `claude/settings.json` | `~/.claude/settings.json` |
| `codex/linux-config.toml` | `~/.codex/config.toml` (Linux only) |
| `codex/linux-hooks.json` | `~/.codex/hooks.json` (Linux only) |
| `copilot/settings.json` | `~/.copilot/settings.json` |
| `copilot/mcp-config.json` | `~/.copilot/mcp-config.json` |
| `copilot/hooks/zellij.json` | `~/.copilot/hooks/zellij.json` |
| `cursor/hooks.json` | `~/.cursor/hooks.json` |
| `grok/config.toml` | `~/.grok/config.toml` |
| `grok/hooks/zellij.json` | `~/.grok/hooks/zellij.json` |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` |
| `opencode/opencode.jsonc` | `~/.config/opencode/opencode.jsonc` |
| `opencode/cli.json` | `~/.config/opencode/cli.json` |
| `opencode/plugins/zellij-status.ts` | `~/.config/opencode/plugins/zellij-status.ts` |
| `pi/settings.json` | `~/.pi/agent/settings.json` |
| `pi/mcp.json` | `~/.pi/agent/mcp.json` |
| `qwen/settings.json` | `~/.qwen/settings.json` |
| `gemini/settings.json` | `~/.gemini/settings.json` |
| `kimi-code/config.toml` | `~/.kimi-code/config.toml` |
| `kimi-code/mcp.json` | `~/.kimi-code/mcp.json` |
| `kimi-cli/config.toml` | `~/.kimi/config.toml` |
| `kimi-cli/mcp.json` | `~/.kimi/mcp.json` |
| `mcode/config.yaml` | `~/.minimax/config.yaml` |
| `mcode/mcp.json` | `~/.minimax/mcp.json` |
| `antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` |
| `antigravity/mcp_config.json` | `~/.gemini/config/mcp_config.json` |
| `antigravity/acp/settings.json` | `~/.gemini/antigravity-acp/settings.json` |
| `muse/settings.json` | `~/.config/muse/settings.json` |
| `fx/settings.json` | `~/.fx/settings.json` |
| `fx/mcp.json` | `~/.fx/mcp.json` |
| `kilo/kilo.json` | `~/.config/kilo/kilo.json` |
| `vibe/config.toml` | `~/.vibe/config.toml` |
| `cline/cline_mcp_settings.json` | `~/.cline/data/settings/cline_mcp_settings.json` |
| `deepseek/profiles` | `~/.dsh/profiles` |
| `deepseek/tui` | `~/.dsh-tui` |
| `t3code/settings.json` | `~/.t3/userdata/settings.json` |
| `t3code/client-settings.json` | `~/.t3/userdata/client-settings.json` |
| `t3code/keybindings.json` | `~/.t3/userdata/keybindings.json` |

When `~/dev/model-router/integrations/pi/model-router.ts` exists, `setup.sh` also links it to `~/.pi/agent/extensions/model-router.ts`.

T3 Code's `settings.json` stores `providerInstances` and model preferences. `client-settings.json` and `keybindings.json` store UI preferences. `desktop-settings.json` stays machine-local because it holds window geometry and runtime state.

After running `setup.sh`, complete the manual steps for the tools you use:

1. Build `model-router-go` and `harness` in `~/dev/model-router`, then symlink `harness` to `~/.local/bin/harness`.
2. Run `systemctl --user daemon-reload` and `systemctl --user enable --now model-router`. Optionally run `systemctl --user enable --now spaces-hooks.service`.
3. Merge `claude/mcp.json` and `codex/config.toml` MCP entries as described in [browser MCP setup](browser-mcp.md).
4. Run `harnesses/deepseek/setup.sh` if DeepSeek Harness is installed.
5. Link `grok/router-config.toml` to `~/.grok-t3/model-router/config.toml` if you use `grok_router`.

A few clients have specific file-handling rules. MCode's `config.yaml` uses JSON syntax, which its YAML parser accepts. Kimi, MCode, and Antigravity CLI may replace a symlink with a regular file when saving settings; copy live edits back into the tracked file and rerun `setup.sh`. DeepSeek links the `profiles` and `tui` directories so atomic file replacements stay inside this checkout, and `harnesses/deepseek/setup.sh` preserves those tracked files. fx follows symlinks for `~/.fx/settings.json` and `~/.fx/mcp.json` while keeping private state outside the repo.

fx builds from `~/dev/fx`, branch `main` of the [fx fork](https://github.com/arhammahajan/fx/tree/main), and installs its launcher target at `~/.local/libexec/fx/fx-router`. See the [fx setup notes](fx/README.md) for `build-fx.py` and the direct router setup.

## Shared skills

The single skill catalog lives at `harnesses/shared/skills`. There are no per-agent skill folders. Agents with a skills config key point directly to `~/dev/dots/harnesses/shared/skills`. Agents with a fixed user skills directory receive a symlink from `harnesses/setup.sh`. Any skill added under `harnesses/shared/skills` becomes available to all supported agents.

The catalog vendors Poteto's pstack, a set of 58 skills, file-for-file inside `harnesses/shared/skills`. See [`shared/skills/UPSTREAM.md`](../../harnesses/shared/skills/UPSTREAM.md) for the upstream commit, the local renames to `pstack-tdd` and `pstack-teach`, and the re-sync steps.

| Agent | Skill discovery |
| --- | --- |
| [Claude](claude/README.md) | Symlink `~/.claude/skills` to `shared/skills` |
| [Codex](codex/README.md) | Symlink `~/.codex/skills/shared` to `shared/skills`; `.system` stays separate |
| [Cline](cline/README.md) | Symlink `~/.cline/skills` to `shared/skills` |
| [Copilot](copilot/README.md) | `skillDirectories` in `copilot/settings.json` |
| [Grok](grok/README.md) | `[skills] paths` in `grok/config.toml` and `grok/router-config.toml` |
| OpenCode | `skills` in `opencode/opencode.json` |
| [Kilo](kilo/README.md) | `skills` in `kilo/kilo.json` |
| Pi | `skills` in `pi/settings.json` |
| Qwen | `skills.directories` in `qwen/settings.json` |
| [Kimi Code](kimi-code/README.md) | `extra_skill_dirs` in `kimi-code/config.toml` and symlink `~/.kimi-code/skills` |
| [Kimi CLI](kimi-cli/README.md) | `extra_skill_dirs` in `kimi-cli/config.toml` and symlink `~/.kimi/skills` |
| [MCode](mcode/README.md) | Symlink `~/.minimax/skills` to `shared/skills` |
| [Muse](muse/README.md) | Symlink each `~/.config/muse/skills/<skill>` entry to `shared/skills/<skill>` |
| [Antigravity](antigravity/README.md) | Symlink `~/.gemini/config/skills` to `shared/skills` |
| [Gemini CLI](gemini/README.md) | Symlink `~/.gemini/skills` to `shared/skills`; `skills.enabled: true` in `gemini/settings.json` |
| [fx](fx/README.md) | Symlink `~/.fx/skills` to `shared/skills` |
| [Vibe](vibe/README.md) | `skill_paths` in `vibe/config.toml` |
| [DeepSeek](deepseek/README.md) | None; the installed release has no skill discovery |

`harnesses/setup.sh` links the whole folder at `~/.codex/skills/shared` so Codex's nested discovery finds every skill without colliding with `.system`. For Muse, `setup.sh` links each skill entry individually because Muse manages `~/.config/muse/skills` as a store, and prunes broken symlinks that point into `shared/skills`. Agents with an explicit config key need no home-directory symlink.

If an agent ever requires a subset of skills, point its config key or fixed directory at a regular directory instead. No agent uses a subset today. Several agents also scan `~/.agents/skills`, so configuring `shared/skills` explicitly keeps behavior independent of `~/.agents/skills`.

## Second Codex provider in T3 Code

`harnesses/codex/t3-router-provider.json` records the separate `codex_router` provider. `harness codex` passes process-local overrides while T3 Code shares Codex state through a shadow home. See the [Codex setup](codex/README.md) for paths, credentials, model inventory, and router compatibility details.

## Second Grok Build provider in T3 Code

`harnesses/grok/t3-router-provider.json` records the separate `grok_router` provider. `harness grok` uses an isolated Grok home and sets process-local endpoint and API-key variables. See the [Grok setup](grok/README.md#second-provider-through-model-router-in-t3) for paths, model discovery, capability overrides, and session affinity.
