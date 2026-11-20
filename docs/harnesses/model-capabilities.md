# Context and effort audit

The initial audit on 2026-10-28 checked the `model-router` 30-model inventory, OpenCode catalog facts, and `llama.cpp`. The tracked static client configs under `harnesses/` currently hold 28 models. Static context limits are snapshots because the local `llama.cpp` server can change its context allocation. Query `/v1/models` for the current runtime limit instead of relying on a model's theoretical maximum.

## Client capability matrix

| Client | Context source | Effort controls |
| --- | --- | --- |
| Claude Code 2.1.312 | Static `modelPicker.options[].description` text shows the exact router limit. The built-in context meter cannot read custom per-model limits. | Per-model flags in `CLAUDE_CODE_MODEL_CAPABILITIES` and `modelSettings.<model>.maxEffortLevel`. The built-in `low`, `medium`, and `high` options cannot represent sparse enums exactly. |
| OpenCode 2.2.0 | Static `providers.*.models.*.limit.context` in `harnesses/opencode/opencode.json`. | `capabilities.reasoning.effort` and `capabilities.reasoning.toggle` come from router facts. |
| Qwen Code | Static `modelProviders.*[].generationConfig.contextWindowSize` in `harnesses/qwen/settings.json`. | Explicit `capabilities.reasoning` profile, `efforts` list, and `defaultEffort`. The Qwen UI has no `minimal` tier. |
| Kimi Code | Static `models.*.max_context_size` in `harnesses/kimi-code/config.toml`. | `support_efforts` and `off_effort` match the router inventory. |
| MCode | Static `custom_provider.*.models.*.limit.context` in `harnesses/mcode/config.yaml`. | `thinking.effortOptions` and `thinking_config.mode`. MCode offers thinking-off only when the router publishes a toggle or a `none` effort. |
| Pi | Live `/v1/models` metadata fetched by the router extension. | The extension maps published effort lists across Chat, Responses, Anthropic, and Gemini. Models without a published effort list hide effort controls. |
| Antigravity CLI | Static `customModelsConfig.customModels.*.maxTokens` from router facts in `harnesses/antigravity/cli/settings.json`. | `supportsThinking` flag only. Custom models report `adjustable: false` and reject `--effort`. See [Antigravity context and reasoning](antigravity/README.md#context-and-reasoning). |
| Gemini CLI source fork | Live router context and output limits in native token checks and T3's context meter. Audited on 2026-11-14. | Terminal `/effort` and `/thinking` use exact per-model levels. T3 uses a combined menu with per-model validation, and other ACP clients keep exact per-model menus. See [Gemini terminal model and effort selection](gemini/README.md#terminal-model-and-effort-selection). |
| Muse 1.4.13 | Live `/muse-code/models` metadata. | The router supplies `metadata.muse-code.variants` and a valid default without an external adapter. |
| fx effort fork | Static `providers.model-router.model_metadata.*.context_window` in `harnesses/fx/settings.json`. | Native menus read `reasoning_efforts` for the 22 models with published enums and send Chat `reasoning_effort`. Models with empty `reasoning_efforts` keep destination defaults. See [fx reasoning effort](fx/README.md#reasoning-effort). |

## Live discovery and source forks

Pi still requires its provider registration extension at `~/dev/model-router/integrations/pi/model-router.ts`. `harnesses/setup.sh` links that file to `~/.pi/agent/extensions/model-router.ts` because Pi does not register arbitrary providers from `/v1/models` on its own.

Muse compatibility lives in `internal/server/read_listing.go` in the `model-router` checkout. Its `handleMuseModels` handler serves `/muse-code/models` and `/v1/muse-code/models` with the variant metadata Muse expects.

The maintained Gemini CLI source fork provides live model discovery, per-model effort controls, and live context limits. Stock Gemini CLI does not expose those controls for custom router models.

## Static client constraints

Claude Code's configuration is a partial workaround rather than a fixed context meter. The installed picker schema accepts only `model`, `label`, `description`, and `behavesAs`. Gateway discovery does not consume context limits or effort enums, and model retrieval is disabled in this build. Borrowing a built-in Claude identity through `behavesAs` would assign invented limits to custom models. Setting a global context override would become wrong after switching models. `harnesses/claude/settings.json` configures neither option, and full support requires a client change.

Qwen Code exposes `low`, `medium`, `high`, `xhigh`, and `max`, so the local model's `minimal` tier cannot appear in its picker. Turning thinking off stays a separate control instead of an effort entry. The `anthropic-adaptive` profile sends adaptive thinking and output effort, which `model-router` may approximate on translated routes under its configured lossy policy.

## Unknown effort lists and local output limits

When an upstream model's effort enum is unknown, neither the static client configs nor `model-router` metadata invents one. Some clients still show built-in defaults for those models, notably Muse. Pi hides effort controls for those models by setting `reasoning` to `false`.

OpenCode, Kimi Code, and MCode set a 16,384-token fallback for the unknown `local-llama` output limit. `harnesses/fx/settings.json` omits the output limit for `local-llama`. Neither choice reflects a maximum reported by `llama.cpp`.

## Active config files and verification

Kimi Code and MCode save regular files when updating settings and can overwrite symlinks created by `harnesses/setup.sh`. After inventory changes, check the tracked files at `harnesses/kimi-code/config.toml` and `harnesses/mcode/config.yaml` along with the active copies at `~/.kimi-code/config.toml` and `~/.minimax/config.yaml`. When updating MCode's active metadata, preserve its selected model and other local preferences. Restart running clients to load updated settings and refresh their model catalogs.

The original audit checked client configs with schema parsing, Qwen's reasoning validator, OpenCode's model listing, and Muse's read-only catalog. Later validation for `fx` added native fork tests and live requests. See [fx verification](fx/README.md#verification) for those commands.
