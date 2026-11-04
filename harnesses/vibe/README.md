# Mistral Vibe through model-router

Install with `uv tool install mistral-vibe`. Link `config.toml` to
`~/.vibe/config.toml`, and `vibe-with-router` to both `~/.local/bin/vibe` and
`~/.local/bin/vibe-with-router`. The wrapper runs the uv-managed executable and
creates a public per-invocation bearer session token for the router's
`bearer_session_compat` mode; upstream credentials stay in the router. The
router must be running on `127.0.0.1:4000`. Vibe's configured OpenAI-compatible
Chat provider uses the router for every model, including models whose upstream
protocol differs. A resumed session gets a new router affinity token.

The default is `deepseek-v4.1-flash`. Select another model with `/model` in Vibe
or `vibe --model MODEL --prompt '...'`. The static catalog is a snapshot of the
router inventory; update its model entries and capabilities when the inventory
changes. Vibe also includes its own built-in Mistral model, which is not part of
the router configuration. Telemetry and Vibe's automatic updater are disabled.

Update with `uv tool upgrade mistral-vibe`, then restore the `~/.local/bin/vibe`
link if uv replaces it. Vibe may rewrite its configuration when settings are
changed; preserve the tracked config when updating the active file. See the
[shared setup](../README.md) for router service and client settings.

## Context and reasoning

Each entry records the router's published context window, output limit, and
image support. The 21 models with a published effort enum also list its exact
choices in `thinking_levels`. Vibe calls the router's `none` effort `off` and
sends it as `none` on Chat requests. Where the router publishes a separate
thinking toggle but no `none` effort, that toggle is not represented by this Chat
provider. The remaining six models have unknown effort enums; Vibe may display
its own generic choices for them, which are not verified model capabilities.
The router has not published default effort values; Vibe chooses its own unless
one is explicitly set in the config. Restart Vibe after updating the catalog.

## T3 Code ACP

Select **Mistral Vibe** in T3 Code. The ACP Registry provider uses
`mistral-vibe` with executable override `~/.local/bin/vibe-acp-with-router`,
which launches Vibe's official `vibe-acp`
executable with a fresh public router affinity token. It reads the same Vibe
configuration as the CLI. ACP reports models and thinking options; changing the
model in a session updates the available thinking choices. T3's initial
provider probe may show the default model's thinking choices against other
models until the session changes model. Vibe also advertises its built-in
Mistral model, which uses Mistral credentials rather than the router.

For another machine, link the ACP launcher into `~/.local/bin`, install Vibe as
above, and merge `t3-provider.json` into `providerInstances` in
`~/.t3/userdata/settings.json`, adjusting the absolute path. Vibe's installer
may replace the CLI `vibe` link during upgrades, but this ACP wrapper targets
the uv-managed executable directly. ACP processes each receive a fresh affinity
token, including when a session is resumed in a new process.
