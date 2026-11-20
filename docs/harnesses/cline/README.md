# Cline CLI

Cline CLI 3.0.74 connects to the local model-router OpenAI-compatible Chat endpoint at `http://127.0.0.1:4000/v1`, with `deepseek-v4.1-flash` as the default model. The router handles client compatibility without a separate proxy and supplies upstream credentials. Enable `[server] bearer_session_compat = true` in the router configuration with client authentication disabled. See the [shared setup](../README.md) for router service details.

## Running and selecting models

Start an interactive session with `cline -i`, or pass a prompt to `cline`. Inside an interactive session, run `/model` to browse and select models served by the router's `/v1/models` endpoint.

Pass any model ID from the router inventory with `-m`:

```sh
cline -i -m local-llama
cline -i -m kimi-k3
curl -fsS http://127.0.0.1:4000/v1/models
```

In interactive Zsh, `.zshrc` unaliases `cline`, so the plain `cline` command resolves the installed CLI from fnm's default Node installation. The `harness cline` launcher is opt-in. When you run `harness cline`, the launcher finds the `cline` binary next to `fnm exec --using=default which node` and passes `--key` with a fresh public affinity token in the `router-session-UUID` format.

Cline saves that temporary public token as its API key. The token is not an upstream credential. It binds routing affinity across all turns in one invocation, not across resumed invocations that start a new process.

## Configuration and symlinks

`harnesses/setup.sh` creates two symlinks for Cline:

| Repository path | Destination |
| --- | --- |
| `harnesses/cline/cline_mcp_settings.json` | `~/.cline/data/settings/cline_mcp_settings.json` |
| `harnesses/shared/skills` | `~/.cline/skills` |

`cline_mcp_settings.json` configures the shared Helium `agent-browser` MCP server. See [shared MCP servers](../browser-mcp.md) for launcher setup.

Cline stores provider settings in `~/.cline/data/settings/providers.json`. Keep this file machine-local because Cline also writes credentials for other providers there. Cline normalizes the auth provider ID `openai` to `openai-compatible` in its saved settings.

## T3 Code ACP provider

`harnesses/cline/t3-provider.json` records the `acpRegistry_cline` entry for the official ACP Registry, with `agentId` set to `"cline"`. It defines no executable override. T3 Code resolves the registry version of Cline, installs its distribution, and passes `--acp` itself.

Provider-level environment variables in `t3-provider.json` set `CLINE_PROVIDER` to `openai-compatible`, `CLINE_MODEL` to `deepseek-v4.1-flash`, `CLINE_API_KEY` to a fixed public `router-session-UUID` value, and `PATH` to include Node and npm for the registry installer. T3 Code does not invoke `harness cline`; the CLI launcher remains available for terminal use. Cline requires a working Node and npm installation.

Because `CLINE_API_KEY` is fixed for this T3 Code provider instance, all of its conversations share upstream routing affinity. Each ACP conversation keeps its own messages and session ID. By contrast, `harness cline` generates a new affinity token for each process invocation.

Cline writes its ACP model catalog to `~/.cline/data/settings/models.json`. T3 Code populates its model picker from the agent's ACP session.
