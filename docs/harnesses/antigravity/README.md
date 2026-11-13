# Antigravity CLI through model-router

Run `harness antigravity`, or the interactive Zsh alias `agy`. The central
launcher works without inherited Gemini environment variables.
Use `--model <router-model-id>` to select a router model. All 30 current router
models are registered in `/model`.

The CLI's supported API-key mode is configured with `modelProvider: gemini`,
`GEMINI_API_KEY=router-session-<UUIDv4>`, and `GOOGLE_GEMINI_BASE_URL`. See the official
[settings reference](https://antigravity.google/docs/settings?tab=cli).
No upstream credentials are stored here.

The installed CLI resolves custom models using `gemini-api://<model-id>` and
the environment base URL. The longer endpoint-bearing form in the documentation
was sent as a literal model ID by this version. Native function calling is
enabled with `TOOL_FORMATTER_TYPE_NONE`.

The launcher points `GOOGLE_GEMINI_BASE_URL` directly at model-router and supplies
a fresh public session token through `GEMINI_API_KEY` for each invocation.
Enable `[server] bearer_session_compat = true` on the router with client
authentication disabled. The router consumes the token as session identity and
sets the upstream session header; it always uses its own upstream credentials.
Resuming in another invocation gets a new affinity ID. There is no local HTTP
proxy process. The launcher passes the configured default
through `--model`, because this CLI did not reliably resolve the custom default
from settings alone. Explicit `--model` overrides are preserved.

For another machine, copy `cli/settings.json` to
`~/.gemini/antigravity-cli/settings.json`, preserving existing `trustedWorkspaces`.
Keep the original CLI at `~/.local/libexec/antigravity/agy`. The central
launcher executes that binary directly. If an
installer writes a new `~/.local/bin/agy`, move it to the libexec location.
The standalone desktop app and the separately configured ACP adapter retain
their existing authentication settings.

## T3 Code ACP connection

Select **Antigravity (Router ACP)** in T3 Code. Provider instance
`acpRegistry_antigravity_router` runs `~/.local/bin/harness` with
`commandArgs: ["antigravity-acp"]`.
The existing account-based Antigravity instances remain available.

The launcher resolves T3's installed adapter using
`~/.t3/tools/antigravity-acp/linux-x64/active.json`, enables its gateway auth
mode, and fetches the router inventory at startup. The gateway model allowlist
contains exactly that inventory.
Built-in Gemini entries are excluded from this ACP connection's picker.

ACP configuration is isolated under
`~/.config/ai/antigravity-router-acp/antigravity-acp/settings.json`, linked to
`acp/router-settings.json`. It uses gateway auth and the public placeholder key.
The launcher points `AGY_LLM_GATEWAY_URL` directly at model-router and sets
`AGY_LLM_GATEWAY_API_KEY` to a fresh public session token. With
`bearer_session_compat` enabled, the router consumes the ACP adapter's
nonstandard `sessionId` body field and hashes its JSON-encoded string into an
`agy-acp-<SHA-256>` identity, preserving the former proxy's identity mapping.
The body field is removed before translation or native upstream forwarding.
Native session headers take priority; requests without a conversation ID fall
back to the per-launch token. No forwarding server or Python supervisor runs
alongside the adapter. The central launcher reads the installed release and
router inventory, then executes the adapter directly.

The installed adapter reuses its gateway `sessionId` across multiple ACP
conversations in one process. This behavior is unchanged from the former proxy;
the router uses the identity supplied by the adapter.

For another machine with T3's ACP adapter installed, install the central
`harness` command, create the isolated configuration link above, then merge
`acp/t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json`.
Adjust its absolute command path. This is a separate ACP adapter: the CLI's
`customModelsConfig`, context limits, and effort settings are not inherited.
The ACP adapter's custom-model effort and context reporting remain limited.

Validated with ACP initialize, session creation, exact inventory comparison,
and a streaming hosted-model prompt returning `ROUTER_ACP_OK`. T3's live
provider catalog recognizes the enabled provider without restarting the app.

Verification: `agy models` listed all router models; headless prompts through
Gemini ingress returned `ROUTER_OK` with both the local model and the hosted
DeepSeek model. Existing live settings were backed up under
`~/.local/share/dots-backups/antigravity-router-20261028-173436/`.

## Context and reasoning

All 30 custom models have `maxTokens` set from the router's
`metadata.router.limits.context_window.value`. Refresh local `local-llama` when
its server context allocation changes. The CLI schema defines `maxTokens` as
total context, including the response; it does not expose a separate
custom-model output limit.
`supportsThinking` follows the router's explicit reasoning support flag.
`supportsImages` follows the router's explicit input-image support flag: enabled
for the 21 image-capable models in the current inventory and disabled for the
remaining models. The active CLI settings carry the same flags.

Per-model effort enums are **not supported for custom models** in this CLI.
The schema has no effort-list or default-effort field. Verified with the installed
binary: `agy --model deepseek-v4.1-flash -p /effort --output-format json` returns
`adjustable: false`, and adding `--effort max` fails as unsupported. A thinking
capability flag does not enable those controls. Reasoning uses the destination's
default behavior; no numeric budget or approximate effort mapping is forced.
See the [router capability reference](model-capabilities.md) for each model's
actual context and effort vocabulary. Native per-model effort controls require
Antigravity support for custom-model effort variants.
