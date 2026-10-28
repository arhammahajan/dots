# Coding harnesses through model-router

Claude Code, OpenCode, Pi, Qwen Code, Kimi Code, MCode, Muse, and Antigravity CLI use
`http://127.0.0.1:4000`. A second T3 Codex instance also uses the router; the
primary Codex instance, Grok, and Copilot keep their existing connections.
The router checkout is at `~/dev/model-router`.

Only model-router reads upstream credentials from `~/.config/ai/credentials.env`.
Clients use the public placeholder `unused`: the router's current configuration
explicitly disables client authentication on loopback. The shared client settings
are linked at `~/.config/ai/model-router.env`, sourced by `.zshenv` and the Claude
and Muse launchers. Never copy the upstream key into a harness config.

| Harness | Default | Protocol and discovery |
| --- | --- | --- |
| Codex (Model Router) | `deepseek-v4.1-flash` | Responses; T3 custom models from the router inventory |
| Claude | `claude-haiku-5-5` | Anthropic; picker includes every router model |
| OpenCode | `model-router/deepseek-v4.1-flash` | Full inventory across three protocol provider groups |
| Pi | `model-router/deepseek-v4.1-flash` | Router extension discovers all models, including llama.cpp |
| Qwen | `qwen3.8-max` | Anthropic ingress for the full inventory |
| Kimi | `model-router/deepseek-v4.1-flash` | Full inventory across three protocol provider groups |
| MCode | `custom_provider:model-router/deepseek-v4.1-flash` | Full inventory across three protocol provider groups |
| Muse | `gpt-6-luna` | Responses directly; Muse catalog is built into model-router |
| Antigravity CLI | `deepseek-v4.1-flash` | Gemini ingress; all router models in the picker |

Every harness sees the complete inventory from `/v1/models`, including Gemini
and discovered local models. Claude always uses Anthropic Messages, and Muse
always uses Responses; the router translates to the model's upstream protocol.
Clients with multiple protocol adapters prefer a native route where available,
and use Chat ingress for Gemini. Model availability is independent of ingress.

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
Other clients' native session headers or `prompt_cache_key` supply affinity.
There are no shared install-wide session IDs in these configs.

## Services on this machine

`model-router.service` starts the router from its existing virtual environment
and `router.toml`. Muse catalog compatibility is built into the router at
`/muse-code/models` and `/v1/muse-code/models`; Muse connects directly to port
4000. No separate adapter process or service is needed.

```sh
systemctl --user status model-router
systemctl --user restart model-router
curl http://127.0.0.1:4000/readyz
```

The service starts at user login. Its unit file assumes `~/dev/model-router`;
adjust the path for a different checkout location.

## Model names and IDs

All eight harnesses use the router model IDs and shared display names from
`~/dev/model-router/model_router/model_names.py`. Pi and Muse discover names
from the router; the other six store the same labels in their model menus.
Harness-required provider prefixes wrap the same underlying IDs. The local
model uses `local-llama` with the display name **Local (llama.cpp)**.

## Model limits and effort controls

Pi and Muse read metadata from the router. The other clients use the tracked
static model entries. See the [capability audit](model-capabilities.md) for the
supported fields and client limitations. Update those entries when upstream
limits or effort vocabularies change. The old `sync-router-models.py` predates
these capability settings and will overwrite them; do not use it to refresh
these configs.

Restart existing harness sessions and open a new shell to pick up the environment.
In T3, use `~/.local/bin/muse-with-env` for Muse and
`~/dev/dots/harnesses/claude/claude-with-env` for Claude. T3 model pickers have
their own settings; CLI menus do not populate them.

## Links on another machine

Back up existing files, then link the individual configs from this checkout
(copy MCode's file instead of linking it):

| Repo file | Destination |
| --- | --- |
| `shared/model-router.env` | `~/.config/ai/model-router.env` |
| `shared/model-router.service` | `~/.config/systemd/user/model-router.service` |
| `claude/settings.json` | `~/.claude/settings.json` |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` |
| `opencode/opencode.jsonc` | `~/.config/opencode/opencode.jsonc` |
| `pi/settings.json` | `~/.pi/agent/settings.json` |
| `qwen/settings.json` | `~/.qwen/settings.json` |
| `kimi-code/config.toml` | `~/.kimi-code/config.toml` |
| `mcode/config.yaml` | `~/.minimax/config.yaml` |
| `antigravity/agy-with-env` | `~/.local/bin/agy-with-env` |
| `muse/settings.json` | `~/.config/muse/settings.json` |
| `muse/muse-with-env` | `~/.local/bin/muse-with-env` |

Paths in the left column are relative to `harnesses/`. Also link
`~/dev/model-router/integrations/pi/model-router.ts` at
`~/.pi/agent/extensions/model-router.ts`. Install the router dependencies following
its README, then run `systemctl --user daemon-reload` and
`systemctl --user enable --now model-router`.

MCode's `config.yaml` uses JSON syntax, which is valid YAML, so the refresh script
needs only Python's standard library. Qwen's active file is now a symlink; MCode
uses a managed copy because its CLI replaces links when saving. Their former
independent copies are backed up under
`~/.local/share/dots-backups/model-router-20261027-213743/`.

Antigravity CLI uses a managed copy of `antigravity/cli/settings.json` at
`~/.gemini/antigravity-cli/settings.json`, preserving machine-specific trusted
workspaces. See its [setup notes](antigravity/README.md).

## Second Codex provider in T3

`codex/t3-router-provider.json` records the `codex_router` entry installed in
`~/.t3/userdata/settings.json` under `providerInstances`. Its paths describe this
machine. Use existing CLI setup with `codex/codex-with-router` as the binary,
`~/.codex` as homePath, and `~/.codex-t3/model-router` as shadowHomePath.
T3 shares state and config through symlinks, while the shadow auth.json contains
only `{"OPENAI_API_KEY":"unused"}` (mode 0600). The launcher supplies process-local
provider overrides, so the shared Codex config and primary login are unaffected.
The T3 picker has a snapshot of `/v1/models`; refresh customModels when it changes.

Codex 0.165 sends Responses `client_metadata`. The local model-router accepts
that field and removes it with an `x-model-router-loss` diagnostic when translating
in lossy mode; strict mode continues to reject the unsupported translation.
This compatibility change lives in `~/dev/model-router/model_router/constants.py`.
