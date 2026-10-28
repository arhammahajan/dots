# Codex through model-router in T3

Select **Codex (Model Router)** in T3's provider picker. The primary Codex provider
and default model selection remain unchanged.

The active provider uses `codex-with-router`, shared home `~/.codex`, and shadow
home `~/.codex-t3/model-router`. T3 materializes shared state symlinks. Only the
placeholder API credential is stored in the shadow home; upstream credentials
remain in model-router. The wrapper supplies provider overrides on every call.

`t3-router-provider.json` is a snapshot of the installed provider entry for this
machine. Merge it into `providerInstances` in `~/.t3/userdata/settings.json` when
recreating the setup, adjusting absolute paths. Create the shadow directory and
an `auth.json` containing `{"OPENAI_API_KEY":"unused"}` with permissions 0600.
The 30 custom model entries came from the router's `/v1/models` endpoint.
Built-in Codex models outside that inventory are hidden in T3 model preferences.

Codex 0.165 needs the local router's `client_metadata` compatibility change
in `~/dev/model-router/model_router/constants.py`. It is discarded during lossy
translation with a loss diagnostic, and never becomes adapter configuration.

A live ephemeral Codex request using `deepseek-v4.1-flash` returned `ROUTER_OK`.
