# Cline CLI

Cline CLI 3.0.74 uses the local router's OpenAI-compatible Chat endpoint at
`http://127.0.0.1:4000/v1`, with `deepseek-v4.1-flash` as its default model.
The main router handles client compatibility directly. The router
supplies upstream credentials. The launcher supplies a fresh public affinity
token for each invocation using the router's `router-session-UUID` format.

Run `sh harnesses/cline/setup.sh` from the dots checkout to configure this
connection using `~/.config/ai/model-router.env`. An optional first argument
selects another default router model. Cline stores its settings in
`~/.cline/data/settings/providers.json`; keep this file machine-local because
Cline also stores other providers' credentials there.

Start an interactive session with `cline -i`, or supply a prompt to `cline`.
In an interactive session, type `/model` to browse and select router models.
The main router serves `/v1/models` for this discovery.
Use any model ID from the router inventory, for example:

```sh
cline -i -m local-llama
cline -i -m kimi-k3
curl -fsS http://127.0.0.1:4000/v1/models
```

The CLI normalizes the auth provider ID `openai` to `openai-compatible` in its
saved settings. Link `cline-with-router` at `~/.local/bin/cline` (ahead of npm's
binary on PATH). The launcher resolves the installed CLI through fnm's default
Node installation. Enable `[server] bearer_session_compat = true` in the
router configuration with client authentication disabled. No additional service
is required. The temporary public token is saved by Cline as its API key; it is
not an upstream credential. This provides affinity for one invocation, including
all of its turns, rather than across separately resumed invocations.
