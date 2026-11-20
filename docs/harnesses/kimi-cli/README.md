# Legacy Kimi CLI through model-router

Run `kimi-cli` to start the legacy Python CLI. The `kimi` command runs [Kimi Code](../kimi-code/README.md).

## Installation

Install version `1.51.0` with `uv` into its own tool binary directory:

```sh
UV_TOOL_BIN_DIR="$HOME/.local/share/kimi-cli/bin" uv tool install --python 3.13 'kimi-cli==1.51.0'
```

Keep the `1.51.0` pin. Version `1.52.0` replaces the CLI entry point with an installer.

## Configuration and symlinks

Run `harnesses/setup.sh` to symlink the tracked files into `~/.kimi/`:

| Repository source | Target path |
| --- | --- |
| `harnesses/kimi-cli/config.toml` | `~/.kimi/config.toml` |
| `harnesses/kimi-cli/mcp.json` | `~/.kimi/mcp.json` |
| `harnesses/shared/skills` | `~/.kimi/skills` |

`harnesses/kimi-cli/config.toml` also sets `extra_skill_dirs` to `["~/dev/dots/harnesses/shared/skills"]`. `harnesses/kimi-cli/mcp.json` registers the `agent-browser` MCP server. See [browser MCP setup](../browser-mcp.md) for browser MCP details and [shared setup](../README.md) for service units and general launcher behavior.

## Running with model-router

Run `kimi-cli` directly in interactive Zsh, or run `harness kimi-cli` to launch it through the `harness` CLI built from `~/dev/model-router`. The `harness` launcher loads `~/.config/ai/model-router.env` and assigns a fresh per-invocation bearer session token for upstream affinity. Resuming a CLI session starts a new router affinity token. `model-router` must enable `bearer_session_compat` and accept these local client tokens.

`harnesses/kimi-cli/config.toml` defines one provider, `model-router`, with `type = "openai_legacy"`, `base_url = "http://127.0.0.1:4000/v1"`, and `api_key = "unused"`. Upstream credentials stay in `model-router`.

The model list in `harnesses/kimi-cli/config.toml` is a static snapshot of `harnesses/kimi-code/config.toml`. Every model routes through the router's OpenAI Chat endpoint. The legacy CLI accepts only the `thinking`, `image_in`, and `video_in` capability flags. Refresh `harnesses/kimi-cli/config.toml` when the router inventory changes.

The CLI or ACP model picker can update `default_model` in `~/.kimi/config.toml`. Pick an available router model if the saved default is offline.

## T3 Code ACP

`harnesses/kimi-cli/t3-provider.json` defines the `acpRegistry_kimi_legacy` provider entry for T3 Code under the display name "Kimi CLI (Model Router)". Merge that entry into `providerInstances` in `~/.t3/userdata/settings.json` on another machine, and update `commandPath` if your home directory path differs.

The entry uses the ACP Registry `kimi` agent with `commandPath` set to `~/.local/bin/harness` and `HARNESS_AGENT=kimi-cli` in `environment`. The ACP Registry supplies the `acp` argument, so T3 Code starts the legacy CLI's ACP server through the `harness` launcher.

Version `1.51.0` calls `self._check_auth()` before creating or loading an ACP session. That check requires Kimi OAuth even when you configure a custom provider. `harnesses/kimi-cli/acp-router-auth.patch` removes both `self._check_auth()` calls in `kimi_cli/acp/server.py`. The `harness` launcher still applies router credentials. Reapply the patch after installing or reinstalling version `1.51.0`:

```sh
patch "$HOME/.local/share/uv/tools/kimi-cli/lib/python3.13/site-packages/kimi_cli/acp/server.py" < harnesses/kimi-cli/acp-router-auth.patch
```
