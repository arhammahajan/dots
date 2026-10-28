# Coding harnesses through model-router

Claude Code, Cline CLI, OpenCode, Pi, Qwen Code, Kimi Code, MCode, Muse, Antigravity CLI, and
fx use the local model-router directly at `http://127.0.0.1:4000`. A second T3 Codex instance also
uses the router; the primary Codex instance, Grok, and Copilot keep their
existing connections.
The router checkout is at `~/dev/model-router`.

Only model-router reads upstream credentials from `~/.config/ai/credentials.env`.
Clients use the public placeholder `unused` (fx and Cline use public per-launch
session tokens): the router's current configuration
explicitly disables client authentication on loopback. The shared client settings
are linked at `~/.config/ai/model-router.env`, sourced by `.zshenv` and the Claude
and Muse launchers. Never copy the upstream key into a harness config.
Only `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` are exported globally. Claude,
Muse, and Antigravity map them to their own endpoint and credential variables in
their settings or launchers. Pi uses its original managed launcher and discovers
the router inventory without inheriting placeholders for other providers.

| Harness | Protocol and discovery |
| --- | --- |
| [Codex (Model Router)](codex/README.md) | Responses; T3 custom models from the router inventory |
| [Claude](claude/README.md) | Anthropic; picker includes every router model |
| [Cline CLI](cline/README.md) | OpenAI-compatible Chat; any router model via `-m` |
| OpenCode | Full inventory across three protocol provider groups |
| Pi | Router extension discovers all models, including llama.cpp |
| Qwen | Anthropic ingress for the full inventory |
| [Kimi](kimi-code/README.md) | Full inventory across three protocol provider groups |
| [MCode](mcode/README.md) | Full inventory across three protocol provider groups |
| [Muse](muse/README.md) | Responses directly; Muse catalog is built into model-router |
| [Antigravity CLI](antigravity/README.md) | Gemini ingress; all router models in the picker |
| [fx](fx/README.md) | Chat directly; full static inventory |

Every harness sees the complete inventory from `/v1/models`, including Gemini
and discovered local models. Claude always uses Anthropic Messages, and Muse
always uses Responses; the router translates to the model's upstream protocol.
Clients with multiple protocol adapters prefer a native route where available,
and use Chat ingress for Gemini. Model availability is independent of ingress.

fx and Cline use the main router's opt-in `bearer_session_compat` mode to map
per-invocation public session tokens and combine leading system messages.
No separate compatibility service is required.

The local `router.toml` explicitly selects `compatibility = "lossy"` on its
providers and llama.cpp discovery. This allows nonportable controls such as
Responses `store`, encrypted reasoning requests, reasoning summaries, or
Gemini-incompatible cache hints to be dropped or approximated. The router reports
each change in `x-model-router-loss`.
It retains strict mode as the default for other configurations. Ordinary function
tools are translated, and Muse's namespaced function tools are flattened and
restored with their namespace on JSON and streaming responses. Upstream-native
features such as server-side conversation state still require their native API.

Pi uses its durable session ID in `x-session-id`. Qwen expands `${session_id}`
into that header. MCode supplies `X-Mavis-Session-Id`; the router recognizes it.
fx and Antigravity CLI launchers supply a fresh per-invocation session ID.
Other clients' native session headers or `prompt_cache_key` supply affinity.
There are no shared install-wide session IDs in these configs.

## Services on this machine

`model-router.service` starts the router from its existing virtual environment
and `router.toml`. Muse catalog compatibility is built into the router at
`/muse-code/models` and `/v1/muse-code/models`; Muse connects directly to port
4000. These clients need no separate adapter service.

```sh
systemctl --user status model-router
systemctl --user restart model-router
curl http://127.0.0.1:4000/readyz
```

The service starts at user login. Its unit file assumes `~/dev/model-router`;
adjust the path for a different checkout location.

## Model names and IDs

The router connections use model IDs from `/v1/models`. Shared display names
come from `~/dev/model-router/model_router/model_names.py`. Pi and Muse discover
names from the router; other menus store those labels where supported. fx's
custom-provider catalog shows the router IDs.
Harness-required provider prefixes wrap the same underlying IDs. The local
model uses `local-llama` with the display name **Local (llama.cpp)**.

## Model limits and effort controls

Pi and Muse read metadata from the router. The other clients use the tracked
static model entries. See the [capability audit](model-capabilities.md) for the
supported fields and client limitations. Update those entries when upstream
limits or effort vocabularies change, keeping context, output, modalities, and
reasoning capabilities together.

Restart existing harness sessions and open a new shell to pick up the environment.
When removing old global provider exports, log out and back in, then reopen T3
and Pi to clear variables inherited from the desktop session and refresh model lists.
In T3, use `~/.local/bin/muse-with-env` for Muse and
`~/dev/dots/harnesses/claude/claude-with-env` for Claude. T3 model pickers have
their own settings; CLI menus do not populate them.

## Links on another machine

Back up existing files, then link the individual configs from this checkout
(use managed copies for Kimi, MCode, Antigravity CLI, and fx settings):

| Repo file | Destination |
| --- | --- |
| `shared/model-router.env` | `~/.config/ai/model-router.env` |
| `shared/model-router.service` | `~/.config/systemd/user/model-router.service` |
| `cline/cline-with-router` | `~/.local/bin/cline` (run `sh harnesses/cline/setup.sh` too) |
| `claude/settings.json` | `~/.claude/settings.json` |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` |
| `opencode/opencode.jsonc` | `~/.config/opencode/opencode.jsonc` |
| `opencode/cli.json` | `~/.config/opencode/cli.json` |
| `pi/settings.json` | `~/.pi/agent/settings.json` |
| `qwen/settings.json` | `~/.qwen/settings.json` |
| `kimi-code/config.toml` | `~/.kimi-code/config.toml` (managed copy) |
| `mcode/config.yaml` | `~/.minimax/config.yaml` (managed copy) |
| `antigravity/agy-with-env` | `~/.local/bin/agy-with-env` |
| `antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` (managed copy) |
| `muse/settings.json` | `~/.config/muse/settings.json` |
| `muse/muse-with-env` | `~/.local/bin/muse-with-env` |
| `fx/settings.json` | `~/.fx/settings.json` (managed copy) |
| `fx/fx-with-router` | `~/.local/bin/fx` and `~/.local/bin/fx-with-router` |

Paths in the left column are relative to `harnesses/`. Also link
`~/dev/model-router/integrations/pi/model-router.ts` at
`~/.pi/agent/extensions/model-router.ts`. Install the router dependencies following
its README, then run `systemctl --user daemon-reload` and
`systemctl --user enable --now model-router`.

MCode's `config.yaml` uses JSON syntax, which is valid YAML. Kimi and MCode can
replace symlinks when saving; preserve personal preferences when updating their
active copies. Backups from setup are under `~/.local/share/dots-backups/`.

Antigravity CLI uses a managed copy of `antigravity/cli/settings.json` at
`~/.gemini/antigravity-cli/settings.json`, preserving machine-specific trusted
workspaces. See its [setup notes](antigravity/README.md).

fx builds from `~/dev/fx`, branch `main` of the [fork](https://github.com/arhammahajan/fx/tree/main).
Its launcher uses `~/.local/libexec/fx/fx-router`. See the
[fx setup notes](fx/README.md) for the build helper and direct router setup.

## Second Codex provider in T3

`codex/t3-router-provider.json` records the separate `codex_router` provider.
Its launcher supplies process-local overrides while T3 shares the existing
Codex state through a shadow home. See the [Codex setup](codex/README.md) for
paths, credentials, model inventory, and router compatibility requirements.
