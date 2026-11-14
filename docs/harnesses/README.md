# Coding harnesses through model-router

For Orca's terminal agent picker and additional provider configurations, see
[Orca setup](orca.md).

All harnesses also include the shared Helium `agent-browser`, `playwright`, and
`browser-use` MCP servers. See [browser MCP setup](browser-mcp.md) for config
destinations, launcher dependencies, and interaction highlighting.

Claude Code, Cline CLI, OpenCode, Pi, Qwen Code, Kimi Code, MCode, Muse,
Antigravity CLI, Gemini CLI, fx, DeepSeek Harness, Mistral Vibe, and Kilo Code use
the local model-router directly at `http://127.0.0.1:4000`. Second T3 Codex and Grok Build instances also
use the router; the primary Codex and Grok instances and Copilot keep their
existing connections.
The router checkout is at `~/dev/model-router`.

Only model-router reads upstream credentials from `~/.config/ai/credentials.env`.
Clients use the public placeholder `unused` or public per-launch session tokens
(including Gemini CLI): the router's current configuration
explicitly disables client authentication on loopback. The shared client settings
are linked at `~/.config/ai/model-router.env`, sourced by `.zshenv` and the shared
`harness` launcher. Never copy the upstream key into a harness config.
Only `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` are exported globally. Claude,
Muse, Antigravity, and Gemini map them to their own endpoint and credential
variables in their settings or launchers. Pi uses its original managed launcher and discovers
the router inventory without inheriting placeholders for other providers.

| Harness | Protocol and discovery |
| --- | --- |
| [Codex (Model Router)](codex/README.md) | Responses; T3 custom models from the router inventory |
| [Grok Build (Model Router)](grok/README.md#second-provider-through-model-router-in-t3) | Native Chat, Responses, and Anthropic; router inventory with capability overrides |
| [Claude](claude/README.md) | Anthropic; picker includes every router model |
| [Cline CLI](cline/README.md) | OpenAI-compatible Chat; any router model via `-m` |
| OpenCode | Full inventory across three protocol provider groups |
| Pi | Router extension discovers all models, including llama.cpp |
| Qwen | Anthropic ingress for the full inventory |
| [Kimi](kimi-code/README.md) | Full inventory across three protocol provider groups |
| [MCode](mcode/README.md) | Full inventory across three protocol provider groups |
| [Muse](muse/README.md) | Responses directly; Muse catalog is built into model-router |
| [Antigravity CLI](antigravity/README.md) | Gemini ingress; all router models in the picker |
| [Gemini CLI](gemini/README.md) | Gemini ingress; maintained source build with one ACP entry per model, separate effort controls, and live context limits |
| [fx](fx/README.md) | Chat directly; full static inventory |
| [DeepSeek Harness](deepseek/README.md) | Responses; full current inventory with per-session affinity |
| [Kilo Code CLI](kilo/README.md) | Full current inventory across Chat, Responses, and Anthropic groups; native conversation affinity; official ACP in T3 |
| [Mistral Vibe](vibe/README.md) | Chat; static router inventory with per-launch affinity; official ACP in T3 |

Every harness sees the complete inventory from `/v1/models`, including Gemini
and discovered local models. Claude always uses Anthropic Messages, and Muse
always uses Responses; the router translates to the model's upstream protocol.
DeepSeek Harness also uses Responses for all models to preserve durable session
affinity with its installed adapter. Other clients with multiple protocol
adapters prefer a native route where available,
and use Chat ingress for Gemini. Model availability is independent of ingress.

fx, Cline, and Vibe use the main router's opt-in `bearer_session_compat` mode to map
per-invocation public session tokens and combine leading system messages.
Antigravity uses the same setting for Gemini API-key session tokens and ACP
`sessionId` normalization. Both Antigravity connections use the router directly.
Gemini CLI uses the same public API-key session token convention.

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
fx, Antigravity CLI, and Gemini CLI launchers supply a fresh per-invocation session ID.
Other clients' native session headers or `prompt_cache_key` supply affinity.
There are no shared install-wide session IDs in these configs.

## One launcher for all agents

`harnesses/harness` is the central shell launcher. Install it on PATH once,
running this from the dots checkout:

```sh
mkdir -p "$HOME/.local/bin"
ln -s "$PWD/harnesses/harness" "$HOME/.local/bin/harness"
```

Then select the agent as the first argument:

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

Run `harness --help` for the full list. `agy`, `dsh`, and `kimi-code` are aliases
for `antigravity`, `deepseek`, and `kimi`. `kimi-cli` selects the legacy Python
CLI. `deepseek` without arguments starts its Web UI. All arguments after the
agent name are forwarded unchanged, including flags such as `--help`.

`harness codex` and `harness grok` select their model-router setups. The plain
`codex` and `grok` commands keep their primary connections. OpenCode, Pi, Qwen,
Copilot, Kimi Code, and MCode run their existing PATH executables and use their
own provider settings.

The launcher's `case` statement owns environment setup, binary selection, and
per-launch session tokens. It loads `~/.config/ai/model-router.env` (respecting
`XDG_CONFIG_HOME`); if absent, it defaults to `http://127.0.0.1:4000` and the
public token `unused`. Provider-specific variables apply only to the launched
process. Antigravity's CLI and ACP adapter connect directly to the router;
session normalization is handled there. The launcher uses short Python commands
to read settings and the ACP inventory, then executes the agent directly.

There are no individual launcher scripts or compatibility symlinks. Interactive
Zsh shortcuts (`agy`, `cline`, `dsh`, `fx`, `gemini`, `kilo`, `kimi-cli`, `muse`, and `vibe`)
call `harness <agent>`; scripts should use the explicit command.

### T3 provider settings

All providers that previously needed a custom launcher now use the same
absolute executable path, `~/.local/bin/harness` (expand `~` in saved settings).
Native providers accept a binary path but cannot prepend the agent name before
all probes and SDK commands. Set a provider-level environment entry instead:

```json
{
  "environment": [
    { "name": "HARNESS_AGENT", "value": "claude", "sensitive": false }
  ],
  "config": { "binaryPath": "/home/arhammahajan/.local/bin/harness" }
}
```

Use `claude`, `codex`, `grok`, `muse`, or `fx` as appropriate. The selector is
removed from the launched agent's environment so nested `harness` calls can
choose a different agent. Existing custom models, homes, and other provider
settings must be preserved when merging these launch settings.

Local ACP providers pass the selector as their first `commandArgs` entry:
`["antigravity-acp"]` or `["deepseek", "acp"]`.
ACP Registry overrides retain the registry's arguments and ignore `commandArgs`,
so Vibe, Kilo, legacy Kimi CLI, and Gemini CLI use `commandPath: ".../harness"`
with `HARNESS_AGENT=vibe-acp`, `kilo`, `kimi-cli`, or `gemini`. The registry
supplies Kilo's and Kimi's `acp` argument, Gemini's `--acp`, and its own
environment. Router URL and key settings come from the
shared launcher environment.
Provider JSON templates in the individual agent directories record these settings.

To check dispatch and environment handling without starting agents or making
model requests:

```sh
python3 -m unittest discover -s harnesses/tests -v
```

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
In T3, use `~/.local/bin/harness` with the provider settings above. T3 model
pickers have their own settings; CLI menus do not populate them.

## Setup on a machine

Run the setup script from this checkout:

```sh
./harnesses/setup.sh          # create or refresh the links
./harnesses/setup.sh --check  # verify without writing
```

`setup.sh` symlinks the files below, the shared launcher, standalone MCP files,
and every fixed skills directory. Existing destinations are moved to
`~/.local/share/dots-backups/harnesses-<timestamp>-<pid>/` before replacement.
It does not install harness binaries, extract managed copies, or enable services.

| Repo file | Destination |
| --- | --- |
| `shared/model-router.env` | `~/.config/ai/model-router.env` |
| `shared/model-router.service` | `~/.config/systemd/user/model-router.service` |
| `harness` | `~/.local/bin/harness` |
| `claude/settings.json` | `~/.claude/settings.json` |
| `copilot/settings.json` | `~/.copilot/settings.json` |
| `grok/config.toml` | `~/.grok/config.toml` |
| `grok/router-config.toml` | `~/.grok-t3/model-router/config.toml` |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` |
| `opencode/opencode.jsonc` | `~/.config/opencode/opencode.jsonc` |
| `opencode/cli.json` | `~/.config/opencode/cli.json` |
| `pi/settings.json` | `~/.pi/agent/settings.json` |
| `qwen/settings.json` | `~/.qwen/settings.json` |
| `gemini/settings.json` | `~/.gemini/settings.json` |
| `kimi-code/config.toml` | `~/.kimi-code/config.toml` |
| `kimi-cli/config.toml` | `~/.kimi/config.toml` |
| `mcode/config.yaml` | `~/.minimax/config.yaml` |
| `antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` |
| `muse/settings.json` | `~/.config/muse/settings.json` |
| `fx/settings.json` | `~/.fx/settings.json` |
| `kilo/kilo.json` | `~/.config/kilo/kilo.json` |
| `vibe/config.toml` | `~/.vibe/config.toml` |

Paths in the left column are relative to `harnesses/`. `setup.sh` also links
`~/dev/model-router/integrations/pi/model-router.ts` when that checkout exists,
and the standalone MCP files listed in [browser MCP setup](browser-mcp.md).
Install the router dependencies following its README, then run
`systemctl --user daemon-reload` and `systemctl --user enable --now model-router`.

MCode's `config.yaml` uses JSON syntax, which is valid YAML. Kimi, MCode,
Antigravity CLI, and fx may replace a symlink when saving; rerun `setup.sh` after
they rewrite a file, or keep a managed copy if you prefer. DeepSeek Harness
profiles are installed and refreshed by `deepseek/setup.sh`; the general setup
leaves them alone.

fx builds from `~/dev/fx`, branch `main` of the [fork](https://github.com/arhammahajan/fx/tree/main).
Its launcher uses `~/.local/libexec/fx/fx-router`. See the
[fx setup notes](fx/README.md) for the build helper and direct router setup.

## Shared skills

The catalog lives once at `shared/skills`. There are no per-harness skill
folders: harnesses with an explicit skills setting point directly at
`~/dev/dots/harnesses/shared/skills`, and harnesses with a fixed user skills
directory get a symlink to that same directory. Adding a skill under
`shared/skills` is immediately visible to every harness.

Three upstream catalogs are vendored there file-for-file: Matt Pocock's
engineering and productivity skills (27), Poteto's pstack (58), and Addy
Osmani's production-engineering skills (25). Their upstream commits, the local
renames (`pstack-tdd`, `pstack-teach`), the shared checklists Addy's skills
reach for at `shared/references/`, and the re-sync steps are recorded in
[`shared/skills/UPSTREAM.md`](../../harnesses/shared/skills/UPSTREAM.md).

| Harness | Discovery |
| --- | --- |
| [Claude](claude/README.md) | Link `~/.claude/skills` to `shared/skills`; no settings key |
| [Codex](codex/README.md) | Link the folder at `~/.codex/skills/shared`; `.system` stays separate |
| [Cline](cline/README.md) | Link `~/.cline/skills` to `shared/skills` |
| [Copilot](copilot/README.md) | `skillDirectories` in `settings.json` |
| [Grok](grok/README.md) | `[skills] paths` in each `config.toml` |
| OpenCode | `skills` in `opencode.json` |
| [Kilo](kilo/README.md) | `skills` in `kilo.json` |
| Pi | `skills` in `settings.json` |
| Qwen | `skills.directories` in `settings.json` |
| [Kimi Code](kimi-code/README.md) | `extra_skill_dirs` in `config.toml`, plus the fixed `~/.kimi-code/skills` link |
| [Kimi CLI](kimi-cli/README.md) | `extra_skill_dirs` in `config.toml`, plus the fixed `~/.kimi/skills` link |
| [MCode](mcode/README.md) | Link `~/.minimax/skills` to `shared/skills` |
| [Muse](muse/README.md) | Link each `~/.config/muse/skills/<skill>` to `shared/skills/<skill>` |
| [Antigravity](antigravity/README.md) | Link `~/.gemini/config/skills` to `shared/skills` |
| [Gemini CLI](gemini/README.md) | Link `~/.gemini/skills` to `shared/skills`; `skills.enabled: true` |
| [fx](fx/README.md) | Link `~/.fx/skills` to `shared/skills` |
| [Vibe](vibe/README.md) | `skill_paths` in `config.toml` |
| [DeepSeek](deepseek/README.md) | none; installed release has no skill discovery |

`harnesses/setup.sh` wires the whole checkout into a machine. It symlinks the
harness configs, the shared launcher, router client settings, standalone MCP
files, and every fixed skills directory. It writes only symlinks in `$HOME` and
moves any existing destination into `~/.local/share/dots-backups/` first:

```sh
./harnesses/setup.sh          # create or refresh the links
./harnesses/setup.sh --check  # verify every destination without writing
```

For Codex it links the whole folder at `~/.codex/skills/shared` so nested
discovery finds every skill. For Muse it links each entry because Muse owns that
directory as a store. Harnesses with an explicit skills setting need no
home-directory link.

If a harness ever needs a subset, point its config or fixed-directory link at a
real directory instead. Nothing uses that layout today.

Several harnesses also read the generic `~/.agents/skills` directory. The
explicit `shared/skills` path is used instead so behavior does not depend on that
global directory being set up.

## Second Codex provider in T3

`codex/t3-router-provider.json` records the separate `codex_router` provider.
Its launcher supplies process-local overrides while T3 shares the existing
Codex state through a shadow home. See the [Codex setup](codex/README.md) for
paths, credentials, model inventory, and router compatibility requirements.

## Second Grok Build provider in T3

`grok/t3-router-provider.json` records the separate `grok_router` provider.
Its launcher uses an isolated Grok home and process-local endpoint and API-key
variables. See the [Grok setup](grok/README.md#second-provider-through-model-router-in-t3)
for paths, model discovery, capability overrides, and upstream session affinity.
