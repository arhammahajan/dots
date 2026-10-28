# Antigravity CLI through model-router

Run `agy`, or `~/.local/bin/agy-with-env` directly. Both executable paths use
the launcher, including in existing shells without Gemini environment variables.
The default is `deepseek-v4.1-flash`; `--model <router-model-id>` selects another
model. All 30 current router models are registered in `/model`.

The CLI's supported API-key mode is configured with `modelProvider: gemini`,
`GEMINI_API_KEY=unused`, and `GOOGLE_GEMINI_BASE_URL`. See the official
[settings reference](https://antigravity.google/docs/settings?tab=cli).
No upstream credentials are stored here.

The installed CLI resolves custom models using `gemini-api://<model-id>` and
the environment base URL. The longer endpoint-bearing form in the documentation
was sent as a literal model ID by this version. Native function calling is
enabled with `TOOL_FORMATTER_TYPE_NONE`.

The launcher starts a private loopback forwarder on an ephemeral port, supplies
`x-session-id` with a fresh UUID for that CLI invocation, and stops it when the
CLI exits. Hosted OpenCode providers require this header. Resuming in another
invocation gets a new affinity ID. The launcher passes the configured default
through `--model`, because this CLI did not reliably resolve the custom default
from settings alone. Explicit `--model` overrides are preserved.

For another machine, copy `cli/settings.json` to
`~/.gemini/antigravity-cli/settings.json`, preserving existing `trustedWorkspaces`.
Move the original CLI to `~/.local/libexec/antigravity/agy`, then link
`agy-with-env` at both `~/.local/bin/agy` and `~/.local/bin/agy-with-env`.
The original binary is kept separate so the launcher can execute it without
recursion. If an installer replaces `~/.local/bin/agy`, move the new binary to
the libexec location and restore the launcher link.
The standalone desktop app and the separately configured ACP adapter retain
their existing authentication settings.

## T3 Code ACP connection

Select **Antigravity (Router ACP)** in T3 Code. Provider instance
`acpRegistry_antigravity_router` runs `~/.local/bin/antigravity-acp-with-env`.
The existing account-based Antigravity instances remain available.

The launcher resolves T3's installed adapter using
`~/.t3/tools/antigravity-acp/linux-x64/active.json`, enables its gateway auth
mode, and fetches the router inventory at startup. The gateway model allowlist
contains exactly that inventory, with `deepseek-v4.1-flash` selected first.
Built-in Gemini entries are excluded from this ACP connection's picker.

ACP configuration is isolated under
`~/.config/ai/antigravity-router-acp/antigravity-acp/settings.json`, linked to
`acp/router-settings.json`. It uses gateway auth and the public placeholder key.
The launcher runs a temporary loopback forwarder because the ACP harness adds
`sessionId` to Gemini request bodies. It removes that nonstandard field and
hashes its value into a valid `x-session-id` header, preserving conversation
affinity across turns without sending an invalid Gemini field upstream.

For another machine with T3's ACP adapter installed, link the launcher into
`~/.local/bin`, create the isolated configuration link above, then merge
`acp/t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json`.
Adjust its absolute command path. This is a separate ACP adapter: the CLI's
`customModelsConfig`, context limits, and effort settings are not inherited.
The ACP adapter's custom-model effort and context reporting remain limited.

Validated with ACP initialize, session creation, exact inventory comparison,
and a streaming hosted-model prompt returning `ROUTER_ACP_OK`. T3's live
provider catalog recognizes the enabled provider without restarting the app.

Verification: `agy models` listed all router models; headless prompts through
Gemini ingress returned `ROUTER_OK` with both the local model and the hosted
DeepSeek default. Existing live settings were backed up under
`~/.local/share/dots-backups/antigravity-router-20261028-173436/`.

## Context and reasoning

All 30 custom models have `maxTokens` set from the router's
`metadata.router.limits.context_window.value`. Local `local` uses its running
128,000-token allocation. The CLI schema defines `maxTokens` as total context,
including the response; it does not expose a separate custom-model output limit.
`supportsThinking` follows the router's explicit reasoning support flag.

Per-model effort enums are **not supported for custom models** in this CLI.
The schema has no effort-list or default-effort field. Verified with the installed
binary: `agy --model deepseek-v4.1-flash -p /effort --output-format json` returns
`adjustable: false`, and adding `--effort max` fails as unsupported. A thinking
capability flag does not enable those controls. Reasoning uses the destination's
default behavior; no numeric budget or approximate effort mapping is forced.
See the [router capability reference](model-capabilities.md) for each model's
actual context and effort vocabulary. Native per-model effort controls require
Antigravity support for custom-model effort variants.
