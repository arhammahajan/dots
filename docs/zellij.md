# Zellij as an agent terminal multiplexer

This setup turns Zellij 0.46 into a Herdr-style workspace manager for coding
agents. Agent state travels over **OSC 7501** (the Program Status Protocol), so
Zellij shows which agent is working, blocked, done or failed without scraping
pane output.

```text
Herdr                           Zellij
────────────────────────────    ──────────────────────────────────────────────
space (project)              →  session (one per project, resurrectable)
tab                          →  tab (one per agent or task)
pane                         →  pane
spaces popup                 →  Spaces navigator (Ctrl-b w)
agent inbox                  →  Inbox (Ctrl-b i / b)
agent status (OSC 7501)      →  attention: frame markers, tab markers,
                                 session indicator, cross-session inbox
prefix key (Ctrl-b)          →  Ctrl-b leader, locked by default
```

## Native OSC 7501, plus a real Spaces navigator

Zellij 0.46 parses OSC 7501 in its terminal server, before output reaches a
plugin. It already maintains per-pane program-status records, resolves them into
an attention state, and renders that state in pane frames, the tab bar, the
session indicator and the Inbox. The OSC 7501/Inbox side is therefore native.

The **Spaces navigator** is a standalone plugin built from the source in
`~/dev/model-router/plugins/zellij-spaces` (override its location
with `ZJ_SPACES_REPO`). It is not the session manager and it is not a
sidebar. It follows Herdr's workspace navigator shape:

- spaces are grouped rows, with their agents and terminals underneath;
- each row carries live attention state, the agent/app name, tab, attention
  text and age;
- spaces roll up their agents' state, so a blocked agent is visible without
  opening the space;
- `Enter` jumps directly to the selected space or agent pane;
- `/` searches names, apps, tabs, paths and attention text;
- `b` / `w` / `d` / `i` filter by blocked, working, done or idle;
- new, rename, close and collapse are inline actions, not separate screens.

The look is deliberately Herdr-inspired and uses the Catppuccin Mocha palette:
a rounded lavender-accented frame, a `/` search bar with attention-filter chips,
grouped space rows with status counts and paths, tree-style agent/terminal rows
with attention messages and ages, a detail line for the selected destination,
and compact key help at the bottom. There is no persistent sidebar and no
session-manager form UI.

The implementation is split into a small model (`model.rs`), a pure
event-to-effect update layer (`app.rs`), a cell renderer (`canvas.rs`) and the
Zellij host boundary (`main.rs`). It subscribes to both `SessionUpdate` and
`GlobalAttentionUpdate`: the former provides tabs, panes and viewport data, the
latter is the cross-session source of truth for agent state, including detached
spaces. It requests `ReadApplicationState`, `ChangeApplicationState` and
`RunCommands`; the grant is cached by absolute path in
`~/.cache/zellij/permissions.kdl`.

A tiny opener instance opens the navigator through Zellij's real popup API with
`capture_all_keys` and outside-click close, then exits. Because the navigator is
a true popup, keystrokes cannot leak to the terminal underneath and focus cannot
drift behind it, including in split-pane tabs.

`Ctrl-b w` launches it as a bordered, centered popup sized to about 86% x 82% of
the terminal. `Ctrl-b i` / `b` continue to open the stock session manager
exactly as before.

## Install

[`scripts/setup-zellij.sh`](../scripts/setup-zellij.sh) downloads the official
v0.46.0 static musl build for the current platform, verifies the release
SHA-256, installs `~/.local/bin/zellij`, the navigator WASM from
`~/dev/model-router/plugins/zellij-spaces`, seeds its permission cache, and
symlinks the config plus the harness CLI as the `zj` helper:

```sh
bash scripts/setup-zellij.sh
zellij --version          # zellij 0.46.0
zellij setup --check      # CONFIG FILE: Well defined.
```

For this machine the extracted binary's SHA-256 is
`afb6bfb5f9fdc3d9e9d094ce21866bc99d37073a5fbe75bb612e21fc9b96033d`. The manual
equivalent (x86_64 Linux musl) is:

```sh
mkdir -p ~/.local/bin ~/.config/zellij
curl -fL -o /tmp/zellij.tar.gz \
  https://github.com/zellij-org/zellij/releases/download/v0.46.0/zellij-x86_64-unknown-linux-musl.tar.gz
tar xzf /tmp/zellij.tar.gz -C /tmp
install -m 755 /tmp/zellij ~/.local/bin/zellij
install -m 644 ~/dev/model-router/plugins/zellij-spaces/zellij-spaces.wasm \
  ~/.config/zellij/plugins/zellij-spaces.wasm
ln -sfn "$PWD/zellij/config.kdl" ~/.config/zellij/config.kdl
ln -sfn ~/.local/bin/harness ~/.local/bin/zj
```

The plugin is developed and rebuilt in the model-router monorepo:

```sh
bash ~/dev/model-router/plugins/zellij-spaces/build.sh
```

That installs the `wasm32-wasip1` Rust target if needed, runs the plugin's unit
tests, builds the WASM and refreshes that repo's tracked artifact. Rerun
`scripts/setup-zellij.sh` afterwards to install the fresh WASM.

## OSC 7501 wiring

Zellij answers the protocol probe `ESC ] 7501 ; ? ST` and keeps a record per
pane (per `id` inside that pane). The config enables the whole attention
pipeline:

```kdl
attention {
    enabled true
    frame_markers true
    tab_markers true
    host_status true
    idle_rule "all"
    track_commands true
    done_threshold_secs 10
    startup_card "attention"
}
```

`host_status` re-emits the most urgent record to the outer terminal as OSC 7501,
so a supporting terminal or a nested Zellij session still sees agent state.
Terminals that do not understand it ignore the sequence.

### Agents that report natively

Installed agents that emit OSC 7501 on their own (verified against the upstream
integration table and, where possible, on this machine):

| Agent        | Version here | Notes |
| ------------ | ------------ | ----- |
| Claude Code  | 2.1.325      | reports while the TUI runs |
| Codex CLI    | 0.169.0      | reports done/failed since 0.164 |
| OpenCode     | 2.3.0        | working/waiting/idle; the local `zellij-status` plugin adds done |
| Pi           | 1.5.2        | live-verified: `state=idle:app=pi`, then `working`, then `done` |
| Crush        | 0.101.0      | reports natively |
| Grok Build   | 1.0.77       | reports natively |
| omp          | 18.12.6      | reports natively |
| GitHub Copilot CLI | 1.0.112 | reports natively |
| Kimi Code    | 2.5.3        | needs `PI_PROGRAM_STATUS=1` (set in `.zshrc`) |
| Muse         | installed    | reports `idle`/`working`/`blocked`/`done`/`error` with `app=muse` |
| MiniMax Code | 0.7.7        | reports with `:app=mcode` |
| fx           | installed    | reports from `src/ui/terminal/program_status.zig` |
| Mistral Vibe | 3.1.2        | bundled `vibe-rs` reports with `:app=vibe` |
| DeepSeek/dsh-tui | 0.20.1   | reports with `:app=dsh-tui`; `DSH_TUI_PROGRAM_STATUS=0` disables it |
| Qwen Code    | 0.26.2       | below 0.26.9; falls back to notifications/screen |
| Gemini CLI   | 0.68.0       | no OSC 7501 yet; notifications enabled |
| Cursor CLI   | current      | no OSC 7501; notifications already enabled |

Nothing needs to be installed per agent for the native reporters: Zellij is the
terminal, so their OSC 7501 output is parsed directly. Prompt markers for plain
shell commands come from the `.zshrc` precmd/preexec hooks (OSC 133 plus OSC 7).

### Screen-reader fallback in Spaces

The navigator keeps OSC 7501 primary, but fills the gap for agents that do not
report. While it is open it calls `zj __space-snapshot` about every five
seconds. That helper uses `zellij --session NAME action dump-screen` to read
each agent pane's viewport across every running space, so peer and detached
sessions work without a new plugin permission or a blocking scrollback call.
The plugin evaluates the text with a trimmed port of
[Herdr](https://github.com/herdrdev/herdr)'s manifest engine and bundled agent
manifests (Apache-2.0; provenance in
`~/dev/model-router/plugins/zellij-spaces/src/screen/NOTICE`).

Precedence:

- a strong visible blocker (permission/question prompt) overrides OSC;
- otherwise a live non-idle OSC state wins for the authority agents listed above
  (Claude, Codex, OpenCode, Pi, Crush, Grok, omp, Copilot, Kimi, Muse, MCode, fx,
  Vibe, DeepSeek/dsh-tui);
- for the remaining agents (Gemini, Cursor, Qwen, Cline, Antigravity, Kilo),
  the viewport supplies `working` / `needs you` / `idle`;
- a screen-detected working→idle transition shows `Done` in the navigator until
  the pane is focused or ten minutes pass (while that navigator instance stays
  open).

This fallback is navigator-only: Zellij's tab bar and stock Inbox still show
native attention. `screen_fallback false` in the plugin configuration restores
the previous app-label-only path.

Regenerate the embedded manifest bundle after updating the vendored TOMLs:

```sh
python3 ~/dev/zellij-spaces/scripts/sync-screen-manifests.py
```

The screen detector adds `regex` and the manifests to the WASM (about 2.5 MB
total). `dump-screen` returns the current viewport, so a pane that a client has
scrolled back can misread until it is scrolled to the bottom; plain-idle
evidence requires two consecutive observations to limit false transitions.


### Per-agent hooks

OSC 7501 is the shared transport, but it carries only state and a one-line
message. The Spaces navigator also accepts structured per-agent hook events, so
a pane can show the active tool, the prompt, the assistant reply, the model, the
provider session id, blocked kinds and subagents. The design follows Orca's
per-agent hook system without its Electron runtime.

```text
harness hook ──stdin JSON──> zj hook <agent> [Event]
                              ├─ normalize the provider payload
                              ├─ emit OSC 7501 (main events only)
                              ├─ record the resume session id
                              ├─ POST HTTP over a Unix socket
                              └─ on failure: append the event to a JSONL spool
                                             and lazy-start the listener

Spaces navigator ──1.5 s──> zj __space-hooks          (small summaries)
                 ──H──────> zj __space-hooks --detail SESSION PANE
```

Runtime files live in `${XDG_RUNTIME_DIR:-/tmp}/zj` (0700):

| Path | Purpose |
| --- | --- |
| `h.sock` | HTTP-over-Unix-socket listener (0600) |
| `hooks.endpoint.json` | socket, optional TCP port, token, pid (0600) |
| `hooks.store.json` | merged per-pane records (0600, ephemeral) |
| `hooks-spool.jsonl` | events written while the listener is down |
| `hooks.server.lock`, `hooks.store.lock` | single-instance and write locks |
| `hooks.log` | listener log (1 MiB rotation) |

The listener speaks HTTP/1.1 over the socket: `POST /v1/hook` with an
`X-ZJ-Token` header (the agent is in the body, or in `/v1/hook/<agent>`), plus
an optional loopback TCP port. Requests are capped at 256 KiB, the socket is
mode 0600, and on Linux the peer uid must match the listener's uid. The helper
client is the normal path; direct HTTP clients must include `session` and
`pane` in the body, query string or `X-ZJ-Session`/`X-ZJ-Pane` headers.

The listener is optional. `zj hook` posts with a short timeout; when the socket
is missing it spools the event and starts `zj hook-server --daemon` in the
background, throttled to once every ten seconds. The daemon drains the spool on
start and every second. `zj hooks start|stop|status` manage it manually.
`harnesses/shared/zj-hooks.service` is an optional systemd user unit:

```sh
systemctl --user enable --now zj-hooks.service
```

`ZJ_HOOKS_RUNTIME`, `ZJ_HOOKS_STORE` and `ZJ_HOOKS_DISABLE=1` override the
defaults. The store is in the user runtime dir, so prompts, tool input and
replies are not written to persistent storage unless `ZJ_HOOKS_STORE` points at
one.

#### Installed harness hooks

`harnesses/setup.sh` links the hook configs; every command is observational
(`zj hook … >/dev/null 2>&1 || true`) and always exits 0. No hook approves or
denies a permission.

| Harness | Config | Events |
| --- | --- | --- |
| Claude Code | `harnesses/claude/settings.json` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Stop, StopFailure, SubagentStart, SubagentStop, PostCompact, TeammateIdle, Elicitation, ElicitationResult, SessionEnd |
| Qwen Code | `harnesses/qwen/settings.json` | SessionStart, SessionEnd, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification, SubagentStart, SubagentStop, Stop, StopFailure |
| Codex | `harnesses/codex/linux-hooks.json` | SessionStart, Stop, UserPromptSubmit (legacy `session-hook`), PreToolUse, PostToolUse, PermissionRequest, SubagentStart, SubagentStop, Interrupt |
| Gemini | `harnesses/gemini/settings.json` | SessionStart, SessionEnd, BeforeAgent, BeforeTool, AfterTool, AfterAgent, Notification (timeout in milliseconds) |
| Copilot | `harnesses/copilot/hooks/zellij.json` | sessionStart, userPromptSubmitted, sessionEnd, preToolUse, postToolUse, postToolUseFailure, permissionRequest, notification, subagentStart, subagentStop, preCompact, stop, errorOccurred |
| Cursor | `harnesses/cursor/hooks.json` | sessionStart, sessionEnd, beforeSubmitPrompt, postToolUse, postToolUseFailure, afterAgentResponse, stop, subagentStart, subagentStop (no permission-bearing pre-tool events) |
| Grok | `harnesses/grok/hooks/zellij.json` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, Notification, Stop, StopCancelled, StopFailure, SessionEnd |
| Kimi Code | `harnesses/kimi-code/config.toml` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Stop, StopFailure |
| OpenCode | `harnesses/opencode/plugins/zellij-status.ts` | execution started/succeeded/interrupted/failed, tool execute before/after |

Grok also reads `~/.claude/settings.json` and `~/.cursor/hooks.json` by default.
`zj hook` detects `GROK_SESSION_ID`/`GROK_HOOK_EVENT` in the environment and
attributes those events to Grok, so a shared Claude or Cursor config does not
record a Grok session as Claude. A 500 ms dedupe absorbs double-fired events.

Codex keeps the original three `session-hook` command strings, so their trust
hashes stay valid; the added events are new hook entries and Codex asks to trust
them once. Approve them in Codex's hooks UI; do not hand-edit the hashes in
`harnesses/codex/linux-config.toml`.

#### Precedence

Hooks win while fresh. OSC 7501 fills in when hooks are missing, off, or silent
on a state the program has since left.

- `working`/`blocked`/`waiting` are fresh for 2 minutes since the last event.
- `done`/`failed` persist for 30 minutes; `blocked` with kind `permission` or
  `question` for 30 minutes; `blocked` with kind `auth` for 60 minutes.
- A new `working` event clears a turn-scoped block. `auth` survives a turn end
  and is cleared by the next prompt or session boundary.
- A newer OSC state beats an older hook state; hook `idle` never overrides a
  live OSC/screen state.
- A permission/question prompt that is visibly on screen still overrides any
  hook state.
- Hook `done` is not cleared by focusing the pane; native remembered Done
  behaves the same way.

#### Navigating hook state

Pane rows show the hook preview (`Bash · cargo test`, `approve Write …`, the
reply). The bottom detail line adds blocked kind, event, model, short session id
and subagent count when there is room. `H` opens the hook overlay for the
selected agent pane:

```text
┌ Spaces · agent hooks ─────────────────────────────┐
│ claude-code · working · pre_tool · 12s            │
│ model    deepseek-v4.1-flash                      │
│ session  9f2c4a10                                 │
│                                                   │
│ prompt                                            │
│ fix the failing test                              │
│                                                   │
│ tool · Bash                                       │
│ cargo test --all                                  │
│                                                   │
│ subagents (1)                                     │
│ · Explore  working  a1  8s  find the fixture      │
└───────────────────────────────────────────────────┘
```

`↑`/`↓` and `PgUp`/`PgDn` scroll the overlay; `H`/`Esc` close it. Detail is
fetched only when the overlay opens, so the periodic summary poll stays small.

#### Diagnostics

```sh
zj hooks status                 # listener, store and spool state
zj __space-hooks | jq           # summaries for the navigator
zj __space-hooks --detail S P   # one full record
printf '%s' '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}' \
  | ZELLIJ_SESSION_NAME=probe ZELLIJ_PANE_ID=1 zj hook claude PreToolUse
```

If the listener never starts, `zj hooks status` shows the spool count; starting
the server (`zj hooks start` or the systemd unit) drains it. A stale socket or
endpoint is replaced automatically when the recorded pid is gone.

### Agents or scripts without native support

`zj status` writes the protocol to the pane's controlling terminal:

```sh
zj status working "indexing repository"
zj status blocked "approve the migration?" permission
zj status done "12 files updated"
zj status error "build failed"
zj status clear
```

Agents launched through `zj agent`, restored panes (`zj __resume-wrap`), and
directly from an interactive Zsh all get the same initial state: **idle**, not
working. The launcher writes a short working record followed by an idle record,
because Zellij only remembers panes that reported a non-idle state once. That is
what keeps a freshly opened or resumed conversation visible in the stock Inbox
under `Idle` instead of dropping out of it entirely.

During a turn, the agent's own OSC 7501 records report `working` and `done`
(OpenCode reports only `working`/`idle`, so its plugin adds `done`). Where the
harness exposes lifecycle hooks, `zj hook <agent> [Event]` normalizes the
provider payload, emits the matching OSC 7501 state and also writes the richer
per-pane record described under [Per-agent hooks](#per-agent-hooks). `prompt`
sets working, `stop` sets done, `start` primes idle, and `permission` blocks.
The legacy `zj session-hook <agent> start|prompt|stop` remains an alias.
Claude, Codex and Qwen have Stop hooks; Gemini uses AfterAgent; Copilot,
Cursor, Grok, Kimi and the natively reporting agents use their own records.

OpenCode needs two extra steps. First, its background service broadcasts OSC
7501 records to every TUI connected to it, so three OpenCode panes would
otherwise all turn working when one does. The shell launcher, `harness
opencode`, `zj agent opencode` and restored panes run it with `--standalone`,
giving each pane a private server. Existing OpenCode panes started before this
change keep the shared service until they are restarted, and while they are
shared the navigator cannot tell which pane owns a broadcast record. Run
`opencode --server <url>` (or `command opencode`) if you deliberately want the
shared background service for one pane.

Second, OpenCode 2.3 reports working and idle but never done. The local plugin
in `harnesses/opencode/plugins/zellij-status.ts` subscribes to the server's
execution events and writes a done record after an execution succeeds. It waits
for the TUI's own idle record first and repeats the done record once, so done is
the pane's last state. The plugin is linked into `~/.config/opencode/plugins/`
by `harnesses/setup.sh`; restart OpenCode panes to load it. Because each pane
has a private server, one pane finishing does not mark its siblings done.

`zj status` remains the manual path for scripts and for agents without any
lifecycle support.

## Keys

The session starts in **Locked** mode so agent TUIs get every keystroke. `Ctrl-b`
is the leader (Herdr's default prefix), and every action returns to Locked. Alt
chords are explicitly unbound while locked. If an outer multiplexer already
swallows `Ctrl-b` (Herdr's prefix is also `Ctrl-b`), `Ctrl-Space` is a fallback
leader that does the same thing.

| Key | Action |
| --- | ------ |
| `Ctrl-b c` | new tab |
| `Ctrl-b n` / `p` | next / previous tab |
| `Ctrl-b 1..9` | go to tab |
| `Ctrl-b Shift-x` | close tab |
| `Ctrl-b ,` | rename tab |
| `Ctrl-b v` / `-` | split right / down |
| `Ctrl-b x` | close pane |
| `Ctrl-b z` | zoom focused pane |
| `Ctrl-b h/j/k/l` | focus pane |
| `Ctrl-b Tab` | cycle panes |
| `Ctrl-b b` or `i` | Inbox (stock session manager) |
| `Ctrl-b o` or `a` | jump to the next pane needing attention |
| `Ctrl-b w` | Spaces navigator (Herdr-style popup) |
| `Ctrl-b Shift-n` | Spaces navigator, opened on the new-space form |
| `Ctrl-b Shift-w` | Rename the current space |
| `Ctrl-b e` | edit scrollback |
| `Ctrl-b [` | scroll mode |
| `Ctrl-b r` / `s` / `t` / `m` / `P` | resize / session / tab / move / pane modes |
| `Ctrl-b q` or `d` | detach |
| `Ctrl-b Ctrl-b` | send a literal `Ctrl-b` to the pane |
| `Ctrl-Space` | fallback leader (same actions) |

The Inbox (`Ctrl-b b` / `i`) opens the stock session manager exactly as before.
Inside it: arrows to select, `Enter` to jump (switching session if needed), `n`
watch, `m` mute, `c` mark seen, `C` dismiss all, `Tab` for the session list.

## Spaces

A Herdr space is a project-level container. In Zellij a space **is a session**:
it owns its own tabs, panes, cwd, agents and attention state, survives detach,
and is restored after a restart by `session_serialization`.

`Ctrl-b w` opens the **Spaces navigator** as a popup over the current tab. It
shows every running space, its rolled-up attention, and each agent or terminal
inside it. The current space is pinned first and marked with `current`.

| Key | Action |
| --- | ------ |
| `↑` / `↓`, `j` / `k` | move between spaces and agents |
| `←` / `→`, `h` / `l` | jump to the previous / next space |
| `Enter` | open the selected space or focus that exact agent pane |
| `/` | search names, agents, tabs, paths and attention text |
| type while searching | filter; `↑` / `↓` still move; `Enter` opens |
| `n` | new space form (name and starting folder) |
| `r` | rename the current space |
| `x` | close the selected space (asks first) |
| `Space` | collapse or expand the selected space |
| `1` … `9` | select the nth visible space |
| `b` / `w` / `d` / `i` / `a` | filter blocked / working / done / idle / all |
| `Tab` / `Shift-Tab` | cycle the attention filter |
| `Ctrl-r` | refresh from Zellij |
| `H` | hook detail for the selected agent (scroll with `↑↓`/`PgUp`/`PgDn`) |
| `?` | full key help |
| `Esc` | clear search, cancel a dialog, or close the navigator |
| mouse | click a row to open it; wheel moves the selection |

Search is a mode, as in Herdr: press `/` to type a filter, `Esc` to clear it and
return to navigation. The selected row is always visible; a scrollbar appears
when the tree is taller than the popup.

New spaces use the selected space's folder when there is one, otherwise the
current space's folder. The form pre-fills a unique name based on the folder;
`Tab` moves between name and folder, `Enter` creates and switches, `Esc`
cancels. Closing the current space is refused; switch away first. Renaming is
limited to the current space because Zellij's plugin API only exposes
`rename_session` for the session the plugin is attached to.

No pane switching is involved: the navigator is a popup, like the Inbox.
`Ctrl-b b` / `i` opens the stock session manager as before; its tab may be Inbox
or Spaces depending on attention. `Ctrl-b w` always opens this navigator.

Create a space from the navigator or from the shell. New sessions do not change
your current layout; they are ordinary Zellij sessions.

### Closing the last pane of a space

A space is a Zellij session, so closing its last pane would normally end the
session and take the attached client out of Zellij with it. Clients started
through `zj` instead hand off to another space: the wrapper follows the server
socket the client is connected to, so it keeps tracking the right space even
after switching sessions inside Zellij. When that space closes and another
space is still running, `zj` attaches to it instead of exiting. Zellij closes
only when the closed space was the last running one.

The handoff target is the space the client was started in when that space is
still running, otherwise the most recently created other running space. Detach
(`Ctrl-b q`) is unchanged: the space keeps running and `zj` exits. Exited
(resurrectable) spaces are never opened automatically: a bare `zj` starts a
fresh space instead, and if the default project name is held by an exited space
it uses the next free name such as `dots-2`. Restore an exited space only
through the navigator or an explicit `zj attach NAME` / `zj space switch NAME`.
That keeps manual restore manual, while still preserving the exited layout for
later.

This lives in the `zj` wrapper rather than the Spaces plugin because the plugin
runs inside the session that is closing and cannot outlive it. The detection
uses the Linux `/proc` filesystem and `ss`; without them `zj` keeps the old
behavior. Sessions started with bare `zellij` do not get this handoff.

### Agent labels in Spaces

Zellij's per-session metadata cache omits program status and terminal commands
for peer sessions. Without help, an idle or done agent in another space is
therefore indistinguishable from a plain terminal. While the navigator is open
it runs `zj __space-snapshot` every few seconds; that helper queries the live
pane list for every running space and returns the agent app plus viewport text
for each agent pane. The navigator merges those labels into its model, never
overriding a label the agent itself reported, and uses the text for the
screen-reader fallback described above. `screen_fallback false` switches the
navigator back to the older `zj __space-agents` app-only call.

Space management from the shell:

```text
zj space list              table: space, tabs, needs/work/done, cwd
zj space current           current space name
zj space new NAME [DIR]    create a space for DIR and switch to it
zj space switch NAME       switch to an existing or exited space
zj space rename NAME       rename the current space
zj space close [NAME] [-y] close a space (asks unless -y)
```

Inside Zellij, `zj space new/switch` launch the navigator with a `target`
configuration; the plugin waits for the session list, switches the client and
closes itself. This avoids the old typing-into-a-form race. Outside Zellij they
attach directly. `zj ls` is the same table as `zj space list`.

## Restoring spaces after a restart

Restoring an exited space from the navigator is now a single action: Zellij
restores every session, tab, pane split, working directory, saved viewport and
scrollback, and runs the saved commands immediately. There is no per-pane
`Waiting to run` banner and no second confirmation.

The pieces:

- `session_serialization`, `serialize_pane_viewport true`,
  `scrollback_lines_to_serialize 2000` and `serialization_interval 30` keep the
  complete layout and enough pane contents to rebuild it.
- Before the navigator switches to an exited space it runs
  `zj __prepare-restore <space>`. That helper removes the `start_suspended`
  marker from the saved layout, so Zellij starts the saved commands instead of
  holding them. It also wraps every interactive agent command in
  `zj __resume-wrap`, even when the saved command already carries `--session`
  or `--resume`, so a restored pane primes itself as idle, gets an agent pane
  title, and appears in the stock Inbox immediately.
- The saved layout is found through Zellij's `session_index.sqlite` first.
  `session-metadata.kdl` is only written on a clean shutdown, so looking for
  metadata alone missed spaces whose server had been killed; the index always
  has the name-to-directory mapping.
- `post_command_discovery_hook "zj __hook"` wraps interactive agent commands
  while a session is serialized. The wrapper, `zj __resume-wrap`, adds the
  agent's resume switch. When the harness exposes its session id, the exact id
  is already embedded in the pane's saved command (see below). The hook wraps
  commands that already have resume flags too, so future saves always carry the
  wrapper rather than a bare `opencode --session ...`.
- Harnesses with lifecycle hooks report the active conversation id to
  `zj hook <agent>` on session start and on prompts (the legacy
  `zj session-hook <agent>` is an alias). That command writes the id into
  Zellij's per-pane resurrect command, so two panes (or two sessions) of the same
  harness in the same working directory restore to their own conversations.
  Hooks are installed for Claude, Codex, Qwen, Gemini, Copilot, Cursor, Grok,
  Kimi and OpenCode; see [Per-agent hooks](#per-agent-hooks).
- For harnesses without hooks (or when a hook has not fired yet), the wrapper
  falls back to the most recent conversation for the pane's working directory.
- `zj attach` outside Zellij uses `--force-run-commands`, so the same automatic
  restore happens when a space is opened from a shell.

Resume switches per harness:

| Harness | Restore invocation |
| --- | --- |
| Claude Code | `claude --resume <id>` (fallback `--continue`) |
| Codex | `codex resume <id>` (fallback `resume --last`) |
| OpenCode | `opencode --session <id>` (fallback `--continue`) |
| Pi | `pi --session <id>` (fallback `--continue`) |
| Qwen Code | `qwen --resume <id>` (fallback `--continue`) |
| Copilot CLI | `copilot --resume <id>` (fallback `--continue`) |
| Kimi Code / Kimi CLI | `--session <id>` (fallback `--continue`) |
| MiniMax Code | `mcode --session <id>` (fallback `--continue`) |
| Muse | `muse resume <id>` (fallback `resume --last`) |
| Mistral Vibe | `vibe --resume <id>` (fallback `--continue`) |
| Gemini CLI | `gemini --session-file <path>` (fallback `--resume latest`) |
| Cline CLI | `cline --id <id>` |
| Cursor CLI | `cursor-agent --resume <id>` (fallback `--continue`) |
| Grok Build | `grok --resume <id>` (fallback `--continue`) |
| Crush | `crush --session <id>` (fallback `--continue`) |
| OMP | `omp --resume=<id>` (fallback `--continue`) |
| Kilo Code | `kilo --session <id>` (fallback `--continue`) |
| Antigravity CLI | `agy --conversation <id>` (fallback `--continue`) |
| DeepSeek Harness | `dsh-tui --resume <id>` (fallback `--continue`) |
| fx | `fx --continue` |

When a harness hook is installed, `<id>` is the exact conversation reported by
that harness and stored on the pane. Without a hook, `<id>` is the most recent
conversation for the pane's working directory, or the fallback "continue latest"
switch shown above.

The exact per-pane id is captured by the harness itself. Claude, Codex, Qwen
and Gemini are configured to call `zj hook <agent> <Event>` (or the legacy
`zj session-hook <agent> start|prompt|stop`) at session start, on prompts and
at turn end (Gemini uses BeforeAgent and AfterAgent); Copilot, Cursor, Grok and
Kimi pass their own session id. The hook payload carries the conversation/session
id and the event drives the pane's idle/working/done status.
Those hooks live in `harnesses/claude/settings.json`,
`harnesses/codex/linux-hooks.json`, `harnesses/qwen/settings.json`,
`harnesses/gemini/settings.json`, and `harnesses/copilot/hooks/zellij.json` and
are linked by `harnesses/setup.sh`. The command looks up the current pane
through `ZELLIJ_PANE_ID` and runs:

```text
zellij action set-resurrect-command --pane-id terminal_N -- \
    zj __resume-wrap <encoded-launch-command> <conversation-id>
```

Zellij stores that as the pane's resurrect command, so it survives serialization
per pane. There is no shared “last conversation” file and no cwd-based
ambiguity.

For other harnesses the helper reads each harness's own local session store,
scoped to the pane's working directory. When a harness has no discoverable id
(or no stable conversation id in the installed version), it uses that harness's
"continue the latest conversation" switch. A world-readable marker or shared
global "last conversation" is never used.

`zj resume-command AGENT [DIR]` prints the discovery result without launching
anything, which is useful when checking a new harness:

```sh
zj resume-command claude ~/dev/dots
zj resume-command codex ~/dev/model-router
```

Limitations:

- Claude, Codex, Qwen, Gemini, and Copilot restore exact per-pane conversations. Other
  harnesses fall back to the most recent conversation for the pane's working
  directory. Two panes of a fallback harness in the same directory restore to
  the same latest conversation. Add a lifecycle hook that calls
  `zj session-hook <agent>` to make another harness exact.
- Commands hidden inside a shell string (`bash -c "claude ..."`) are left alone;
  Zellij only sees the shell, and rewriting arbitrary shell strings is unsafe.
- The stock session manager's saved-session preview still lists commands. Use
  the Spaces navigator or `zj attach` for the automatic path; the helper and
  wrapper are otherwise independent of the UI.
- The navigator requests `RunCommands` in addition to
  `ReadApplicationState`/`ChangeApplicationState`. `scripts/setup-zellij.sh`
  seeds all three permissions for the plugin path so the first open has no
  prompt on a machine installed from this repo.

## `zj` command reference

`zj` maps Herdr workspaces/spaces to Zellij sessions. Inside Zellij every
command targets the **current** space, even one started with bare `zellij` (so
an auto-generated name like `adventurous-cactus` is fine). Outside Zellij it
targets the only running space, then a project-named one, and bare `zj` attaches
to that rather than creating a second space. `ZJ_SESSION` overrides the project
name.

```text
zj                        attach current/single space (create project one if none)
zj attach [NAME]          attach/switch to a named space
zj new NAME               create and attach a named space
zj ls [--names]           spaces table (or just names)
zj kill [NAME]            kill and delete a space
zj agent claude [ARGS]    open claude in a new tab (OSC 7501 wrapped)
zj run -- cargo build     open a command in a new pane
zj inbox                  toggle the Inbox (works outside if a client is attached)
zj next                   jump to the next attention target
zj prompt "continue"      type text + Enter into the focused pane
zj wait needs_you 600     block until the focused pane needs you (optional pane id)
zj read [--full]          dump the focused pane / scrollback
zj panes                  pane + attention state as JSON
zj resume-command AGENT [DIR]  show the resume invocation for the latest conversation
zj doctor                 show config, spaces, target and client state
```

`zj status` must run inside a Zellij pane: OSC 7501 is written to that pane's
pty. Running it from a normal shell has no effect, so it now reports that
instead of silently doing nothing.

`zj agent` uses a generated layout so the tab keeps its name, cwd and output
after the agent exits. When run inside Zellij it adds a tab. Outside Zellij it
creates the session with that layout, or adds the tab to an existing session
(even a detached one) and attaches. Adding a tab to a session with no attached
client needs a viewport, so the helper briefly starts a headless Zellij client
for that operation.

## Verification performed

- `zellij --version` → `zellij 0.46.0`; `zellij setup --check` → config well defined.
- `printf '\e]7501;state=working:progress=40:app=probe:msg=...'` in a pane →
  `list-panes --json` reports `attention.status: Working`, source
  `ProgramStatus`, progress 40, app and message decoded.
- `state=blocked:kind=permission` → `NeedsYou`, text
  `needs approval: Allow edit to config.kdl?`.
- Live agents: Pi 1.5.2 TUI emitted `idle` (`app: pi`) and, after a prompt,
  `Working` with `program_status.state: Working`, then `Done`. OpenCode 2.3.0
  emits `Working` and `Idle` only; its local `zellij-status` plugin now supplies
  `Done` after a turn.
- Fresh versus restore: after saving and killing a space named `zjfresh`, a
  bare `zj` in that directory started a new `zjfresh-2` and left `zjfresh`
  EXITED. An explicit `zj attach zjfresh` restored the exited space instead.
- Restored harness visibility: a saved OpenCode pane whose command was still
  `opencode --standalone` was prepared with `zj __prepare-restore`. The helper
  wrapped it as `zj __resume-wrap ...` and removed `start_suspended`; after
  restore `list-panes` reported `Idle` with `app: opencode` and a remembered
  idle status, and the stock Inbox listed the pane under `Idle` immediately.
- OpenCode done: the local plugin emitted working on `session.execution.started`
  and done after `session.execution.succeeded`. Two standalone OpenCode panes
  in one session kept independent states: prompting one showed `Working` then
  `Done`, while the other stayed `Idle`. Spaces rendered the finished pane as
  `done` and the stock Inbox moved it to `Done`.
- Status priming: a direct `codex resume --last`, a restored
  `zj __resume-wrap b3BlbmNvZGU=` pane, and a `zj __shim opencode opencode` pane
  all settled at `Idle` with `app` set and a remembered idle status. The stock
  Inbox listed the directly launched Codex pane in its `Idle` section.
- Turn status commands: `zj session-hook opencode prompt` emitted a working
  record and `zj session-hook opencode stop` emitted a done record; `start`
  emitted the working+idle prime. Claude, Codex, Qwen and Gemini hook configs
  now pass those events (Gemini through BeforeAgent/AfterAgent).
- Multi-pane OpenCode: three panes started against the shared background
  service all became working when one was prompted; three `--standalone` panes
  kept the unprompted two idle. The shell wrapper, `harness opencode`,
  `zj __shim` and `zj __resume-wrap` now add `--standalone`.
- Done after a turn: a command pane that reported `zj status done` and stayed
  alive showed `✓ done · done: probe done` in Spaces (and Done in the stock
  Inbox) instead of dropping to idle. The navigator follows Zellij's remembered
  Done record, just as the Inbox does.
- Peer agent identity: with the navigator running in a second session, an idle
  Claude pane in the peer space rendered as `claude-code · harnesses idle`
  instead of `terminal`. `zj __space-agents` returned the live app map for both
  spaces; the 2-second session manifest alone omits program status.
- Screen fallback: `cargo test` passes 81 tests, including all 22 vendored herdr
  manifests, Gemini/Cursor blockers, live Claude blocked/working captures, OSC
  authority precedence, native Done stickiness, and working→idle Done synthesis.
  `zj __space-snapshot` returned app, title and viewport text for six agent panes
  across two running sessions in one call. Full popup rendering was not
  re-verified through a headless CLI client; open `Ctrl-b w` once to confirm the
  visual result.
- Key tests through a real PTY client: `Ctrl-b c` created a tab, `Ctrl-b v`
  split a pane, `Ctrl-b x` closed it, `Ctrl-b b` opened the Session Manager,
  `Ctrl-b n` switched tabs, `Ctrl-b o` jumped to a `NeedsYou` pane in another
  tab, `Ctrl-b q` detached while the session kept running; bare `v` and `Alt-n`
  did not trigger Zellij actions while locked.
- `zj wait needs_you` returned `needs_you` after a blocked OSC 7501 report.
- `zj agent sleep 5` produced a `sleep` tab with `Working` state when run inside
  a session, when starting a new session from outside, and when adding a tab to
  an existing detached session.
- Session targeting against a bare `zellij` session (`adventurous-cactus`):
  `Ctrl-b b`/`Ctrl-b i` toggled the Inbox in that session; `zj inbox` toggled it
  from outside with a client attached; `zj doctor` reported that session as the
  target; `zj` inside reported the current session instead of switching away.
- `Ctrl-Space c` opened a tab through the fallback leader.
- Spaces navigator, real PTY: `Ctrl-b w` opened a centered 86%-wide popup
  (103x32 on a 120x40 terminal), current space first, with per-agent status
  lines. A detached space
  showed `blocked` from `GlobalAttentionUpdate` even though its session manifest
  was stale. `←` / `→` jumped between spaces, `Enter` switched to the selected
  agent pane, and the popup closed itself after the switch.
- Search and navigation: `/` + `alpha` filtered to the exact space in the
  search line; `Enter` switched sessions. `?` opened the key help, `Esc`
  returned. `n` opened the new-space form; creating `gamma` started a real
  session and switched the client.
- Space lifecycle: `r` renamed the current space, `x` + `Enter` killed a
  non-current space and deleted its saved state, and a kill of the current
  space was refused with a footer message.
- `zj space switch alpha` and `zj space new three /tmp` launched the navigator
  with a target, switched the attached client within about a second, and left
  no plugin pane behind. The new session's shell started in `/tmp`.
- Space handoff: a `zj` client started in one space, switched to a second and
  closed that space's last pane reattached to the original space instead of
  exiting. `Ctrl-b q` exited the client while leaving both spaces running, and
  closing the initial space's last pane switched to another running space.
- `Ctrl-b b` / `i` still opened the stock session manager on its Inbox tab with
  `Needs You` and `Working` sections; `Esc` closed it.
- Key isolation and split panes: with two shell panes side by side, opening the
  navigator, typing `LEAK7` into the new-space form and closing left both
  shells' screens and prompts unchanged. The popup was centered over the split
  (86x24 on a 100x30 terminal) and no text crossed the popup or split borders.
  After closing, input worked in the focused split pane immediately. Clicking
  outside the popup also closed it and returned input to the terminal.
- Per-agent hooks: `python3 -m unittest discover -s zj/tests -t zj` run from
  `~/dev/zellij-spaces`
  passes 15 tests covering provider normalization, merge semantics, the
  store/spool, the UDS/TCP listener and the `zj hook` client. The 81 plugin
  tests include hook precedence, detail deserialization and the `H` overlay.
- 81 plugin unit tests cover attention roll-up, search ranking, navigation,
  filters, dialogs, session deduplication, live agent-app merging, remembered
  Done handling, exited-space limits, path/width handling and full-width ANSI
  rendering.
- Resurrection: a saved session with a direct `claude` command was prepared
  with `zj __prepare-restore`, then attached without `--force-run-commands`.
  The pane started immediately (no `Waiting to run` banner) and the restored
  process received `--resume <latest-claude-session-id>` for its cwd.
- Exact per-pane ids: two panes in one session, same cwd, ran agents that
  reported ids `conversation-A` and `conversation-B`. The saved layout carried
  one `__resume-wrap ... conversation-A` and one `... conversation-B`. After a
  simulated restart and `zj __prepare-restore`, both processes came back with
  their own `--resume` value and neither pane was held.
- The same test through the Spaces navigator switched the attached client from
  another session to the restored space, with the pane live and no prompt.
- `zj resume-command` returned the expected id and switch for Claude, Codex,
  OpenCode, Pi, Qwen, Copilot, Kimi, Muse, Vibe, Cline, Grok, Kilo, Antigravity
  and fx on this machine; unavailable stores fall back to each harness's
  continue-latest switch.

## Notes and limits

- OSC 7501 works on a per-pane basis. `done` and `error` are *attention*
  states: once the pane has been looked at, Zellij resolves it to `idle` while
  keeping the record in `program_status` (visible in `zj panes`). To see the
  marker, let the agent finish in a background tab. After the process exits,
  Zellij's command-pane heuristic can also report a long-enough run as done
  (`done_threshold_secs`).
- The stock Inbox only lists panes that reported a non-idle state at least
  once. Launching an agent therefore writes a very short `working` record and
  then `idle`; the agent remains visible under `Idle` without a fake working
  turn. Agents that stop reporting after launch stay idle until their next
  status record.
- Gemini/Qwen/Cursor lack OSC 7501 in the versions installed here. Their
  notifications still reach the Inbox as "needs you", `zj agent` adds
  working/done/error around the process, and the Spaces navigator now infers
  their working/blocked/idle state from the viewport. Enable notifications in
  each tool (Gemini's `general.enableNotifications` is tracked in this repo).
- The Spaces screen fallback needs no new plugin permission: it reads panes
  through the `zj __space-snapshot` helper (`zellij --session … dump-screen`),
  not through `get_pane_scrollback`. It is navigator-only; the tab bar and stock
  Inbox still show native OSC/attention state.
- The Spaces navigator is a standalone plugin (source in the
  `~/dev/zellij-spaces` repo), not a
  patched session manager. Its permissions are cached by absolute WASM path; if
  you move or rename the WASM, Zellij will ask again.
- The navigator merges `GlobalAttentionUpdate` over `SessionUpdate`, because
  Zellij can serve a stale manifest for a detached peer session while its
  attention stream is current. Duplicate session snapshots (possible during a
  rename) are collapsed to the richest one.
- Only the five most recent exited/resurrectable spaces are shown. The stock
  session manager remains the place for the complete saved-session list.
- Renaming is limited to the current space; closing the current space is
  refused. This is a plugin-API limit, not a UI choice.
- `zj` sessions are ordinary Zellij sessions; the navigator lists all of them.
  There is no persistent sidebar to manage or hide.
- The automatic switch on closing a space's last pane is a `zj` client-wrapper
  feature. It needs `ss` and `/proc` (Linux); bare `zellij` clients and other
  operating systems keep Zellij's normal behavior.
- Zellij sessions are global, while `zj` names them after the project directory.
  Two projects with the same basename should set `ZJ_SESSION`.
- Desktop notifications and the tray icon are enabled in the config but require
  the Zellij desktop terminal (`zellij terminal`). They are optional.
