# Shared MCP servers

Every agent CLI config in `harnesses/` registers the `agent-browser` MCP server. Zed connects to these agent CLIs over ACP, so the browser server belongs in the CLI configs rather than in Zed. Merging these entries preserves any unrelated MCP servers already present on a machine.

The [Gemini CLI setup](gemini/README.md#shared-settings-skills-and-mcp) and most other tracked configs write `${HOME}` paths for the launcher. OMP has no separate global MCP file because it inherits the Claude config.

## Launcher and Helium remote debugging

The tracked configs run a per-machine `agent-browser` launcher installed under `~/.local`:

| Server | Command and arguments |
| --- | --- |
| `agent-browser` | `~/.local/bin/agent-browser mcp --tools core,debug --config ~/.local/share/helium-tools/agent-browser/config.json` |

Most JSON, TOML, and YAML configs in `harnesses/` write `${HOME}` in both paths so one tracked file works on any machine with the launcher installed. Two files differ:

- `harnesses/codex/linux-config.toml` uses absolute `/home/arhammahajan/.local/...` paths because `harnesses/setup.sh` symlinks it directly on Linux.
- `harnesses/kimi-cli/mcp.json` runs `sh -c` to expand `$HOME` and resolve `agent-browser` from `PATH`.

The launcher script, its config file, and its dependencies live outside this repository. On another Linux machine, install the launcher under `~/.local/bin` or update the paths in machine-specific configs. Local Linux setup steps live in `~/.local/share/helium-tools/README.md`.

Keep Helium open with remote debugging enabled on port 9222. Start it with `helium-cdp` or the `Helium (remote debugging)` application menu entry so it keeps the default profile, open tabs, and signed-in sessions. The launcher attaches to `http://127.0.0.1:9222` by default, and `AGENT_BROWSER_CDP` overrides that target. Confirm the endpoint before starting a session:

```sh
curl -s http://127.0.0.1:9222/json/version
```

Restart active MCP connections after restarting Helium.

## Config destinations

Paths in the first column are relative to `harnesses/`. `harnesses/setup.sh` symlinks the standalone MCP files, shared settings files, and the `deepseek/profiles` directory into `$HOME`. Three configs still require a merge or manual link because they share a file with machine-local state or use a separate router home. Those files are `claude/mcp.json`, `codex/config.toml` on non-Linux hosts, and `grok/router-config.toml`.

Merge into existing destination files rather than replacing them so unrelated servers and machine-local settings stay intact. Existing symlinks pick up edits from this repository on disk. Update copied files and newly added files separately.

| Repository config | Destination | Method |
| --- | --- | --- |
| `codex/linux-config.toml` | `~/.codex/config.toml` | `setup.sh` symlinks on Linux |
| `codex/config.toml` | `~/.codex/config.toml` | Merge `[mcp_servers.agent-browser]` on non-Linux hosts |
| `claude/mcp.json` | `~/.claude.json` | Merge `mcpServers`, or pass `--mcp-config /absolute/path/to/dots/harnesses/claude/mcp.json` |
| `cline/cline_mcp_settings.json` | `~/.cline/data/settings/cline_mcp_settings.json` | `setup.sh` symlinks |
| `opencode/opencode.json` | `~/.config/opencode/opencode.json` | `setup.sh` symlinks under `mcp.servers` |
| `pi/mcp.json` | `~/.pi/agent/mcp.json` | `setup.sh` symlinks |
| `qwen/settings.json` | `~/.qwen/settings.json` | `setup.sh` symlinks under `mcpServers` |
| `gemini/settings.json` | `~/.gemini/settings.json` | `setup.sh` symlinks under `mcpServers` |
| `kimi-code/mcp.json` | `~/.kimi-code/mcp.json` | `setup.sh` symlinks |
| `kimi-cli/mcp.json` | `~/.kimi/mcp.json` | `setup.sh` symlinks |
| `mcode/mcp.json` | `~/.minimax/mcp.json` | `setup.sh` symlinks; copy to the active profile data directory if overridden |
| `muse/settings.json` | `~/.config/muse/settings.json` | `setup.sh` symlinks under `mcpServers` |
| `fx/mcp.json` | `~/.fx/mcp.json` | `setup.sh` symlinks |
| `copilot/mcp-config.json` | `~/.copilot/mcp-config.json` | `setup.sh` symlinks |
| `grok/config.toml` | `~/.grok/config.toml` | `setup.sh` symlinks under `[mcp_servers.agent-browser]` |
| `grok/router-config.toml` | `~/.grok-t3/model-router/config.toml` | Link or merge manually |
| `kilo/kilo.json` | `~/.config/kilo/kilo.json` | `setup.sh` symlinks under `mcp` |
| `vibe/config.toml` | `~/.vibe/config.toml` | `setup.sh` symlinks under `[[mcp_servers]]` |
| `antigravity/mcp_config.json` | `~/.gemini/config/mcp_config.json` | `setup.sh` symlinks for the IDE |
| `antigravity/cli/settings.json` | `~/.gemini/antigravity-cli/settings.json` | `setup.sh` symlinks under `mcpServers` |
| `antigravity/acp/settings.json` | `~/.gemini/antigravity-acp/settings.json` | `setup.sh` symlinks for the ACP provider |
| `deepseek/cordis.patch.yml` | `~/.dsh/profiles/<profile>/cordis.patch.yml` | Merge `mcp-agent-browser` into custom profiles; `setup.sh` symlinks tracked `deepseek/profiles` |

Do not replace `~/.claude.json` with `harnesses/claude/mcp.json` because `~/.claude.json` also stores Claude Code's machine-local state. For DeepSeek, the tracked `acp`, `headless`, `sdk`, `sdk-minimal`, and `web` profiles under `harnesses/deepseek/profiles/` already include `mcp-agent-browser`.

Restart agent CLI sessions after changing their active configs. Pi also accepts `/reload`, and `harnesses/pi/mcp.json` sets `"exposure": "deferred"` with a 120-second timeout. `harnesses/codex/config.toml` restricts Codex to an `enabled_tools` allowlist that includes `agent_browser_highlight`.

## Interaction visibility

Before clicking, typing, filling, checking, or selecting an element through `agent-browser`, take a fresh snapshot and call the native highlight tool on that target. When running the CLI directly, run `agent-browser highlight <selector-or-ref>` before the interaction command.

## Format references

Each config file follows its client's native MCP schema:

- [Grok](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md)
- [Pi](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/mcp.md)
- [Kimi Code](https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/mcp.md)
- [legacy Kimi](https://github.com/MoonshotAI/kimi-cli/blob/main/docs/en/customization/mcp.md)
- [MiniMax MCP config reader](https://github.com/MiniMax-AI/minimax-code/blob/main/packages/local-runtime-v2/src/service/mcp/runtime/config-file.ts)
- [fx MCP config reader](https://github.com/vercel-labs/fx/blob/main/src/core/mcp_host/config_file.zig)
- [Muse](https://meta-models.github.io/muse-code-sdk/next/guides/extend/mcp-servers/)
- [Kilo](https://kilo.ai/docs/automate/mcp/using-in-cli)
- [Vibe](https://docs.mistral.ai/vibe/code/cli/configuration-reference)
- [Gemini CLI](https://geminicli.com/docs/reference/configuration/#mcpservers)
- [DeepSeek](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/mcp/mcp-client/README.md)
- [Copilot](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers)
- [Cline](https://docs.cline.bot/mcp/configuring-mcp-servers)
