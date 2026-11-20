# Codex

This repository tracks Codex CLI configuration, Spaces hook definitions, shared skills wiring, and a second T3 Code provider that routes Codex requests through `model-router`.

## Config files and symlinks

Run `./harnesses/setup.sh` from the repository root to link the Linux Codex configuration and shared skills directory. The script reads `CODEX_HOME` and defaults to `~/.codex`.

| Source in `harnesses/` | Destination | Scope |
| --- | --- | --- |
| `codex/linux-config.toml` | `$CODEX_HOME/config.toml` | Linux only |
| `codex/linux-hooks.json` | `$CODEX_HOME/hooks.json` | Linux only |
| `shared/skills` | `$CODEX_HOME/skills/shared` | All platforms |

Linking `shared/skills` at `$CODEX_HOME/skills/shared` exposes every shared skill through nested discovery. Codex's vendor-managed `.system` directory stays separate.

`harnesses/codex/linux-config.toml` sets `model = "gpt-6-luna"` with `model_reasoning_effort = "low"`, configures the Helium `agent-browser` MCP server, records trusted project directories, enables `[features] hooks = true`, and stores trusted SHA-256 hook hashes for both `~/.codex/hooks.json` and `~/.codex-t3/model-router/hooks.json`.

`harnesses/codex/linux-hooks.json` registers nine Codex hook events for Spaces status tracking. `SessionStart`, `Stop`, and `UserPromptSubmit` run `spaces session-hook codex <start|stop|prompt>`. `PreToolUse`, `PostToolUse`, `PermissionRequest`, `SubagentStart`, `SubagentStop`, and `Interrupt` run `spaces hook codex <event>`. See [the Zellij guide](../../zellij.md) for the hook listener setup.

`harnesses/setup.sh` does not link two portable reference files:

- `harnesses/codex/config.toml` uses `${HOME}` paths for non-Linux or custom setups, enables desktop preferences and bundled plugins, and includes the `agent-browser` MCP entry with an `enabled_tools` allowlist. See [browser MCP setup](../browser-mcp.md) for merging that MCP block into an existing `~/.codex/config.toml`.
- `harnesses/codex/hooks.json` holds an empty `{"hooks": {}}` template.

## Codex through model-router in T3

In T3 Code, select `Codex (Model Router)` in the provider picker to send Codex traffic through the local `model-router` service. The primary `codex` provider and `defaultModelSelection` in `harnesses/t3code/settings.json` stay unchanged. From the shell, plain `codex` keeps its primary connection, and `harness codex` runs Codex through `model-router`.

The `codex_router` provider invokes the Go `harness` CLI at `~/.local/bin/harness` with environment variable `HARNESS_AGENT=codex`, shared home `~/.codex`, and shadow home `~/.codex-t3/model-router`. Build `~/.local/bin/harness` from `~/dev/model-router` with `go build ./cmd/harness`. T3 creates shared state symlinks inside the shadow home, and the `harness` CLI applies provider overrides on every call.

Only the placeholder API credential lives in the shadow home. Upstream provider credentials remain in `model-router`.

## Recreating the T3 router provider

`harnesses/codex/t3-router-provider.json` stores a snapshot of the `codex_router` provider entry for this machine. `harnesses/setup.sh` links `harnesses/t3code/settings.json` to `~/.t3/userdata/settings.json`, and that tracked settings file does not include `codex_router` by default.

To install the router provider on a machine:

1. Merge the `codex_router` object from `harnesses/codex/t3-router-provider.json` into `providerInstances` in `~/.t3/userdata/settings.json`.
2. Update the absolute paths in `binaryPath`, `homePath`, and `shadowHomePath` for the local user.
3. Create the shadow home and write the placeholder credential file with mode `0600`:

```sh
mkdir -p "$HOME/.codex-t3/model-router"
printf '{"OPENAI_API_KEY":"unused"}\n' > "$HOME/.codex-t3/model-router/auth.json"
chmod 600 "$HOME/.codex-t3/model-router/auth.json"
```

The snapshot lists 28 `customModels` entries from the router's `/v1/models` endpoint. Hide built-in Codex models outside that list in T3 model preferences.

## Router compatibility and defaults

Codex 0.165 requires the local router's `client_metadata` compatibility change in `~/dev/model-router/model_router/constants.py`. The router discards `client_metadata` during lossy translation, emits a loss diagnostic, and never converts it into adapter configuration.

The Go router profile for Codex defaults to `gpt-6-luna`, which passed a live Codex request. Because that route uses the native Responses protocol, it accepts Codex reasoning controls. See [the shared router setup](../README.md) for service management, client environment variables, and the `harness` CLI.
