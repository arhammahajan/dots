# Legacy Kimi CLI through model-router

Run `kimi-cli` for the Python CLI; `kimi` continues to run Kimi Code. The legacy
CLI is installed with `uv tool install --python 3.13 'kimi-cli==1.51.0'` using
`UV_TOOL_BIN_DIR=$HOME/.local/share/kimi-cli/bin`. Version 1.52.0 replaces the
CLI entry point with an installer, so retain the 1.51.0 pin.

Copy `config.toml` to `~/.kimi/config.toml` and use `harness kimi-cli` (also
available as the interactive Zsh alias `kimi-cli`). The central launcher sources
the shared router environment and
uses a fresh per-invocation bearer session token for upstream affinity. Resuming
a CLI session starts a new router affinity token. The router must have
`bearer_session_compat` enabled and accept these local client tokens.

The model inventory is a static snapshot of `harnesses/kimi-code/config.toml`;
all models use the router's OpenAI Chat endpoint. The legacy CLI supports only
its own subset of model capability metadata. Refresh the snapshot when the
router inventory changes. Upstream credentials remain in the router.

## T3 Code ACP

Select **Kimi CLI (Model Router)** in T3 Code. The ACP Registry entry for `kimi` uses the central launcher as an
executable override. Set this override to `~/.local/bin/harness` with provider
environment `HARNESS_AGENT=kimi-cli`. The registry supplies the `acp` argument,
invoking the legacy CLI's official ACP entry point through the central launcher. Merge `t3-provider.json` into `providerInstances` in
`~/.t3/userdata/settings.json` on another machine, adjusting the path.

Version 1.51.0's ACP server unconditionally requires Kimi OAuth even for a
configured custom provider. `acp-router-auth.patch` removes those two checks in
the installed ACP server; router credentials are still applied by the launcher.
After reinstalling version 1.51.0, reapply it with:

```sh
patch "$HOME/.local/share/uv/tools/kimi-cli/lib/python3.13/site-packages/kimi_cli/acp/server.py" < harnesses/kimi-cli/acp-router-auth.patch
```

The active config's default model may be changed by the CLI or ACP model picker.
Choose an available router model if the saved default is offline.
