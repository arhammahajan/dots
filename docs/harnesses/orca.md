# Orca launches from T3 configuration

Open Orca's **Quick Commands** menu and select an entry beginning with **T3 ·**.
There is one named launch for every enabled T3 provider instance, plus primary
Codex. These entries retain separate Codex/Grok direct and router connections,
both Antigravity accounts, Antigravity router, and legacy Kimi.

The standard agent picker is also configured for Claude, Codex, Grok, Gemini,
Muse, Cline, Kilo, Kimi Code, Vibe, Qwen, Copilot, Pi, OpenCode, and OMP. Use the
named Quick Commands for the full T3 instance selection. Orca has a fixed agent
catalog; its settings API cannot add duplicate provider rows or arbitrary ACP
presets. Quick Commands open terminal interfaces rather than native chat tabs.

## Installation and refresh

With Orca running:

```sh
uv tool install batrachian-toad
python3 harnesses/orca/setup.py
orca-agent --list
```

The setup uses Orca's running settings RPC service, including per-entry Quick
Command mutations. It never writes to Orca's live SQLite database. Refreshing
preserves unrelated Quick Commands, replaces this integration's entries by stable
ID, and removes its entries for providers that have since been disabled/deleted.
Previous settings are saved under `~/.config/ai/orca/orca-settings-before-*.json`.

`orca-agent` reads the active `~/.t3/userdata/settings.json` on each launch.
It applies that instance's environment and configured executable. Registry
agents retain their ACP entry points; Toad provides the interactive terminal
client. Native agents run directly. The existing shared `harness` continues to
configure router connections. Toad is an additional installed tool, currently
version 0.6.20.

Examples:

```sh
orca-agent codex
orca-agent codex_router --model grok-4.7
orca-agent grok_router --model deepseek-v4.1-flash
orca-agent antigravity
orca-agent antigravity_mahajanarham
orca-agent acpRegistry_antigravity_router
```

Model IDs and labels from T3's custom model settings are retained in the private
`~/.config/ai/orca/inventory.json`. Native CLI model menus and ACP-discovered model
menus belong to those agents; T3's hidden models, ordering, favorites, and custom
option descriptors cannot be imported into Orca's UI through its installed API.
Pass native model flags explicitly when a custom model is absent from a CLI menu.
ACP model/mode controls are exposed through Toad, subject to Toad's support.

## Antigravity account isolation and runtime limitation

T3 stores account profiles below
`~/.t3/userdata/providers/antigravity/<SHA-256(instance ID)>`. The first setup copies
each account's settings and OAuth grant into separate private folders below
`~/.config/ai/orca/accounts/`. Later refreshes preserve these copies, including any
refreshed credentials. Shared skills are linked into each profile. Credentials
are never written into this repository or printed by the launcher.

A retained ACP runtime and its adjacent harness are copied into
`~/.config/ai/orca/antigravity-runtime/`. Both named accounts use their own
`GEMINI_HOME` and file credential storage, so concurrent sessions don't switch a
shared login. The router instance uses the existing router profile and fetches
its inventory from the model router.

**This is not complete Antigravity parity.** The retained working ACP server
reports version 1.3.0 and lists three account models. T3's cache reports version
1.5.8 and six models. Its active runtime path contains a different CLI executable
which ignores `GEMINI_HOME` for its native sign-in; it does not recognize the ACP
grants copied from T3. The original shared `harness antigravity-acp` also expects
an `agy_acp_server.par` beneath that active release, where none exists. This
integration uses the working retained executable rather than modifying T3's
runtime or converting grants into an incompatible credential format.

Orca's native Antigravity chat owns its runtime/account preparation and scrubs
gateway environment overrides. Its native account selector does not import T3's
ACP OAuth grant files. The three named Antigravity Quick Commands therefore run
through Toad. Orca's ordinary Antigravity picker row remains its own integration.

## Verification and limits

```sh
python3 harnesses/orca/verify.py
python3 -m unittest discover -s harnesses/tests -q
```

The verification checks ACP initialization for all ACP instances, and opens
prompt-free sessions for both Antigravity accounts and router. Results are saved
to `~/.config/ai/orca/verification.json`. On 2026-11-14 all 14 initializations
passed; both account sessions opened with three models, and router opened with
27 models. The Antigravity Toad interface was also opened and read in an actual
Orca terminal, then closed. These checks do not send model prompts or establish
full inference/tool-use parity for every agent.

T3 orchestration tools and session history are not migrated. Orca's own terminal
status, native-chat support, permissions and orchestration differ. Existing
agent-specific MCP settings and shared skills continue to be used; pre-existing
macOS-only MCP executable paths in those settings are preserved.
