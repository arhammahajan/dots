# Claude Code through model-router

`settings.json` points Anthropic Messages at `http://127.0.0.1:4000` and uses
the saved default in that file. Its picker lists every router model. The router
translates Claude's Anthropic requests for Chat, Responses, Gemini, and local upstreams.
Picker descriptions record the router's context limits and published effort sets.
`CLAUDE_CODE_MODEL_CAPABILITIES` enables effort, xhigh and max per model;
`modelSettings.maxEffortLevel` caps the selector where supported.

Claude Code 2.1.312 does not consume router limits or effort enums for custom
models. Picker entries accept no context-window setting. Its context meter uses
its own fallback; the descriptions are accurate, but do not change that meter.
Its selector cannot express sparse sets such as `high/max`, nor expose
`none/minimal` as ordinary effort choices. Enabling effort retains its built-in
low/medium/high choices. A global context override would become wrong when
switching models, so it is deliberately unset. Fixing both displays fully needs
client support for custom-model metadata.

The public token `unused` satisfies the SDK; only the router loads upstream
credentials. `claude-with-env` sources `~/.config/ai/model-router.env` and sets
`ANTHROPIC_BASE_URL` and `ANTHROPIC_AUTH_TOKEN` for Claude alone. Use that launcher's
absolute path as T3's Claude binary. Direct `claude` calls use the endpoint and
placeholder token in Claude's own `settings.json`.

Link `settings.json` to `~/.claude/settings.json`. See the [shared setup](../README.md)
for services, environment, and model compatibility. T3's model picker is configured
separately from Claude's CLI picker.
