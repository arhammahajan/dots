# Gemini CLI through model-router

In an interactive Zsh session, `.zshrc` unaliases `gemini`, so running `gemini` directly starts the global installation. Run `harness gemini` to launch Gemini CLI through `model-router`. Both `harness gemini` and T3 Code execute `packages/cli/dist/index.js` from the maintained source checkout at `~/dev/gemini-cli` on branch `router-acp`. The `harness` launcher runs that entry file with `node` and prepends `~/.local/share/fnm/aliases/default/bin` to `PATH` when present. Set `GEMINI_CLI_REPO` to point the launcher at another checkout.

## Build and maintain the checkout

The checkout at `~/dev/gemini-cli` uses `upstream` for Google's repository and `origin` for the fork [arhammahajan/gemini-cli](https://github.com/arhammahajan/gemini-cli). The local `router-acp` branch tracks `origin/router-acp`, which is the fork's default branch on GitHub. All router changes live as regular commits on `router-acp`, so you can rebase onto `upstream/main`.

To pull upstream changes and rebuild:

```sh
cd ~/dev/gemini-cli
git fetch upstream
git rebase upstream/main
~/dev/dots/harnesses/gemini/build.sh --install
```

After reviewing and testing the rebase, push it with `git push --force-with-lease origin router-acp`.

When passed `--install` or when `node_modules` is absent, `harnesses/gemini/build.sh` runs `npm ci --ignore-scripts`. It then runs `npm run generate` and builds `@google/gemini-cli-core`, `@google/gemini-cli-devtools`, and `@google/gemini-cli` in order. Without `--install`, the script reuses the existing `node_modules` directory. This machine builds with Node 24.22.1. The launcher runs the rebuilt entry file directly without global npm installation or `npm link`.

To set up a fresh machine:

```sh
git clone --branch router-acp https://github.com/arhammahajan/gemini-cli.git ~/dev/gemini-cli
cd ~/dev/gemini-cli
git remote add upstream https://github.com/google-gemini/gemini-cli.git
git fetch upstream
~/dev/dots/harnesses/gemini/build.sh --install
~/dev/dots/harnesses/setup.sh
```

The `router-acp` branch modifies four areas of the checkout:

| Path | Responsibility |
| --- | --- |
| `packages/core/src/config/modelRouterCatalog.ts` | Fetches and parses router model metadata from `/v1/models` |
| `packages/core/src/core/tokenLimits.ts` | Applies published router context and output token limits |
| `packages/cli/src/ui/commands/effortCommand.tsx` and `packages/cli/src/ui/components/EffortDialog.tsx` | Implements `/effort`, `/thinking`, and the interactive radio-list effort picker |
| `packages/cli/src/acp/modelRouterConfig.ts` | Implements ACP model and effort menus, protocol errors, and T3's shared effort opt-in |

Core `Config` holds per-model effort state, validation, and request header injection for both terminal and ACP sessions. A separate commit adds footer effort display, and a small `DialogManager` fix keeps custom pickers visible in screen-reader mode. You can drop the `DialogManager` fix once upstream includes it. This setup requires no bundle rewrites, loader hooks, synthetic effort model variants, extra HTTP proxies, or T3 source patches.

When upstream modifies ACP APIs, review and resolve rebase conflicts in `packages/cli/src/acp/modelRouterConfig.ts` and keep T3 compatibility behavior inside that module. CLI tests import the compiled core package, so rebuild `@google/gemini-cli-core` after editing core files and before running CLI tests.

## Endpoint and authentication

When you run `harness gemini`, the launcher sets three environment variables for the child process:

| Variable | Value |
| --- | --- |
| `GEMINI_MODEL_ROUTER` | `1` |
| `GOOGLE_GEMINI_BASE_URL` | `MODEL_ROUTER_URL`, defaulting to `http://127.0.0.1:4000` without `/v1` |
| `GEMINI_API_KEY` | A fresh `router-session-<UUIDv4>` token per invocation |

`harnesses/gemini/settings.json` sets `security.auth.selectedType` to `"gemini-api-key"`, so Gemini CLI does not require Google account sign-in.

`model-router` requires `[server] auth_mode = "none"` and `bearer_session_compat = true`. The router uses the public session token for request affinity and supplies its own upstream credentials. Provider environment variables remain scoped to the launched Gemini process, and the launcher never copies upstream credentials into Gemini files. Resuming a session in a new invocation generates a new router session token.

Router behavior stays off when `GEMINI_MODEL_ROUTER=1` is unset. Passing `--no-router` to `harness gemini` or running the source entry directly allows normal Google account or API-key authentication.

## Terminal model and effort selection

Launch the routed terminal UI with `harness gemini`. Inside the session, switch models and reasoning effort with slash commands:

```text
/model set deepseek-v4.1-flash
/effort
/thinking
/effort auto
```

`/effort` and `/thinking` open an interactive picker matching `/model`. The list contains `Upstream default (auto)` plus the exact effort levels published for the active model. The current choice starts marked and highlighted. Move with the arrow keys, press Enter to apply, or press Escape to cancel without changes.

Direct arguments also work via `/effort <level>` and `/thinking <level>`. Tab completion offers `auto` and only the active model's published values, including sparse vocabularies and values such as `xhigh` or `max` when present. Setting `auto` clears the override and restores the upstream default.

The footer appends the active effort in brackets after the model name, such as `deepseek-v4.1-flash [max]` or `deepseek-v4.1-flash [auto]`. It updates after picker selections, direct effort commands, and model switches. On each model switch, the footer restores the effort stored for the displayed model.

Gemini remembers effort choices per model in memory for the current process, including switches through `/model` and its dialog. It does not save effort choices to disk or share them across terminal or T3 sessions. Unsupported values return an error and keep the previous valid effort. Models without a published effort vocabulary report that fact and accept only `auto`. Gemini does not invent tiers or `off` aliases, and `/thinking none` works only when the model publishes `none`.

These commands control reasoning effort only. Thought-text display remains a separate Gemini setting. The commands register only when `GEMINI_MODEL_ROUTER=1` is set. Terminal mode always shows exact per-model options and ignores T3's shared ACP menu setting.

## T3 models, effort, and context windows

Select Gemini CLI (Model Router) in T3 Code. The `acpRegistry_gemini` provider stays an official ACP registry provider with `agentId: "gemini"`, the official registry icon, and the registry-supplied `--acp` argument. Its `commandPath` overrides the executable with `~/.local/bin/harness`, and its environment sets `HARNESS_AGENT=gemini`.

At startup, the source build queries `/v1/models` on `model-router`. T3 lists one entry per returned router model using its display name, such as `DeepSeek V4.1 Flash`. T3 displays context capacity in its context meter and exposes reasoning effort in a separate control.

The T3 provider sets `GEMINI_ACP_SHARED_EFFORT_CONTROL=1`. With this flag enabled, the Effort ACP control lists `Upstream default` plus the combined set of published effort values across the router catalog. T3 currently probes only the default model and scopes controls categorized as `thought_level` or `model_config` to that model. Leaving Effort uncategorized under this opt-in makes T3 show the control on every model before its first session.

Because the shared menu includes values that some models do not accept, Gemini checks every selection against the active model's exact published vocabulary. An unsupported choice returns an error listing the valid levels and blocks prompts until you pick a valid effort or switch models. That check prevents clients that ignore a failed set-option call from generating at a different effort. Models without a published effort vocabulary accept only `Upstream default`.

`session/set_config_option` updates model and effort independently and stores effort per model. Without `GEMINI_ACP_SHARED_EFFORT_CONTROL=1`, other ACP clients receive the active model's exact menu under category `thought_level`, and models without a published vocabulary omit the effort control. Switching models publishes updated options in both modes. Gemini never renames or clamps effort values.

Selecting `Upstream default` clears the effort override. Explicit values travel in the `x-model-router-reasoning-effort` HTTP header, including `xhigh` and `max`, which Gemini's built-in thinking-level enum cannot represent. `model-router` validates that header against the model's metadata and strips it before forwarding. Existing strict and lossy translation policies and `x-model-router-loss` reporting continue to apply across protocols.

Published context and output limits feed Gemini's token-limit functions. ACP `usage_update.size` reports the active model's context size and updates after model switches. `harnesses/t3code/client-settings.json` enables T3's context meter with `"contextWindowMeterEnabled": true`. Models without published limits retain stock Gemini fallback values.

The T3 provider sets `GEMINI_CLI_TRUST_WORKSPACE=true` for its process. T3 chooses the workspace and permission mode, and Gemini must trust that workspace before T3 can switch into `autoEdit` or `yolo`. Without this variable, discovery probes succeed, but full-access startup fails at `session/set_mode`.

`harnesses/gemini/t3-provider.json` stores the provider template. On another machine, update its absolute `commandPath` and merge the entry into `providerInstances` in `harnesses/t3code/settings.json`, which `harnesses/setup.sh` links to `~/.t3/userdata/settings.json`. Increment `GEMINI_ROUTER_ACP_REVISION` in the provider environment after rebuilding changed code so T3 invalidates its discovery cache. Running agent processes need a new session to load rebuilt code.

## Shared settings, skills, and MCP

`harnesses/setup.sh` creates two symlinks for Gemini CLI and cleans up one obsolete link:

| Source in repo | Target path |
| --- | --- |
| `harnesses/gemini/settings.json` | `~/.gemini/settings.json` |
| `harnesses/shared/skills` | `~/.gemini/skills` |

If `~/.config/ai/gemini/router-acp.mjs` is a symlink pointing to `harnesses/gemini/router-acp.mjs`, `harnesses/setup.sh` removes it. When `harnesses/setup.sh` replaces an existing file or symlink, it moves the original path to `~/.local/share/dots-backups/`.

`harnesses/gemini/settings.json` sets the default model to `deepseek-v4.1-flash` and enables `experimental.dynamicModelConfiguration`, `skills.enabled`, and `general.enableNotifications`. It remaps Gemini's built-in helper models and aliases to `deepseek-v4.1-flash`, clears `chat-base` sampling presets, and hides stock Gemini entries from the static model list. Refresh those helper aliases when an upstream rebase adds new helper models. Static `modelDefinitions` in settings populate the plain CLI picker, while ACP model lists and token limits come from live router metadata.

`harnesses/gemini/settings.json` registers Spaces lifecycle hooks for `SessionStart`, `SessionEnd`, `BeforeAgent`, `BeforeTool`, `AfterTool`, `AfterAgent`, and `Notification`, and each hook calls `spaces hook gemini <Event>`. The file also configures the `agent-browser` MCP server with portable `${HOME}` paths. See [browser MCP setup](../browser-mcp.md) for launcher and Helium CDP details.

Common CLI checks:

```sh
harness gemini --model grok-4.7
harness gemini skills list
GEMINI_CLI_TRUST_WORKSPACE=true harness gemini mcp list
```

Gemini CLI also reads `~/.agents/skills`, where entries override same-named skills in `~/.gemini/skills`. All shared skills live in `harnesses/shared/skills` with no per-tool copies.

## Verification

The `router-acp` branch includes focused Vitest suites across core, CLI, and ACP for catalog discovery, per-model effort state, the shared menu, terminal commands and picker, footer display, and context limits:

```sh
cd ~/dev/gemini-cli/packages/core
npx vitest run src/config/modelRouterCatalog.test.ts src/config/modelRouterEffort.test.ts src/config/config.test.ts src/core/tokenLimits.test.ts
cd ../cli
npx vitest run src/ui/components/FooterRouterEffort.test.tsx src/ui/components/Footer.test.tsx src/ui/components/EffortDialog.test.tsx src/ui/components/DialogManager.test.tsx src/ui/commands/effortCommand.test.ts src/ui/commands/modelCommand.test.ts src/services/BuiltinCommandLoader.test.ts src/acp/modelRouterConfig.test.ts src/acp/acpSession.test.ts src/acp/acpSessionManager.test.ts src/acp/acpRpcDispatcher.test.ts src/acp/acpUtils.test.ts
```

These tests require no upstream inference. When updating the checkout, also verify a live ACP prompt and T3's provider cache.
