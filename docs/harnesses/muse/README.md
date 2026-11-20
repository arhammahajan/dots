# Muse through model-router

Muse connects to `http://127.0.0.1:4000/v1` without a separate adapter process or service. `harnesses/muse/settings.json` sets `provider` to `meta`, `endpoint_transport.base_url` to `http://127.0.0.1:4000/v1`, and `endpoint_transport.auth` to `bearer`.

## Router endpoints and translation

Model-router serves the Muse catalog at `/muse-code/models` and `/v1/muse-code/models`. Those endpoints return display labels, context limits, and output limits from the router's central metadata. Generation requests use `/v1/responses` with standard streaming and session handling.

The `/models` endpoint lists every router model. Model-router translates Responses requests for other upstream protocols and reports nonportable controls through the `x-model-router-loss` header. Translation preserves Muse's namespaced function tools across upstream protocols and streamed tool calls.

## Configuration and symlinks

`harnesses/setup.sh` links Muse configuration and skills under `XDG_CONFIG_HOME`, which defaults to `~/.config`.

| Source in repo | Target path | Behavior |
| --- | --- | --- |
| `harnesses/muse/settings.json` | `~/.config/muse/settings.json` | Direct file symlink |
| `harnesses/shared/skills/<skill>` | `~/.config/muse/skills/<skill>` | Per-skill symlinks inside a real directory |

Muse manages `~/.config/muse/skills` as its own store directory, so `harnesses/setup.sh` creates that directory and symlinks each skill subdirectory instead of linking the directory itself. The script also removes broken symlinks in `~/.config/muse/skills` that point back to `harnesses/shared/skills/`.

`harnesses/muse/settings.json` sets the default model to `deepseek-v4.1-flash`, maps per-model reasoning efforts in `model_reasoning_efforts`, and registers the `agent-browser` MCP server.

## Launching Muse and T3 Code

`harnesses/muse/t3-provider.json` records the T3 Code provider entry for Muse Code. Set T3's Muse binary path to the expanded path of `~/.local/bin/harness` and set the provider environment variable `HARNESS_AGENT=muse`.

When `~/.local/bin/harness` starts Muse, the launcher sets `META_API_KEY` from the router client environment for the Muse process only. Muse does not need a Meta login or an upstream API key. `.zshenv` loads only the shared router settings from `~/.config/ai/model-router.env`.

Interactive Zsh removes any alias on `muse` in `.zshrc` and runs the `muse` binary without the launcher. `harness muse` is opt-in, and scripts should call `harness muse`.

## Verification

Check that `model-router` is running and serving the Muse catalog:

```sh
systemctl --user status model-router
curl http://127.0.0.1:4000/v1/muse-code/models
```

See the [shared setup](../README.md) for installation and model refresh.
