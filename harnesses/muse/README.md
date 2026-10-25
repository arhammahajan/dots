# Muse Code with OpenCode Go

Run `muse`, then use `/models` to choose any model in the current OpenCode Go
inventory. You can also select one at startup:

```zsh
muse --model glm-5.3-flash
muse --model qwen3.8-max
muse --model gemini-4-argon
muse exec --model kimi-k3 "Explain this project"
```

The saved default is Muse Spark 1.3 Contributor; it does not restrict the picker.
The bridge loads the inventory from
`https://opencode.ai/inference/go/v1/models`, refreshes it every five minutes on
demand, and keeps a cached copy for temporary outages. Model context/output
limits come from the OpenCode Go entries at `https://models.dev/api.json` when
available. Models without that metadata get conservative fallback limits.

Muse's provider speaks Responses. OpenCode Go handles Responses for most models;
some models require Anthropic's or Google's protocol. `opencode-go-bridge.py`
passes Responses requests through unchanged and uses LiteLLM for the other
protocols, tools, and multimodal conversion. Routing follows OpenCode Go's
model metadata, with a Chat Completions fallback when a model explicitly rejects
Responses. It also adapts inventory discovery to Muse's catalog route.
The bridge listens only on `127.0.0.1:52187` and starts as a user systemd service.

`settings.json`, linked at `~/.config/muse/settings.json`, pins `endpoint_transport.base_url` to
`http://127.0.0.1:52187/v1`, with `auth` set to `bearer`. It has no fixed
`model_catalog`, so new launches discover all current Go models.

The Zsh setup reads the existing `~/.config/ai/credentials.env`, maps
`OPENCODE_API_KEY` to `META_API_KEY`, and adds a conversation routing header.
The bridge forwards the key provided by Muse; it does not store a separate key.
For callers outside Zsh, export `META_API_KEY` from the existing credential
environment; the bridge can derive the session header from Muse's cache key.

```sh
# Install the isolated adapter environment.
uv venv ~/.local/share/muse-opencode-go/venv
uv pip install --python ~/.local/share/muse-opencode-go/venv/bin/python \
  -r ~/dev/dots/harnesses/muse/requirements.txt

# Link the settings and service after backing up any existing settings.
mkdir -p ~/.config/muse ~/.config/systemd/user
ln -s ~/dev/dots/harnesses/muse/settings.json ~/.config/muse/settings.json
ln -s ~/dev/dots/harnesses/muse/muse-opencode-go.service \
  ~/.config/systemd/user/muse-opencode-go.service
systemctl --user daemon-reload
systemctl --user enable --now muse-opencode-go

systemctl --user status muse-opencode-go
systemctl --user restart muse-opencode-go

~/.local/share/muse-opencode-go/venv/bin/python \
  ~/dev/dots/harnesses/muse/test_bridge.py
```

Availability and account restrictions are still determined by OpenCode Go.
