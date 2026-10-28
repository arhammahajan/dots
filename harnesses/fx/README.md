# fx through model-router

Run `fx` normally. The fx effort fork uses the `model-router` custom
connection and defaults to `local-llama`. All 30 router models are registered
in `/model` and `fx models`; select one with `/model <id>` or `--model <id>`.

`~/.fx/settings.json` is a managed copy of `settings.json`, keeping personal
workspace entries and runtime preferences outside Git. The tracked file contains
the router connection, model metadata, initial model selection, and update policy.
No Vercel login or upstream credentials are needed.

## Compatibility and services

fx's [custom connection](https://fx.sh/docs/configure-fx/custom-model-connections)
supports Chat Completions, bearer environment authentication, and static model
metadata. It cannot set custom session headers, and it sends multiple leading
system messages. Hosted OpenCode routes require session affinity; the currently
loaded llama.cpp template accepts only one leading system message.

fx connects directly to model-router at `127.0.0.1:4000`. Enable
`[server] bearer_session_compat = true` in the router's unauthenticated local
configuration. The router combines consecutive leading plain-text system
messages and maps the launcher's public session token to the upstream header.
Streaming responses and model discovery are served by the main router.

Both `~/.local/bin/fx` and `~/.local/bin/fx-with-router` link to `fx-with-router`.
The launcher sets `FX_MODEL_ROUTER_SESSION` to a fresh `router-session-<UUID>`
for each invocation and executes `~/.local/libexec/fx/fx-router`.
All turns and permission reviews in one invocation share the ID; resuming in
another invocation gets a new ID. This token is public, not an upstream key.
No separate proxy service is needed.

```sh
fx status --json
fx models
fx doctor --json
systemctl --user status model-router
```

## Model metadata

The static inventory was taken from `/v1/models` on 2026-10-28. Context windows,
tool support and image support follow `metadata.router`; fx does not discover
these from the router itself. Refresh `providers.model-router.model_metadata`
in both the tracked template and active copy when the inventory changes,
preserving active preferences and workspace entries.

At setup, the running local model reports a **32,000-token** context window.
Its output limit is unknown and is omitted. Other known output limits are
copied exactly, except `grok-4.7` and `kimi-k2.7-code`: their published output
limit fills the entire context, which fx's custom-provider validator rejects.
Those entries use a 32,768-token response budget, matching fx's own bounded
request policy for full-window output limits.

## Reasoning effort

The effort fork reads `supports_reasoning` and
`reasoning_efforts` from each model's metadata, populates the native model and
effort menus, and sends the selected value as Chat `reasoning_effort`.
The router receives that value and model-router translates it for the upstream
protocol. The default choice omits the field and uses the destination default.

The router publishes exact effort levels for 22 of the 30 models. Only those
models have selectable levels. The other eight keep their default reasoning
behavior because the router has no verified effort enum for them. Local offers
`none`, `minimal`, `low`, `medium`, `high`, and `xhigh`; DeepSeek Flash offers
`low`, `high`, and `max`. Sparse lists remain sparse.

Restart an existing fx process to load a rebuilt executable and metadata.
Use `/effort` or the effort step in `/model`, or set a value for one request:

```sh
FX_MODEL=local-llama fx ask --effort none "Reply with OK"
```

The client implementation lives in [arhammahajan/fx, branch `main`](https://github.com/arhammahajan/fx/tree/main),
checked out at `~/dev/fx`. This folder contains the router integration only.
`build-fx.py` clones that branch if the checkout is missing, then builds and
installs `~/.local/libexec/fx/fx-router`. For an existing checkout, it builds
the current files without pulling or switching branches. Set `FX_SOURCE_DIR`
to use another checkout location.

The helper uses the checksum-verified Zig 0.17.0 Linux x86_64 compiler cached
under `~/.local/share/fx-router/toolchain`, downloading it if necessary.
Build artifacts stay in the fork checkout.

```sh
python3 ~/dev/dots/harnesses/fx/build-fx.py
```

Automatic upgrades are disabled in the fx profile. Update the fork and rebuild
explicitly when upgrading fx.

## Installation elsewhere

Back up any existing `~/.fx/settings.json` before merging this connection into
it. With fx already installed and the shared model-router service configured:

```sh
mkdir -p ~/.fx ~/.local/libexec/fx ~/.config/systemd/user
python3 ~/dev/dots/harnesses/fx/build-fx.py
cp ~/dev/dots/harnesses/fx/settings.json ~/.fx/settings.json
chmod 600 ~/.fx/settings.json
mv ~/.local/bin/fx ~/.local/libexec/fx/fx
ln -s ~/dev/dots/harnesses/fx/fx-with-router ~/.local/bin/fx
ln -s ~/dev/dots/harnesses/fx/fx-with-router ~/.local/bin/fx-with-router
```

## Checks

Router compatibility tests live in `~/dev/model-router/tests/test_client_compat.py`.
Native client tests live in the fx fork.

```sh
fx status --json
fx models
fx doctor --json
```
