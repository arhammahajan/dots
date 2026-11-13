# Kilo Code CLI

Kilo CLI 7.8.18 connects directly to the local model-router at
`http://127.0.0.1:4000/v1`. Its default is `model-router/deepseek-v4.1-flash`,
matching the other harnesses. Upstream credentials stay in model-router;
Kilo uses the shared public placeholder from `ai/model-router.env`.

The global config is linked at `~/.config/kilo/kilo.json`. It includes all 27
models advertised by `/v1/models` during setup on 2026-11-04, with router-provided
context/output limits, input/output modalities, and reasoning effort variants.
Chat models and Gemini use the Chat adapter, Responses models use the OpenAI
adapter, and Anthropic models use the Anthropic adapter. Only these three
router providers are enabled. No Kilo account is needed for these connections.

```sh
kilo
kilo models
kilo -m model-router/kimi-k3
kilo -m model-router-responses/gpt-6-luna
kilo -m model-router-anthropic/claude-haiku-5-5
kilo run --variant low 'Explain this project'
```

Use `/models` inside Kilo to switch models. The catalog is a static snapshot;
refresh the tracked entries from `/v1/models` when router availability or model
capabilities change. The local llama.cpp model was not advertised during setup;
add it when discovery reports it, using its runtime limits. Where the router
does not publish effort levels, speculative built-in effort choices are disabled.

The launcher sources `~/.config/ai/model-router.env`, then executes
`~/.kilo/bin/kilo`. Kilo supplies `X-Session-Id` per conversation, which the
router recognizes for upstream affinity, including resumed sessions. No
install-wide session token or extra compatibility service is required.

## Installation and links

Install the standalone binary using the [official installer](https://kilo.ai/cli/install)
without changing shell startup files:

```sh
curl -fsSL https://kilo.ai/cli/install -o /tmp/kilo-install.sh
bash /tmp/kilo-install.sh --no-modify-path
mkdir -p ~/.config/kilo ~/.local/bin
ln -s "$PWD/harnesses/kilo/kilo.json" ~/.config/kilo/kilo.json
# Install the shared harness command as described in ../README.md.
harness kilo --help
```

Run the link commands from the dots checkout; back up existing destinations
first. `~/.local/bin` is already on PATH through the shared Zsh configuration.
Update with `kilo upgrade`. The installed stable release uses the `provider`
configuration schema; current website examples may describe a newer schema.

Validate with `kilo --version`, `kilo config check`, and `kilo models`.
Model usage costs are not configured, so Kilo's displayed cost is not an
upstream billing estimate.

## T3 Code ACP provider

Select **Kilo Code** in T3 Code. `t3-provider.json` records the official
[ACP Registry](https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json)
entry (`agentId: kilo`). Its executable override is `~/.local/bin/harness` with
provider environment `HARNESS_AGENT=kilo`. The registry supplies `acp`, so T3
runs the equivalent of `harness kilo acp`. This uses the same official binary
at `~/.kilo/bin/kilo` as the terminal launcher.

The central launcher loads `MODEL_ROUTER_URL` and `MODEL_ROUTER_API_KEY` from
the shared `~/.config/ai/model-router.env` (respecting `XDG_CONFIG_HOME`), with
defaults of `http://127.0.0.1:4000` and the public placeholder `unused`. T3 stores
only the agent selector, with no duplicated router URL or key. ACP reads the same global
`~/.config/kilo/kilo.json` as the CLI, including the default model and router
catalog. Kilo reports its models and session configuration over ACP.
T3's initial catalog probe can show the default model's effort choices against
other models; Kilo updates the session's effort choices after model selection.

Setup validation covered the official binary's ACP initialization, all 27
router models in its model configuration option, and a live `session/prompt`
through the router. T3 reports the provider ready with no constraints.

For another machine, install the Kilo entry from T3's ACP Registry and merge
`t3-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json`.
Install the standalone Kilo binary and shared `harness` command, adjust the
absolute command path, and link the Kilo global config and shared router
environment as described above.
