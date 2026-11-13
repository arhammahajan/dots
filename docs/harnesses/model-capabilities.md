# Context and effort audit

These client capability notes were checked on 2026-10-28 against the router's
30-model inventory, OpenCode catalog facts, and llama.cpp. Static limits are
snapshots: the local server's context allocation can change. Use `/v1/models`
for its current runtime limit instead of the model's theoretical maximum.

| Harness | Context source | Effort controls |
| --- | --- | --- |
| Claude Code 2.1.312 | Static picker descriptions show the exact router limit; the built-in meter cannot use custom per-model limits | Per-model capability flags and maximum effort; built-in low/medium/high choices cannot represent sparse enums exactly |
| OpenCode 2.2.0 | Static `providers.*.models.*.limit.context` | `capabilities.reasoning.effort` and `toggle`, using router facts |
| Qwen Code | Static `modelProviders.*[].generationConfig.contextWindowSize` | Explicit `capabilities.reasoning` profile, effort list and default; its UI has no minimal tier |
| Kimi Code | Static `models.*.max_context_size` | Existing `support_efforts` and `off_effort` already match the router |
| MCode | Static `custom_provider.*.models.*.limit.context` | Existing `thinking.effortOptions`; thinking-off is now offered only when published as a toggle or `none` effort |
| Pi 1.4.0 | Live `/v1/models` metadata | The existing extension filters efforts on Chat, Responses, Anthropic and Gemini; unknown enums hide effort controls |
| Antigravity CLI | Static custom-model `maxTokens` from router facts for all 30 models | Thinking support flag only; custom models report `adjustable: false` and reject `--effort`; see [details](antigravity/README.md#context-and-reasoning) |
| Muse 1.4.13 | Live `/muse-code/models` metadata | Router now supplies `metadata.muse-code.variants` and a valid default; no external adapter |
| fx effort fork | Static `providers.model-router.model_metadata.*.context_window` | Native menus use the router's exact effort lists for 22 models and send Chat `reasoning_effort`; unknown enums keep destination defaults; see [details](fx/README.md#reasoning-effort) |

No new discovery plugin or config generator is needed. Pi's existing provider
registration extension remains necessary because Pi does not register arbitrary
providers from `/v1/models` by itself. Muse compatibility lives in
`model_router/discovery.py` in the router checkout.

Claude is a partial fix, not a corrected context meter. Its installed picker
schema only accepts model, label, description and behavesAs. Gateway discovery
does not consume context limits or effort enums. Its model-retrieval path is
disabled in this build. Borrowing a Claude identity would give custom models
invented limits; a global context override would become incorrect after model
switching. Neither is configured. Full support needs a client change.

Qwen exposes low/medium/high/xhigh/max, so local's minimal tier cannot appear
there. Off remains a separate thinking control, not an effort entry. The
Anthropic profile uses adaptive thinking and output effort, which the router
may approximate on translated routes under its configured lossy policy.

Where upstream effort enums are unknown, no exact enum was invented in static
configuration or router metadata. Some clients can still show built-in defaults
for these models, notably Muse. Pi hides those controls instead. Some static
configs use a 16,384-token fallback for unknown local output limits; fx omits
the limit. Neither represents a llama.cpp-reported maximum.

Kimi and MCode save regular active files and can replace symlinks. Kimi's active
and tracked limits/efforts were both checked. MCode's active metadata was updated
without replacing its selected model or other preferences. Restart clients to
load settings and refresh their model catalogs.

The original audit used parsing, Qwen's reasoning validator, OpenCode's model
listing, and Muse's read-only catalog. fx's later validation includes native
tests and live requests; see its setup notes.
