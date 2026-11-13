# Shared Helium browser MCP servers

All harness configurations include `agent-browser`, `playwright`, and
`browser-use`. Zed uses these harnesses over ACP, so the browser servers are
configured in the harnesses.
Existing non-browser servers are retained.

These entries reuse this Mac's installed launchers:

| Server | Executable and arguments |
| --- | --- |
| agent-browser | `~/.local/bin/agent-browser mcp --tools core,debug --config ~/.local/share/helium-tools/agent-browser/config.json` |
| Playwright | `/opt/homebrew/bin/node ~/.local/share/playwright-mcp/attach-helium.mjs` |
| Browser Use | `~/.local/bin/browser-use-helium --mcp` |

The JSON and TOML files use absolute paths because MCP clients do not all expand
`~`. On another machine, install these launchers and change the paths before
using the configs. The launchers and their dependencies live outside this repo;
setup details are in `~/.local/share/helium-tools/README.md` on this Mac.

Keep Helium open with remote debugging enabled at
`helium://inspect/#remote-debugging`. The launchers discover its current endpoint
from `~/Library/Application Support/net.imput.helium/DevToolsActivePort`; they
attach to its existing tabs and signed-in sessions. Restart MCP connections
after restarting Helium. Use one browser controller per task.

## Config destinations

Paths in the first column are relative to `harnesses/`. `harnesses/setup.sh`
symlinks the standalone MCP files below. Entries that share a file with harness
settings still need a merge: `claude/mcp.json` into `~/.claude.json`,
`codex/config.toml` into `~/.codex/config.toml`, and the DeepSeek profile
patches. Merge with existing destination files so unrelated servers and
machine-local settings survive. Existing symlinks pick up edits immediately on
disk; managed copies and new files need to be updated separately.

| Repository config | Destination or loading method |
| --- | --- |
| `codex/config.toml` | `~/.codex/config.toml` |
| `claude/mcp.json` | Merge `mcpServers` into `~/.claude.json`, or launch `claude --mcp-config /absolute/path/to/dots/harnesses/claude/mcp.json` |
| `cline/cline_mcp_settings.json` | `~/.cline/data/settings/cline_mcp_settings.json` |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` (`mcp.servers`) |
| `pi/mcp.json` | `~/.pi/agent/mcp.json` |
| `qwen/settings.json` | `~/.qwen/settings.json` |
| `kimi-code/mcp.json` | `~/.kimi-code/mcp.json` |
| `kimi-cli/mcp.json` | `~/.kimi/mcp.json` |
| `mcode/mcp.json` | `~/.minimax/mcp.json` (or the active profile's data directory) |
| `muse/settings.json` | `~/.config/muse/settings.json` |
| `fx/mcp.json` | `~/.fx/mcp.json` |
| `copilot/mcp-config.json` | `~/.copilot/mcp-config.json` |
| `grok/config.toml` | `~/.grok/config.toml` |
| `grok/router-config.toml` | `~/.grok-t3/model-router/config.toml` |
| `kilo/kilo.json` | `~/.config/kilo/kilo.json` (`mcp`) |
| `vibe/config.toml` | `~/.vibe/config.toml` (`[[mcp_servers]]`) |
| `antigravity/mcp_config.json` | `~/.gemini/config/mcp_config.json` (IDE) |
| `antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` |
| `antigravity/acp/settings.json` | `~/.gemini/antigravity-acp/settings.json` for the primary ACP provider |
| `antigravity/acp/router-settings.json` | `~/.config/ai/antigravity-router-acp/antigravity-acp/settings.json` for the router ACP provider |
| `deepseek/cordis.patch.yml` | Merge the three `mcp-*` client entries into each `~/.dsh/profiles/<profile>/cordis.patch.yml`; `setup.sh` installs the complete tracked patches |

Do not replace the entire `~/.claude.json` with the MCP fragment: it also holds
Claude's machine-local state.

Restart harness sessions after updating their active configs. Pi also supports
`/reload`. Its browser tools use deferred exposure. Codex's agent-browser
allowlist includes the native `agent_browser_highlight` tool.

## Interaction visibility

When using agent-browser, take a fresh snapshot and call its native highlight
tool on the target immediately before clicking, typing, filling, checking, or
selecting. For the CLI, use `agent-browser highlight <selector-or-ref>` first.
Keep Browser Use's configured native highlights enabled. Keep Playwright's
page initialization hook and native action decorations enabled, and use its
native interaction tools so those cues remain visible.

## Format references

The formats follow each client's native configuration:
[Grok](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md),
[Pi](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/mcp.md),
[Kimi Code](https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/mcp.md),
[legacy Kimi](https://github.com/MoonshotAI/kimi-cli/blob/main/docs/en/customization/mcp.md),
[MiniMax's MCP config reader](https://github.com/MiniMax-AI/minimax-code/blob/main/packages/local-runtime-v2/src/service/mcp/runtime/config-file.ts),
[fx's MCP config reader](https://github.com/vercel-labs/fx/blob/main/src/core/mcp_host/config_file.zig),
[Muse](https://meta-models.github.io/muse-code-sdk/next/guides/extend/mcp-servers/),
[Kilo](https://kilo.ai/docs/automate/mcp/using-in-cli),
[Vibe](https://docs.mistral.ai/vibe/code/cli/configuration-reference),
[DeepSeek](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/mcp/mcp-client/README.md),
[Copilot](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers), and
[Cline](https://docs.cline.bot/mcp/configuring-mcp-servers).
