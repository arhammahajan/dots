# Orca terminal agents

Orca uses Terminal as the default launch surface. Start agents from its normal
agent picker; each opens its own terminal interface. There is no ACP frontend
between Orca and these CLIs. The previously installed Toad package and retained
ACP runtime have been removed.

## Standard agent picker

- Claude, Gemini, Muse, Cline, Kilo and Vibe use the existing `harness` launcher
  for router configuration. Cline starts with `--tui`.
- Codex and Grok use their primary connections through their normal executables.
- OpenCode, Pi, Qwen, Copilot and OMP use their normal commands and saved settings.
- Kimi Code uses its installed executable directly.
- Antigravity uses the installed native `agy` terminal CLI. A process-local
  settings file selects native Google authentication instead of the shared
  CLI settings' router provider. Both Google accounts are saved in Orca's
  native Antigravity account selector.
- DeepSeek uses a `dsh-tui` profile installed with its terminal frontend. The
  profile uses the existing router inventory and defaults to
  `model-router/deepseek-v4.1-flash` with low effort.

Command overrides on the picker rows select the executable or shared harness;
Orca still knows which terminal agent it launched and supplies its normal
arguments and hooks. Existing Orca permission arguments are preserved.

## Extra configurations

Quick Commands contains only configurations beyond the standard picker:

- Codex · Model Router
- Grok · Model Router
- Antigravity · mahajanarham0
- Antigravity · mahajanarham
- Antigravity · Model Router
- Kimi CLI · Legacy
- MiniMax Code
- fx

These call `harness profile <instance>`, which executes that agent's terminal CLI.
The Antigravity Google shortcuts select the corresponding saved Orca account
before starting the CLI. Account selection follows Orca's native shared-keyring
behavior; it is not the former ACP profile isolation. The router shortcut runs
`harness antigravity` and retains the standard Antigravity terminal interface.
The legacy Kimi shortcut runs its native UI, not its `term`/ACP subcommands.

The old `T3 ·` Quick Commands have been deleted. Unrelated saved commands are
preserved. `harness profile --list` shows the existing instance IDs for
compatibility, including the `acpRegistry_` names; it never launches ACP.

Use `harness profile --describe <instance>` to inspect a launch without starting
the agent or selecting an account. Ordinary `harness <agent>` commands retain
their existing behavior. `harness profile codex` selects primary Codex, while
`harness codex` and `harness profile codex_router` select the router connection.

## Refresh and verification

With Orca running and the agent CLIs already installed:

```sh
# Install DeepSeek's terminal profile if absent:
~/.local/share/deepseek-harness/bin/dsh plugin --profile dsh-tui add @deepseek-harness-tui/dsh-tui@0.20.1

node harnesses/orca/sync.cjs
# Import existing T3 Google grants once, if the accounts aren't already saved:
python3 harnesses/orca/import-antigravity.py
python3 harnesses/orca/verify.py
```

The import exchanges the existing grants using their OAuth client and stores
native credentials through the Linux keyring and Orca's encrypted account
store. It preserves the previous current native credential during import.
Only the resulting Orca account IDs are saved in
`~/.config/ai/orca/antigravity-native-accounts.json`. No credentials are stored in
this checkout or printed by the scripts. Re-running the import skips accounts
that are already present.

The sync script updates settings through Orca's runtime, not its live SQLite database.
Backups are in `~/.config/ai/orca/orca-settings-before-*.json`. It preserves
unrelated settings and commands. Terminal is already the configured global
surface; any future per-agent Chat override should be switched back to Terminal
in Orca Settings.

Git tracks these integration scripts and documentation. Orca's live settings
database and the installed helper files below `~/.config/ai/orca/` are
machine-local. The removed setup script is no longer needed for this installed
configuration; restoring it on another machine requires those helper files too.

## Verification

All 23 enabled instance mappings resolve to terminal executables without ACP
arguments. Native Antigravity CLI 1.5.9 lists all six account models for both
Google accounts. Antigravity and Cline were opened through Orca's normal
terminal-agent launcher; DeepSeek's terminal frontend was also opened. Test tabs
were closed afterward. Verification does not send inference prompts for every
agent.

The shared router and agent settings remain the source of provider configuration.
T3-specific history, model-menu ordering and orchestration are not imported.
Pre-existing external MCP executable paths and user CLI model preferences are
preserved.
