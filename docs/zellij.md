# Zellij as an agent terminal multiplexer

This setup configures Zellij 0.46 as a Herdr-style workspace manager for coding agents. Agent state travels over OSC 7501, the Program Status Protocol, so Zellij displays which agent is working, blocked, done, or failed without scraping pane output.

| Herdr concept | Zellij equivalent |
| --- | --- |
| Space (project) | Session (one per project, resurrectable) |
| Tab | Tab (one per agent or task) |
| Pane | Pane |
| Spaces popup | Spaces navigator (`Ctrl-b w`) |
| Agent inbox | Stock Inbox (`Ctrl-b i` or `Ctrl-b b`) |
| Agent status (OSC 7501) | Native attention markers on frames, tabs, session indicator, and cross-session Inbox |
| Prefix key (`Ctrl-b`) | `Ctrl-b` leader, locked by default |

## Native OSC 7501 and the Spaces navigator

Zellij 0.46 parses OSC 7501 inside its terminal server before output reaches any plugin. The server stores per-pane program-status records, resolves them into attention states, and renders those states in pane frames, the tab bar, the session indicator, and the stock Inbox. The OSC 7501 and Inbox side relies on built-in Zellij behavior.

The Spaces navigator is a standalone Rust WASM plugin. By default, [`scripts/setup-zellij.sh`](../scripts/setup-zellij.sh) reads the compiled WASM from `~/dev/model-router/plugins/zellij-spaces`. Set `ZJ_SPACES_REPO` to override that path, such as `~/dev/zellij-spaces` for a standalone checkout. The navigator is separate from Zellij's built-in session manager and does not occupy a persistent sidebar.

The navigator presents workspaces in a Herdr-style tree:

- Spaces appear as grouped rows with their agents and terminals indented underneath.
- Each row shows live attention state, the agent or app name, the tab name, the attention message, and the age of that state.
- Space rows roll up their agents' states so a blocked agent is visible before you open the space.
- `Enter` jumps directly to the selected space or agent pane.
- `/` filters by space name, agent app, tab name, path, and attention text.
- `b`, `w`, `d`, and `i` filter by blocked, working, done, or idle state.
- Creating, renaming, closing, and collapsing spaces happen inline inside the popup.

The UI uses the Catppuccin Mocha palette to match `theme "catppuccin-mocha"` in `zellij/config.kdl`. It draws a rounded lavender border, a `/` search bar with filter chips, grouped space rows with status counts and paths, tree-style agent and terminal rows with messages and ages, a bottom detail line for the selected row, and a compact key-hint footer.

The plugin source lives across six modules:

- `src/model.rs` holds the data model and search ranking.
- `src/app.rs` implements the pure event-to-effect state machine.
- `src/canvas.rs` implements the cell buffer and truecolor ANSI renderer.
- `src/ui.rs` draws the layout and dialogs.
- `src/screen/` runs the viewport manifest evaluator.
- `src/main.rs` owns the Zellij host boundary.

The plugin subscribes to `SessionUpdate` for tabs, panes, and viewport data, and to `GlobalAttentionUpdate` for cross-session agent state, including detached spaces. It requests `ReadApplicationState`, `ChangeApplicationState`, and `RunCommands`. Zellij caches those grants by absolute plugin path in `~/.cache/zellij/permissions.kdl`.

The keybindings in `zellij/config.kdl` send `MessagePlugin "spaces"` to load a short-lived launcher without changing pane focus or floating pane visibility. The launcher opens the navigator through Zellij's popup API and exits. The popup captures all keyboard input and closes if you click outside it.

`Ctrl-b w` opens the navigator at roughly 86% of the terminal width and 82% of its height. `Ctrl-b i` and `Ctrl-b b` toggle Zellij's built-in session card and Inbox.

## Install

[`scripts/setup-zellij.sh`](../scripts/setup-zellij.sh) downloads the official `v0.46.0` archive for `Linux-x86_64`, `Linux-aarch64`, `Darwin-arm64`, or `Darwin-x86_64` and checks its SHA-256 against the release checksum file. It backs up existing regular files into `~/.local/share/dots-backups/zellij-<timestamp>`, installs `~/.local/bin/zellij` and `~/.config/zellij/plugins/zellij-spaces.wasm`, and creates compatibility symlinks for `zellij-spacesy.wasm` and `zellij-spaces-panel.wasm`. It also seeds `~/.cache/zellij/permissions.kdl`, symlinks `zellij/config.kdl` to `~/.config/zellij/config.kdl`, and symlinks `~/.local/bin/harness` to `~/.local/bin/zj`.

```sh
bash scripts/setup-zellij.sh
zellij --version          # zellij 0.46.0
zellij setup --check      # CONFIG FILE: Well defined.
```

If your plugin checkout lives at `~/dev/zellij-spaces` instead of the default path, pass `ZJ_SPACES_REPO=~/dev/zellij-spaces bash scripts/setup-zellij.sh`.

On `x86_64` Linux (musl), the extracted `v0.46.0` binary has SHA-256 `afb6bfb5f9fdc3d9e9d094ce21866bc99d37073a5fbe75bb612e21fc9b96033d`. The manual steps are:

```sh
mkdir -p ~/.local/bin ~/.config/zellij/plugins
curl -fL -o /tmp/zellij.tar.gz \
  https://github.com/zellij-org/zellij/releases/download/v0.46.0/zellij-x86_64-unknown-linux-musl.tar.gz
tar xzf /tmp/zellij.tar.gz -C /tmp
install -m 755 /tmp/zellij ~/.local/bin/zellij
install -m 644 ~/dev/model-router/plugins/zellij-spaces/zellij-spaces.wasm \
  ~/.config/zellij/plugins/zellij-spaces.wasm
ln -sfn "$PWD/zellij/config.kdl" ~/.config/zellij/config.kdl
ln -sfn ~/.local/bin/harness ~/.local/bin/zj
```

To rebuild the plugin after editing its Rust source, run `build.sh` inside the plugin repository:

```sh
bash ~/dev/model-router/plugins/zellij-spaces/build.sh
```

That script installs the `wasm32-wasip1` Rust target if missing, runs the plugin unit tests, compiles the WASM binary, and updates the tracked artifact. Run `bash scripts/setup-zellij.sh` afterwards to copy the rebuilt WASM into `~/.config/zellij/plugins/`.

## OSC 7501 wiring

Zellij answers the protocol probe `ESC ] 7501 ; ? ST` and keeps a record per pane, keyed by `id` inside that pane. The `attention` block in `zellij/config.kdl` enables the full attention pipeline:

```kdl
attention {
    enabled true
    frame_markers true
    tab_markers true
    host_status true
    idle_rule "all"
    track_commands true
    done_threshold_secs 10
    idle_activity_secs 10
    idle_silence_secs 3
    startup_card "attention"

    desktop_notifications {
        needs_you true
        failed true
        done true
    }

    notification_details true
}
```

With `host_status true`, Zellij re-emits the most urgent status record to the outer terminal as OSC 7501. Supporting outer terminals and nested Zellij sessions receive agent status, while terminals that do not recognize OSC 7501 ignore the escape sequence.

### Agents that report natively

The table below lists installed coding agents and their OSC 7501 behavior, checked against the upstream integration list and local installs on this machine.

| Agent | Local version | Status behavior |
| --- | --- | --- |
| Claude Code | 2.1.325 | Reports natively while the TUI runs |
| Codex CLI | 0.169.0 | Reports done and failed since 0.164 |
| OpenCode | 2.3.0 | Reports working, waiting, and idle; the local `zellij-status` plugin adds done |
| Pi | 1.5.2 | Live-verified to emit `state=idle:app=pi`, then `working`, then `done` |
| Crush | 0.101.0 | Reports natively |
| Grok Build | 1.0.77 | Reports natively |
| omp | 18.12.6 | Reports natively |
| GitHub Copilot CLI | 1.0.112 | Reports natively |
| Kimi Code | 2.5.3 | Requires `PI_PROGRAM_STATUS=1`, exported in `.zshrc` |
| Muse | installed | Reports `idle`, `working`, `blocked`, `done`, and `error` with `app=muse` |
| MiniMax Code | 0.7.7 | Reports with `:app=mcode` |
| fx | installed | Reports from `src/ui/terminal/program_status.zig` |
| Mistral Vibe | 3.1.2 | Bundled `vibe-rs` reports with `:app=vibe` |
| DeepSeek / dsh-tui | 0.20.1 | Reports with `:app=dsh-tui`; `DSH_TUI_PROGRAM_STATUS=0` disables it |
| Qwen Code | 0.26.2 | Below 0.26.9; falls back to hooks, notifications, and viewport scanning |
| Gemini CLI | 0.68.0 | No OSC 7501 yet; `general.enableNotifications` is enabled |
| Cursor CLI | current | No OSC 7501; hooks and notifications handle status |

Native reporters need no per-agent plugin for OSC 7501 because Zellij parses escape sequences straight from each pane PTY. Prompt and command boundaries for plain shell commands come from `.zshrc` hooks. `__zj_osc133_precmd` emits OSC 133 `D` and `A` plus OSC 7 working-directory reports, and `__zj_osc133_preexec` emits OSC 133 `C`.

### Screen-reader fallback in Spaces

The Spaces navigator treats OSC 7501 as the primary signal and inspects pane viewports for agents that do not emit OSC 7501. While the popup is open, it calls `zj __space-snapshot` about every five seconds. That command runs `zellij --session NAME action dump-screen` for every agent pane across all running spaces, so peer and detached sessions are readable without a new plugin permission or a blocking `get_pane_scrollback` call. The plugin evaluates the dumped text with a trimmed port of [Herdr](https://github.com/herdrdev/herdr)'s manifest engine and vendored Apache-2.0 agent manifests. Provenance is recorded in `src/screen/NOTICE` in the plugin repository.

Status resolution follows this order:

1. A visible permission or question prompt on screen overrides OSC 7501.
2. Otherwise, a live non-idle OSC 7501 state wins for authority agents: Claude, Codex, OpenCode, Pi, Crush, Grok, omp, Copilot, Kimi, Muse, MiniMax Code, fx, Vibe, and DeepSeek/dsh-tui.
3. For Gemini, Cursor, Qwen, Cline, Antigravity, and Kilo, the viewport supplies `working`, `needs you`, or `idle`.
4. A screen-detected transition from `working` to `idle` displays `Done` in the navigator until the pane is focused or ten minutes pass while that navigator instance stays open.

This fallback applies only inside the Spaces navigator. Zellij's tab bar and stock Inbox continue to display native attention state. Setting `screen_fallback false` in the `spaces` plugin block disables viewport scanning and restores the older `zj __space-agents` app-label path.

After updating the vendored TOML manifests in the plugin repo, regenerate the embedded bundle:

```sh
python3 ~/dev/zellij-spaces/scripts/sync-screen-manifests.py
```

Embedding the `regex` crate and manifests brings the compiled WASM to about 2.5 MB. Because `dump-screen` captures the current viewport, a pane that a client has scrolled up may read inaccurately until scrolled back to the bottom. Plain-idle evidence requires two consecutive matching observations to avoid false transitions.

### Per-agent hooks

OSC 7501 carries only a state and a one-line message. The Spaces navigator also reads structured per-agent hook events so a pane can display the active tool, the user prompt, the assistant reply, the model name, the provider session ID, blocked kinds, and subagents. This follows Orca's per-agent hook model without an Electron runtime.

Both `spaces` and `zj` are entry-name symlinks to `~/.local/bin/harness` (the Go CLI from `~/dev/model-router`). The hook configurations tracked under `harnesses/` call `spaces hook <agent> [Event]` (and `spaces session-hook` for legacy Codex and Copilot entries), while the Spaces plugin polls `zj __space-hooks`.

```text
agent hook --stdin JSON--> spaces hook <agent> [Event]
                             |-- normalize the provider payload
                             |-- emit OSC 7501 (main events only)
                             |-- record the resume session ID
                             |-- POST HTTP over a Unix socket
                             +-- on failure: append to a JSONL spool
                                             and start the listener

Spaces navigator --1.5 s--> zj __space-hooks          (small summaries)
                 --H------> zj __space-hooks --detail SESSION PANE
```

Runtime files live in a mode `0700` directory under `$XDG_RUNTIME_DIR` (`${XDG_RUNTIME_DIR:-/tmp}/spaces` when run via `spaces-hooks.service` or `${XDG_RUNTIME_DIR:-/tmp}/zj` by default for `zj`):

| File | Purpose |
| --- | --- |
| `h.sock` | HTTP-over-Unix-socket listener (mode `0600`) |
| `hooks.endpoint.json` | Socket path, optional TCP port, auth token, and PID (mode `0600`) |
| `hooks.store.json` | Merged per-pane hook records (mode `0600`, ephemeral) |
| `hooks-spool.jsonl` | Events buffered while the listener is down |
| `hooks.server.lock`, `hooks.store.lock` | Single-instance and store write locks |
| `hooks.log` | Listener log (1 MiB rotation) |

The listener speaks HTTP/1.1 over the Unix socket on `POST /v1/hook` (or `/v1/hook/<agent>`) with an `X-ZJ-Token` header, plus an optional loopback TCP port. Request bodies are capped at 256 KiB, the socket is mode `0600`, and on Linux the listener checks that the peer UID matches its own UID. Direct HTTP clients must pass `session` and `pane` in the JSON body, query string, or `X-ZJ-Session` and `X-ZJ-Pane` headers.

Running the listener ahead of time is optional. When the socket is missing, `spaces hook` or `zj hook` appends the event to the spool and launches `hook-server --daemon` in the background, throttled to once every ten seconds. The daemon drains the spool at startup and every second afterwards. Use `zj hooks start|stop|status` (or `spaces hooks start|stop|status`) to control it manually.

[`harnesses/setup.sh`](../harnesses/setup.sh) links [`harnesses/shared/spaces-hooks.service`](../harnesses/shared/spaces-hooks.service) to `~/.config/systemd/user/spaces-hooks.service` as an optional systemd user unit:

```sh
systemctl --user enable --now spaces-hooks.service
```

Environment variables `ZJ_HOOKS_RUNTIME`, `ZJ_HOOKS_STORE`, and `ZJ_HOOKS_DISABLE=1` (and `SPACES_HOOKS_RUNTIME` in the systemd unit) override the defaults. Because the store lives inside the user runtime directory, prompts, tool inputs, and replies stay off persistent storage unless you point `ZJ_HOOKS_STORE` at a disk path.

#### Installed agent hooks

`harnesses/setup.sh` links each hook config into its target location. Every command is observational (`spaces hook ... >/dev/null 2>&1 || true`, with JSON stdout where the tool requires it) and exits `0`. No hook approves or denies permissions.

| Agent CLI | Tracked config | Hook events |
| --- | --- | --- |
| Claude Code | `harnesses/claude/settings.json` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification, Stop, StopFailure, SubagentStart, SubagentStop, PostCompact, TeammateIdle, Elicitation, ElicitationResult, SessionEnd |
| Qwen Code | `harnesses/qwen/settings.json` | SessionStart, SessionEnd, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification, SubagentStart, SubagentStop, Stop, StopFailure |
| Codex | `harnesses/codex/linux-hooks.json` | SessionStart, Stop, UserPromptSubmit (legacy `spaces session-hook`), PreToolUse, PostToolUse, PermissionRequest, SubagentStart, SubagentStop, Interrupt |
| Gemini | `harnesses/gemini/settings.json` | SessionStart, SessionEnd, BeforeAgent, BeforeTool, AfterTool, AfterAgent, Notification (timeout in milliseconds; prints `{}`) |
| Copilot | `harnesses/copilot/hooks/zellij.json` | sessionStart, userPromptSubmitted (legacy `spaces session-hook`), sessionEnd, preToolUse, postToolUse, postToolUseFailure, permissionRequest, notification, subagentStart, subagentStop, preCompact, stop, errorOccurred |
| Cursor | `harnesses/cursor/hooks.json` | sessionStart, sessionEnd, beforeSubmitPrompt, postToolUse, postToolUseFailure, afterAgentResponse, stop, subagentStart, subagentStop (no permission-bearing pre-tool events; prints `{}` or `{"continue":true}`) |
| Grok | `harnesses/grok/hooks/zellij.json` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, Notification, Stop, StopCancelled, StopFailure, SessionEnd |
| Kimi Code | `harnesses/kimi-code/config.toml` | SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Stop, StopFailure |
| OpenCode | `harnesses/opencode/plugins/zellij-status.ts` | `session.execution.started`, `session.execution.succeeded`, `session.execution.interrupted`, `session.execution.failed`, `session.idle`, and tool `execute.before` / `execute.after` |

Grok reads `~/.claude/settings.json` and `~/.cursor/hooks.json` by default. The hook command checks for `GROK_SESSION_ID` and `GROK_HOOK_EVENT` in the environment and attributes those events to Grok so shared Claude or Cursor hooks do not mislabel a Grok session. A 500 ms deduplication window drops duplicate events fired across shared configs.

Codex keeps the three legacy `session-hook` command strings so their existing trust hashes remain valid. Approve the newer hook entries once in Codex's hooks UI instead of hand-editing trust hashes in `harnesses/codex/linux-config.toml`.

#### Precedence

Fresh hook records take priority over OSC 7501 and screen scanning. OSC 7501 fills in when hooks are absent, disabled, or older than a state transition the program already made.

- `working`, `blocked`, and `waiting` hook states remain fresh for 2 minutes after the last event.
- `done`, `failed`, and `blocked` with kind `permission` or `question` persist for 30 minutes.
- `blocked` with kind `auth` persists for 60 minutes.
- A new `working` event clears a turn-scoped block. An `auth` block survives turn end and clears on the next prompt or session boundary.
- A newer OSC 7501 state beats an older hook state. A hook `idle` state never overrides a live OSC 7501 or screen state.
- A visible permission or question prompt on screen overrides any hook state.
- Focusing a pane does not clear hook `done`. Zellij's remembered native `Done` state behaves the same way.

#### Navigating hook state

Pane rows in the Spaces navigator show a hook preview (`Bash · cargo test`, `approve Write ...`, or the latest reply). The bottom detail bar appends blocked kind, event name, model, short session ID, and subagent count when width allows. Press `H` to open the full hook overlay for the selected agent pane:

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

Use `Up`, `Down`, `PgUp`, and `PgDn` to scroll the overlay, and press `H` or `Esc` to close it. The plugin fetches detailed records only when you open the overlay so the 1.5-second summary poll stays lightweight.

#### Diagnostics

```sh
zj hooks status                 # listener, store, and spool state
zj __space-hooks | jq           # summaries for the navigator
zj __space-hooks --detail S P   # one full record
printf '%s' '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}' \
  | ZELLIJ_SESSION_NAME=probe ZELLIJ_PANE_ID=1 zj hook claude PreToolUse
```

If the listener has not started, `zj hooks status` reports the queued spool count. Starting the server with `zj hooks start` or `systemctl --user start spaces-hooks.service` drains the spool. When the recorded PID no longer exists, the next startup replaces the stale socket and endpoint file.

### Agents or scripts without native support

`zj status` writes OSC 7501 directly to the pane's controlling terminal:

```sh
zj status working "indexing repository"
zj status blocked "approve the migration?" permission
zj status done "12 files updated"
zj status error "build failed"
zj status clear
```

Agents started through `zj agent`, restored panes (`zj __resume-wrap`), and direct launches from an interactive Zsh shell (`__zj_prime_agent` in `.zshrc`) all start in the `idle` state rather than `working`. The launcher emits a brief `working` record followed immediately by `idle` because Zellij's stock Inbox only tracks panes that reported at least one non-idle state. That keeps a new or resumed conversation listed under `Idle` in the Inbox.

During a turn, native OSC 7501 reporters emit `working` and `done`. Where an agent exposes lifecycle hooks, `spaces hook <agent> [Event]` (or `zj hook`) normalizes the payload, emits the matching OSC 7501 status, and stores the structured hook record described in [Per-agent hooks](#per-agent-hooks). `prompt` sets `working`, `stop` sets `done`, `start` primes `idle`, and `permission` sets `blocked`. The legacy `session-hook <agent> start|prompt|stop` command remains supported as an alias.

OpenCode requires two extra adjustments:

1. OpenCode's shared background service broadcasts OSC 7501 records to every connected TUI, which would mark every OpenCode pane as `working` when only one pane runs a turn. The `opencode()` shell function in `.zshrc`, `harness opencode`, `zj agent opencode`, and restored panes pass `--standalone` so each pane runs a private server. Existing OpenCode panes started against the shared service keep using it until restarted. Pass `opencode --server <url>` or run `command opencode` if you intentionally want a pane to join the shared background service.
2. OpenCode 2.3 emits `working` and `idle`, never `done`. The plugin at `harnesses/opencode/plugins/zellij-status.ts` (linked into `~/.config/opencode/plugins/zellij-status.ts` by `harnesses/setup.sh`) subscribes to server execution events and sends `Stop` or `StopFailure` at 400 ms and 1500 ms after an execution ends. That delay lets OpenCode's own `idle` record land first so `done` remains the pane's final status. Restart OpenCode panes to load plugin changes.

`zj status` remains available for shell scripts and tools that lack both native OSC 7501 output and lifecycle hooks.

## Keybindings

Zellij starts in Locked mode (`default_mode "locked"` with `keybinds preset="unlock-first" unlock="Ctrl b"`) so agent TUIs receive every keystroke. `Ctrl-b` enters Tmux command mode, and every action switches back to Locked. `Ctrl-Space` is bound as a fallback leader with the same actions for terminals or outer multiplexers that already intercept `Ctrl-b`. While locked, `zellij/config.kdl` unbinds `Alt` chords (`n`, `f`, `i`, `o`, `h`, `j`, `k`, `l`, arrow keys, `=`, `+`, `-`, `[`, `]`, `p`, and `Shift-p`) so agent TUIs can use them.

| Key | Action |
| --- | --- |
| `Ctrl-b c` | New tab |
| `Ctrl-b n` / `p` | Next or previous tab |
| `Ctrl-b 1` to `9` | Go to tab 1 through 9 |
| `Ctrl-b Shift-x` | Close tab |
| `Ctrl-b ,` | Rename tab |
| `Ctrl-b v` / `-` | Split pane right or down |
| `Ctrl-b x` | Close focused pane |
| `Ctrl-b z` | Toggle fullscreen zoom on focused pane |
| `Ctrl-b h/j/k/l` or arrow keys | Move focus between panes |
| `Ctrl-b Tab` | Cycle to next pane |
| `Ctrl-b b` or `i` | Toggle stock Inbox and session manager |
| `Ctrl-b o` | Jump to next pane needing attention |
| `Ctrl-b a` | Open agents selector (`harness agents --json`); tools with profiles open a second level, and `Enter` opens in a new tab |
| `Ctrl-b w` | Open Spaces navigator popup |
| `Ctrl-b Shift-n` | Open Spaces navigator on the new-space form |
| `Ctrl-b Shift-w` | Rename the current space |
| `Ctrl-b e` | Edit scrollback |
| `Ctrl-b [` | Enter scroll mode |
| `Ctrl-b r` / `s` / `t` / `m` / `P` | Enter resize, session, tab, move, or pane mode |
| `Ctrl-b q` or `d` | Detach from session |
| `Ctrl-b Ctrl-b` | Send a literal `Ctrl-b` to the focused pane |
| `Ctrl-b Esc` or `Enter` | Return to Locked mode |
| `Ctrl-Space` | Fallback leader with identical actions |

Sub-modes return to Locked mode instead of Normal mode when you press their exit keys (`r` in resize, `s` in scroll, `o` in session, `t` in tab, `m` in move, and `p` in pane).

Inside the stock Inbox (`Ctrl-b b` or `Ctrl-b i`), use the arrow keys to select an entry, `Enter` to jump (switching sessions if needed), `n` to watch, `m` to mute, `c` to mark seen, `C` to dismiss all, and `Tab` to switch to the session list.

## Spaces

Each space maps to a Zellij session. A session owns its tabs, panes, working directory, running agents, and attention state. It survives detaching and restores after a restart through `session_serialization`.

`Ctrl-b w` opens the Spaces navigator popup over the current tab. It lists every running space with its rolled-up attention state and its child agents or terminals. The current space is pinned at the top with a `current` marker.

| Key | Action |
| --- | --- |
| `Up` / `Down` or `j` / `k` | Move between spaces and agents |
| `Left` / `Right` or `h` / `l` | Jump to previous or next space |
| `Enter` | Switch to selected space or focus that exact agent pane |
| `/` | Enter search mode (matches names, agents, tabs, paths, and attention text) |
| Typing while searching | Filter rows (`Up` / `Down` still move selection, `Enter` opens) |
| `n` | Open new-space form (name and starting directory) |
| `r` | Rename the current space |
| `x` | Close the selected space (prompts for confirmation) |
| `Space` | Collapse or expand the selected space |
| `1` to `9` | Select the nth visible space |
| `b` / `w` / `d` / `i` / `a` | Filter by blocked, working, done, idle, or all |
| `Tab` / `Shift-Tab` | Cycle attention filter |
| `Ctrl-r` | Refresh state from Zellij |
| `H` | Open hook detail overlay for the selected agent |
| `?` | Toggle full key help |
| `Esc` | Clear search, cancel dialog, or close the navigator |
| Mouse | Click a row to open it; scroll wheel moves selection |

Search is modal. Press `/` to enter a query, and press `Esc` to clear the query and return to normal navigation. The selected row stays scrolled into view, and a scrollbar appears when the list exceeds the popup height.

The new-space form defaults to the selected space's folder when one is selected, or the current space's folder otherwise. It pre-fills a unique name derived from that folder. Press `Tab` to move between name and folder, `Enter` to create and switch, or `Esc` to cancel. Closing the current space is refused until you switch to another space. Renaming applies only to the current space because Zellij's plugin API exposes `rename_session` only for the attached session.

New spaces are regular Zellij sessions and do not alter your existing layout. `Ctrl-b b` and `Ctrl-b i` always open the stock session manager, whereas `Ctrl-b w` always opens the Spaces navigator.

### Closing the last pane of a space

Because a space is a Zellij session, closing its last pane normally terminates the session and exits the attached client. Clients launched through `zj` hand off to another running space instead. The wrapper tracks the server socket the client is attached to, so it follows session switches made inside Zellij. When that space exits and at least one other space is still running, `zj` attaches to the next space rather than exiting. The client exits only when the closed space was the last running one.

The handoff prefers the space the client originally started in if it is still running, and otherwise picks the most recently created running space. Detaching with `Ctrl-b q` leaves all spaces running and exits `zj`. Bare `zj` never reopens an exited (resurrectable) space. If the project basename is already held by an exited space, bare `zj` creates a new name such as `dots-2` and leaves the exited layout available for manual restore through the navigator, `zj attach NAME`, or `zj space switch NAME`.

This handoff runs inside the `zj` client wrapper because a plugin running inside the closing session exits with that session. On Linux, detection reads `/proc` and runs `ss`. Without them, or when using bare `zellij` directly, Zellij uses its standard exit behavior.

### Agent labels in Spaces

Zellij's per-session metadata cache omits program status and terminal commands for peer sessions. Without a supplemental lookup, an idle or done agent in another space looks identical to a plain shell. While open, the Spaces navigator calls `zj __space-snapshot` every few seconds to query the live pane list and viewport text for all running spaces. The navigator merges discovered app labels into its model without overriding a label reported directly by the agent, and passes the viewport text to the screen-reader fallback. Setting `screen_fallback false` switches to the older `zj __space-agents` app-only query.

Space management from the shell:

```text
zj space list              table of space, tabs, needs/work/done counts, and cwd
zj space current           current space name
zj space new NAME [DIR]    create a space for DIR and switch to it
zj space switch NAME       switch to an existing or exited space
zj space rename NAME       rename the current space
zj space close [NAME] [-y] close a space (prompts unless -y is passed)
```

Inside Zellij, `zj space new` and `zj space switch` invoke Zellij's native `switch-session` action. Outside Zellij, they attach directly. `zj ls` prints the same table as `zj space list`.

## Restoring spaces after a restart

Restoring an exited space from the Spaces navigator takes one action. Zellij restores every tab, pane split, working directory, saved viewport, and scrollback buffer, and starts the saved commands immediately without a per-pane `Waiting to run` banner.

This relies on five parts working together:

- `zellij/config.kdl` enables `session_serialization true`, `serialize_pane_viewport true`, `scrollback_lines_to_serialize 2000`, and `serialization_interval 30`.
- Before switching to an exited space, the navigator runs `zj __prepare-restore <space>`. That helper strips `start_suspended` from the saved layout so Zellij runs the saved commands immediately, and wraps interactive agent commands in `zj __resume-wrap` even if they already include `--session` or `--resume`. Wrapping ensures every restored pane primes itself as `idle`, receives an agent pane title, and appears in the stock Inbox right away.
- `zj __prepare-restore` locates the saved layout via Zellij's `session_index.sqlite` first. `session-metadata.kdl` is written only on clean shutdown, so checking the SQLite index handles spaces whose server was killed.
- `post_command_discovery_hook "zj __hook"` in `zellij/config.kdl` wraps interactive agent commands during serialization. `zj __resume-wrap` adds the agent's resume flag or uses the exact conversation ID stored on the pane.
- `zj attach` outside Zellij passes `--force-run-commands` so shell-initiated restores behave the same way.

Resume invocations per agent CLI:

| Agent CLI | Restore invocation |
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
| DeepSeek / dsh-tui | `dsh-tui --resume <id>` (fallback `--continue`) |
| fx | `fx --continue` |

When lifecycle hooks are installed for an agent, `<id>` is the exact conversation ID reported by that agent on session start and prompt events. The hook command reads `ZELLIJ_PANE_ID` and saves the per-pane resurrect command in Zellij:

```text
zellij action set-resurrect-command --pane-id terminal_N -- \
    zj __resume-wrap <encoded-launch-command> <conversation-id>
```

Because Zellij stores that command per pane, two panes of the same agent in the same working directory restore to their own conversations without sharing a global "last conversation" file.

For agents without lifecycle hooks (or before a hook fires), `zj __resume-wrap` queries the agent's own local session store scoped to the pane's working directory. If no session ID is discoverable, it falls back to that tool's continue-latest flag shown in the table above.

To inspect the resolved resume command without launching anything, run `zj resume-command`:

```sh
zj resume-command claude ~/dev/dots
zj resume-command codex ~/dev/model-router
```

Known restore limits:

- Agents with lifecycle hooks (Claude, Codex, Qwen, Gemini, Copilot, Cursor, Grok, Kimi, and OpenCode) record exact per-pane conversation IDs. Other tools fall back to the most recent conversation in the pane's working directory, so two panes of a fallback tool in the same directory restore to the same conversation.
- Commands nested inside a shell string (`bash -c "claude ..."`) are left unchanged because Zellij only sees the outer shell command.
- Zellij's built-in session manager preview still lists saved commands and prompts before running them. Use the Spaces navigator or `zj attach` for immediate restore.
- The navigator requires `RunCommands` in addition to `ReadApplicationState` and `ChangeApplicationState`. `scripts/setup-zellij.sh` seeds all three permissions in `~/.cache/zellij/permissions.kdl` so the first open does not prompt.

## Command reference for `zj`

Inside Zellij, every `zj` command targets the current session, including sessions started with bare `zellij` and auto-generated names such as `adventurous-cactus`. Outside Zellij, `zj` targets the sole running space if only one exists, and otherwise selects the space named after the current project directory. Bare `zj` attaches to that target rather than creating a duplicate session. Set `ZJ_SESSION` to override the project-derived name.

```text
zj                             attach current or single space (create project space if none)
zj attach [NAME]               attach or switch to a named space
zj new NAME                    create and attach a named space
zj ls [--names]                print spaces table (or names only)
zj kill [NAME]                 kill and delete a space
zj agent claude [ARGS]         open claude in a new tab (OSC 7501 wrapped)
zj run -- cargo build          open a command in a new pane
zj inbox                       toggle the stock Inbox (works outside if a client is attached)
zj next                        jump to the next attention target
zj prompt "continue"           send text and Enter into the focused pane
zj wait needs_you 600          block until the focused pane needs attention (optional pane ID)
zj read [--full]               dump the focused pane viewport or full scrollback
zj panes                       output pane and attention state as JSON
zj resume-command AGENT [DIR]  print the resume invocation for the latest conversation
zj doctor                      show config, spaces, target, and client state
```

`zj status` must run inside a Zellij pane because it writes OSC 7501 to that pane's PTY. Running it in a plain terminal reports an error instead of failing silently.

`zj agent` creates a generated layout so the tab preserves its name, working directory, and output after the agent exits. Inside Zellij, it opens a new tab. Outside Zellij, it starts a new session with that layout or adds the tab to an existing session (including a detached session) and attaches. Adding a tab to a detached session requires a viewport, so `zj` briefly attaches a headless Zellij client for that step.

## Verification

The table below records checks verified on this machine.

| Check | Result |
| --- | --- |
| Binary and config | `zellij --version` reported `zellij 0.46.0`, and `zellij setup --check` reported the config file as well defined. |
| OSC 7501 working | Emitting `printf '\e]7501;state=working:progress=40:app=probe:msg=...'` caused `list-panes --json` to report `attention.status: Working`, source `ProgramStatus`, progress `40`, and the decoded app and message. |
| OSC 7501 blocked | Emitting `state=blocked:kind=permission` produced `NeedsYou` with text `needs approval: Allow edit to config.kdl?`. |
| Live agents | Pi 1.5.2 emitted `idle` (`app: pi`), then `Working`, then `Done` after a prompt. OpenCode 2.3.0 emitted `Working` and `Idle` natively, and `harnesses/opencode/plugins/zellij-status.ts` added `Done` after `session.execution.succeeded`. |
| Multi-pane OpenCode | Three panes sharing the background service all turned `Working` when one received a prompt, while three `--standalone` panes kept the unprompted two `Idle`. Two standalone OpenCode panes in one session tracked `Working` and `Done` independently in both Spaces and the stock Inbox. |
| Fresh versus restore | After saving and killing a space named `zjfresh`, bare `zj` in that directory created `zjfresh-2` and left `zjfresh` exited. Running `zj attach zjfresh` restored the exited space. |
| Restored visibility | Running `zj __prepare-restore` on a saved OpenCode pane wrapped `opencode --standalone` in `zj __resume-wrap` and stripped `start_suspended`. After restore, `list-panes` showed `Idle` with `app: opencode` and the stock Inbox listed the pane under `Idle` immediately. |
| Status priming | Direct `codex resume --last`, a restored `zj __resume-wrap b3BlbmNvZGU=` pane, and a `zj __shim opencode opencode` pane all settled at `Idle` with `app` populated and appeared in the stock Inbox `Idle` section. |
| Turn status commands | `session-hook opencode prompt` emitted `working`, `stop` emitted `done`, and `start` emitted the `working` plus `idle` prime sequence. |
| Remembered `Done` | A command pane that called `zj status done` and remained alive stayed marked as `done` in Spaces and `Done` in the stock Inbox instead of reverting to `idle`. |
| Peer identity | With the navigator open in a second session, an idle Claude pane in a peer space rendered as `claude-code` instead of `terminal` once `zj __space-agents` or `zj __space-snapshot` returned the live app map. |
| Plugin tests and snapshots | `cargo test` in the plugin repository passed 81 unit tests covering all 22 vendored Herdr manifests, Gemini and Cursor blockers, live Claude captures, OSC precedence, native `Done` stickiness, working-to-idle `Done` synthesis, search ranking, dialogs, session deduplication, hook precedence, and the `H` overlay. `zj __space-snapshot` returned app, title, and viewport text for six agent panes across two running sessions in one call. |
| Real PTY keybindings | `Ctrl-b c`, `v`, `x`, `b`, `n`, `o`, and `q` performed tab creation, splits, pane close, Inbox toggle, tab cycling, attention jump, and detach. Bare `v` and `Alt-n` did not trigger Zellij actions while locked. `Ctrl-Space c` opened a tab through the fallback leader. |
| Wait and agent helpers | `zj wait needs_you` returned `needs_you` after a blocked OSC 7501 sequence. `zj agent sleep 5` created a `sleep` tab in `Working` state inside a session, when starting a new session from outside, and when adding a tab to a detached session. |
| Bare `zellij` targeting | Against a bare `zellij` session (`adventurous-cactus`), `Ctrl-b b` and `Ctrl-b i` toggled the Inbox, `zj inbox` toggled it from outside while attached, `zj doctor` reported that session as the target, and `zj` inside stayed on that session. |
| Spaces popup in a PTY | `Ctrl-b w` opened a centered popup (`103x32` on a `120x40` terminal) with the current space first, reflected `blocked` on a detached space via `GlobalAttentionUpdate`, jumped to the selected agent pane on `Enter`, and closed after switching. |
| Search and lifecycle | `/` filtered spaces, `?` toggled help, `n` created a real session and switched to it, `r` renamed the current space, `x` plus `Enter` killed a non-current space and removed its saved state, and closing the current space was refused with a footer message. |
| Shell space switching | `zj space switch alpha` and `zj space new three /tmp` switched the attached client in about a second without leaving a plugin pane behind, and started the new shell in `/tmp`. |
| Space handoff | Closing the last pane of a second space reattached the `zj` client to the original space, while `Ctrl-b q` exited the client and kept both spaces running. |
| Key isolation over splits | Typing into the new-space form and closing left two side-by-side shell panes untouched (`86x24` popup on a `100x30` terminal), with immediate input return after closing or clicking outside. |
| Per-agent hooks | Unit tests for the hook bridge cover provider normalization, merge semantics, store and spool handling, Unix socket and TCP listeners, and the hook client (`go test ./internal/harness/zj... -count=1` in `~/dev/model-router`). |
| Resurrection and exact IDs | Preparing a saved session with `zj __prepare-restore` and attaching without `--force-run-commands` launched the pane immediately with `--resume <id>`. Two panes in the same working directory reporting `conversation-A` and `conversation-B` restored with their respective `--resume` IDs both from the shell and from the Spaces navigator. |
| Resume discovery | `zj resume-command` returned the expected ID and switch for Claude, Codex, OpenCode, Pi, Qwen, Copilot, Kimi, Muse, Vibe, Cline, Grok, Kilo, Antigravity, and fx. When a store was unavailable, it fell back to the continue-latest switch. |

## Notes and limits

- OSC 7501 applies per pane. `done` and `error` are attention states. Once you focus the pane, Zellij resolves the attention state to `idle` while retaining the raw record in `program_status` (visible via `zj panes`). To observe the marker, let the agent finish in a background tab. After a command pane exits, Zellij can also mark long runs as done using `done_threshold_secs`.
- The stock Inbox lists only panes that have reported at least one non-idle state. Agent launchers therefore emit a short `working` record followed by `idle` so idle agents stay visible in the Inbox without showing a false working state.
- The installed versions of Gemini, Qwen, and Cursor lack native OSC 7501 support. Their hooks and notifications still drive the Inbox, `zj agent` wraps their process lifecycle, and the Spaces navigator infers working, blocked, and idle states from viewport text. `general.enableNotifications` is enabled in `harnesses/gemini/settings.json`.
- Viewport scanning in the Spaces navigator runs through `zj __space-snapshot` (`zellij --session NAME action dump-screen`) rather than `get_pane_scrollback`, so it requires no extra plugin permission. It affects only the Spaces navigator; the tab bar and stock Inbox continue to reflect native attention state.
- Zellij caches plugin permissions by absolute WASM path. Moving or renaming `~/.config/zellij/plugins/zellij-spaces.wasm` causes Zellij to prompt for permissions again unless you rerun `scripts/setup-zellij.sh`.
- The navigator overlays `GlobalAttentionUpdate` onto `SessionUpdate` because Zellij can send stale manifests for detached peer sessions while global attention stays current. Duplicate session snapshots that can appear during a rename are collapsed to the richest entry.
- The navigator displays only the five most recent exited spaces. Use the stock session manager (`Ctrl-b b` or `Ctrl-b i`) to browse the full list of saved sessions.
- Renaming works only on the current space, and closing the current space is blocked. Both are limits of Zellij's plugin API.
- `zj` sessions are ordinary Zellij sessions, so the navigator lists sessions started by both `zj` and bare `zellij`.
- Handoff when closing a space's last pane runs inside the `zj` wrapper and requires Linux `/proc` and `ss`. Bare `zellij` clients and other operating systems keep Zellij's default exit behavior.
- Zellij session names are global, whereas `zj` derives default names from the project directory basename. Set `ZJ_SESSION` when working on two projects that share a directory name.
- `desktop_notifications` (`needs_you`, `failed`, `done`) and `notification_details` are enabled in `zellij/config.kdl`, but require the optional Zellij desktop terminal (`zellij terminal`).
