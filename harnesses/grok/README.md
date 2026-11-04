# Grok Build CLI

`config.toml` mirrors this computer's Grok Build settings: Grok 4.7 with low
reasoning effort, the official xAI plugin marketplace, and terminal preferences.

To link the config on another machine, back up any existing config first,
then run from the dotfiles checkout:

```sh
grok_dir="${GROK_HOME:-$HOME/.grok}"
mkdir -p "$grok_dir"
ln -s "$PWD/harnesses/grok/config.toml" "$grok_dir/config.toml"
```

Only link `config.toml`. OAuth credentials, sessions, caches, installed plugins,
and CLI binaries stay outside the repo.

## Second provider through model-router in T3

Select **Grok Build (Model Router)** in T3's provider picker. Instance
`grok_router` uses `grok-with-router`, with its own home at
`~/.grok-t3/model-router`. Link `router-config.toml` there as `config.toml`.
The existing **Grok** provider continues using `~/.grok` and its normal login.

The launcher sources `~/.config/ai/model-router.env`, sets the custom model
endpoint to `${MODEL_ROUTER_URL}/v1`, and uses the public placeholder API key.
It calls the installed `~/.local/bin/grok`; Python 3 generates a fresh session
identity for each invocation. Per-model `env_http_headers` send that identity in
`x-session-id`, which the router uses for upstream affinity (required by
OpenCode Go). Upstream credentials stay in model-router.

Grok discovers available models from the router's `/v1/models`. The 27 model
overrides in `router-config.toml` record the inventory on 2026-11-04, including
context and output limits, image support, and exact reasoning-effort lists.
Refresh these fields together when the router inventory changes. Models use
their native Chat, Responses, or Anthropic protocol; Gemini uses translated Chat.
Grok-hosted backend search is disabled for these custom endpoints.
The default model is `deepseek-v4.1-flash` with low effort. This isolated home
keeps its own sessions, credentials, caches, and plugins.

To recreate the installed setup, start the shared model-router service, then run
from this checkout (back up an existing router config before linking):

```sh
mkdir -p "$HOME/.grok-t3/model-router"
chmod 700 "$HOME/.grok-t3/model-router"
ln -s "$PWD/harnesses/grok/router-config.toml" "$HOME/.grok-t3/model-router/config.toml"
chmod +x harnesses/grok/grok-with-router
harnesses/grok/grok-with-router models
```

Merge `t3-router-provider.json` into `providerInstances` in
`~/.t3/userdata/settings.json`, adjusting the absolute binary path for that
machine. It includes custom model entries for the initial T3 snapshot; ACP
discovery supplies the model limits and reasoning controls. No Grok login or
copied OAuth credential is required for the router provider.

Verified with Grok Build 1.0.70: T3 discovered all 27 models with the configured
context limits and effort menus. A live ACP request through DeepSeek V4.1 Flash
returned `ROUTER_OK`; a second request executed `read_file` and returned the
fixture contents. Session summaries also use a configured router model.
