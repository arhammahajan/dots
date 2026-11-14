# Gemini CLI through model-router

Run `harness gemini`, or `gemini` in a new Zsh session. Both the terminal and
T3 use the maintained source checkout at `~/dev/gemini-cli`, branch `router-acp`.
The launcher runs `packages/cli/dist/index.js` directly with fnm's default Node
installation. Set `GEMINI_CLI_REPO` to use another checkout.

## Build and maintain the checkout

The repository is cloned from Google's upstream, with that remote named
`upstream`. Your fork is [arhammahajan/gemini-cli](https://github.com/arhammahajan/gemini-cli),
available locally as `origin`; `router-acp` tracks `origin/router-acp`.
`router-acp` is also the fork's default branch on GitHub.
Local router support is committed on `router-acp`, so it can be rebased normally.

After pulling upstream changes:

```sh
cd ~/dev/gemini-cli
git fetch upstream
git rebase upstream/main
~/dev/dots/harnesses/gemini/build.sh --install
```

After reviewing and testing a rebase, publish it with
`git push --force-with-lease origin router-acp`.

`--install` refreshes dependencies with `npm ci --ignore-scripts`. The helper
then generates build metadata and builds core, devtools, and CLI in dependency
order. Without `--install`, it reuses installed dependencies. Node 24.22.1 is
used on this machine. You can also run those build commands yourself; the
launcher uses the checkout immediately after rebuilding, without npm linking
or global installation.

For a fresh machine:

```sh
git clone --branch router-acp https://github.com/arhammahajan/gemini-cli.git ~/dev/gemini-cli
cd ~/dev/gemini-cli
git remote add upstream https://github.com/google-gemini/gemini-cli.git
git fetch upstream
~/dev/dots/harnesses/gemini/build.sh --install
~/dev/dots/harnesses/setup.sh
```

Review and resolve conflicts when upstream changes the ACP APIs. The branch
adds opt-in catalog discovery, real token limits, and native ACP configuration
methods. A separate compatibility commit adds T3's shared effort menu. Terminal
effort commands and their picker are separate changes. A small dialog-manager
fix keeps custom pickers visible in screen-reader mode; that fix can be dropped
when upstream supplies it. There are no bundle rewrites, loader hooks, effort
variants, or additional HTTP proxy processes.

Router metadata parsing and token limits live in core's
`config/modelRouterCatalog.ts` and `core/tokenLimits.ts`. Core `Config` owns
per-model effort state, validation, and header application, shared by terminal
and ACP. CLI's `ui/commands/effortCommand.tsx` supplies terminal commands and
`ui/components/EffortDialog.tsx` reuses Gemini's radio-list picker;
`acp/modelRouterConfig.ts` supplies ACP menus and protocol errors. Keep T3
compatibility choices in the ACP module, and rerun the focused tests below after
rebasing. CLI tests import the built core package, so rebuild core after changing
it and before running CLI tests. No T3 source patch is needed.

## Endpoint and authentication

The launcher sets `GEMINI_MODEL_ROUTER=1`,
`GOOGLE_GEMINI_BASE_URL=$MODEL_ROUTER_URL` (default
`http://127.0.0.1:4000`, without `/v1`), and a fresh public
`router-session-<UUIDv4>` as `GEMINI_API_KEY`. Authentication uses
`security.auth.selectedType: gemini-api-key`. Google login is not required.

The router needs `[server] auth_mode = "none"` and
`bearer_session_compat = true`. It consumes the public token for affinity and
uses its own upstream credentials. Provider-specific variables stay local to
the launched process; upstream credentials are never copied into Gemini.
Resuming in another invocation creates a new router affinity identity.

Router support is inactive without `GEMINI_MODEL_ROUTER=1`. Normal
Google account or API-key usage remains available when launching the source
entry directly outside the harness.

## Terminal model and effort selection

Launch `harness gemini` (or the `gemini` Zsh alias). Inside the terminal UI:

```text
/model set deepseek-v4.1-flash
/effort
/thinking
/effort auto
```

`/effort` and `/thinking` open a selectable list like `/model`, containing
`Upstream default (auto)` plus the selected model's exact published levels.
The current effort is marked and initially highlighted. Move with arrow keys,
press Enter to apply, or Escape to cancel without changing the effort.

Direct arguments still work: `/effort <level>` or `/thinking <level>`. Tab
completion offers `auto` plus only that model's values, including sparse
vocabularies and values such as `xhigh` or `max` when published. `auto` removes
the override and restores the upstream default.

The footer shows the current effort in brackets after the model, for example
`deepseek-v4.1-flash [max]` or `deepseek-v4.1-flash [auto]`. It updates after
picker selections, direct effort commands, and model switches, restoring the
effort saved for the displayed model.

Effort choices are remembered separately for each model during the current
invocation, including switches through `/model` or its dialog. They are not
saved to disk or shared with another terminal or T3 session. Unsupported values
show an error and leave the previous valid effort intact. Models without a
published vocabulary report that fact and accept only `auto`; no tiers or
`off` aliases are invented. `/thinking none` works only when `none` is published.

These commands control model reasoning effort; thought-text display remains a
separate Gemini setting. They are registered only with `GEMINI_MODEL_ROUTER=1`.
The terminal always uses exact per-model choices, independently of T3's shared
ACP menu setting.

## T3 models, effort, and context windows

Select **Gemini CLI (Model Router)** in T3 Code. `acpRegistry_gemini` remains an
**official ACP registry provider** with `agentId: gemini`, its official icon,
and registry-supplied `--acp` argument. Its executable override is the shared
`harness` launcher with `HARNESS_AGENT=gemini`.

The source build discovers `/v1/models` at initialization. T3 shows exactly one
entry per router model: currently 27. Labels use the model's display name, for
example `DeepSeek V4.1 Flash`. Context capacity is shown by T3's native context
meter, and effort is selected through its separate control.

**Effort** is a separate ACP control. The T3 provider opts into
`GEMINI_ACP_SHARED_EFFORT_CONTROL=1`: it offers `Upstream default` plus the
combined published effort values across the router catalog. This keeps the
control visible on every model before its first session. T3 currently probes
only the default model and scopes controls categorized as `thought_level` or
`model_config` to that model. The opt-in leaves Effort uncategorized so T3
shares it across its catalog. It appears as an ordinary provider option.

The combined menu includes values that some models do not support. Gemini
checks every selection against the selected model's exact published values.
An unsupported value returns an error listing valid choices and blocks prompts
until a valid effort or another model is selected. This also prevents clients
that ignore a set-option failure from silently generating at another effort.
Models without a published vocabulary accept only `Upstream default`.

`session/set_config_option` changes model and effort independently, with effort
remembered separately per model. Without the shared-control opt-in, other ACP
clients receive the selected model's exact menu with category `thought_level`;
models with unknown vocabularies have no effort control. Model switches publish
updated options in either mode. No effort levels are renamed or clamped.

Selecting `Upstream default` removes the effort override. Explicit values use
`x-model-router-reasoning-effort`, including `xhigh` and `max`, which Gemini's
native thinking-level enum cannot represent. The router validates the header
against the model's facts and consumes it before forwarding. Existing
strict/lossy translation policies and `x-model-router-loss` reporting still
apply across protocols.

Published context and output limits feed Gemini's native token-limit functions.
ACP `usage_update.size` reports the actual context size, including after model
switches. T3's context meter is enabled in
`~/.t3/userdata/client-settings.json`. Unknown facts retain stock Gemini
fallback behavior.

The provider sets `GEMINI_CLI_TRUST_WORKSPACE=true` only for its process. T3
chooses the workspace and permission mode; Gemini must trust that workspace
before T3 can switch into `autoEdit` or `yolo`. Without this setting a discovery
probe succeeds but full-access startup fails at `session/set_mode`.

`harnesses/gemini/t3-provider.json` records the installed provider. On another
machine, adjust its absolute command path and merge it into `providerInstances`
in `~/.t3/userdata/settings.json`. Shell setup links the tracked T3 user
settings, including `providerInstances`; merge new provider templates there.
Increment `GEMINI_ROUTER_ACP_REVISION` after rebuilding changed code to
invalidate T3's discovery cache. Existing agent processes need a new session
to load rebuilt code.

## Shared settings, skills, and MCP

`setup.sh` links `harnesses/gemini/settings.json` to `~/.gemini/settings.json`
and the shared catalog to `~/.gemini/skills`. It removes the former managed
runtime-hook symlink, backing up replaced configuration paths as usual.

The default model is `deepseek-v4.1-flash`. Settings remap Gemini's built-in
helper models to the router default and omit incompatible Gemini sampling
presets. Refresh helper aliases when an upstream rebase adds new helper models.
The plain CLI's static model definitions live in settings; ACP inventory and
limits come from live router metadata.

```sh
harness gemini --model grok-4.7
harness gemini skills list
GEMINI_CLI_TRUST_WORKSPACE=true harness gemini mcp list
```

Gemini also discovers `~/.agents/skills`, whose entries override same-named
skills under `~/.gemini/skills`. There are no per-harness skill copies.

`cua-driver` and `agent-desktop` MCP are enabled. The shared `agent-browser`,
`playwright`, and `browser-use` Helium entries remain excluded until their Linux
launchers are installed. See [browser MCP setup](../browser-mcp.md).

## Verification

The branch has focused core, CLI, and ACP tests covering catalog discovery,
per-model effort state, the shared menu, terminal commands and picker, footer
display, and context limits.

```sh
cd ~/dev/gemini-cli/packages/core
npx vitest run src/config/modelRouterCatalog.test.ts src/config/modelRouterEffort.test.ts src/config/config.test.ts src/core/tokenLimits.test.ts
cd ../cli
npx vitest run src/ui/components/FooterRouterEffort.test.tsx src/ui/components/Footer.test.tsx src/ui/components/EffortDialog.test.tsx src/ui/components/DialogManager.test.tsx src/ui/commands/effortCommand.test.ts src/ui/commands/modelCommand.test.ts src/services/BuiltinCommandLoader.test.ts src/acp/modelRouterConfig.test.ts src/acp/acpSession.test.ts src/acp/acpSessionManager.test.ts src/acp/acpRpcDispatcher.test.ts src/acp/acpUtils.test.ts
```

These tests require no upstream inference. A live ACP prompt and T3's actual
provider cache are also checked when updating the checkout.
