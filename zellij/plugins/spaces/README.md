# zellij-spaces

The Herdr-style Spaces navigator used by this dotfiles checkout.

It is a standalone Zellij 0.46 plugin, not a patched session manager and not a
sidebar. Each Zellij session is a space; each terminal pane under it is an agent
or terminal. Pressing `Enter` opens the selected space or jumps directly to the
selected pane's session, tab and pane.

## Architecture

| File | Responsibility |
| --- | --- |
| `src/model.rs` | Zellij `SessionInfo` / `GlobalAttention` to spaces, agents, status roll-ups, search ranking; hook summaries and details |
| `src/app.rs` | pure state machine: key/mouse events to `Effect`s, navigation, filters, dialogs, hook and screen-fallback precedence |
| `src/screen/` | herdr-derived manifest evaluator and bundled agent viewport manifests |
| `src/canvas.rs` | cell buffer with truecolor ANSI output and control-character sanitization |
| `src/ui.rs` | layout, responsive rows, dialogs, help, keyboard hints |
| `src/layout.rs` | shared list geometry and scroll window |
| `src/theme.rs` | Catppuccin Mocha palette |
| `src/main.rs` | Zellij host boundary: opener/popup lifecycle, events in, effects out |

The plugin subscribes to `SessionUpdate` (tabs, panes, viewport) and
`GlobalAttentionUpdate` (cross-session agent state). Session manifests can be
stale for detached peer sessions, so global attention is applied over them.
Duplicate session snapshots are collapsed by name.

A third channel, `zj __space-hooks`, carries normalized per-agent hook records
(tool, prompt, reply, model, session id, blocked kind, subagents). It is polled
about every 1.5 seconds; full detail is fetched only when the `H` overlay opens.
Hooks win over OSC and screen evidence while fresh; see `docs/zellij.md` for the
freshness rules. The listener and spool live in
`${XDG_RUNTIME_DIR:-/tmp}/zj`, and the shared helper module is
`zellij/zj_hook_core.py`.

It requests three permissions:

- `ReadApplicationState` for session and attention data;
- `ChangeApplicationState` for opening the popup and for switch, create,
  rename and kill actions;
- `RunCommands` for the `zj __space-snapshot` helper.

## Screen-reader fallback

OSC 7501 is the primary source of truth. Agents that do not update it (Gemini,
Cursor, older Qwen, Cline, Antigravity, Kilo) are detected from their viewport
by a trimmed port of [herdr](https://github.com/herdrdev/herdr)'s manifest engine
(`src/screen/`, Apache-2.0, see `src/screen/NOTICE`). While the navigator is
open it asks `zj __space-snapshot` every ~5 seconds for app, title and viewport
text for every agent pane in every running space. The helper reads panes with
`zellij --session NAME action dump-screen`, so peer/detached sessions work
without a new plugin permission or a blocking `get_pane_scrollback` call.

A strong `visible_blocker` (permission/question prompt) overrides OSC; otherwise
a live non-idle OSC state wins for authority agents (Claude, Codex, OpenCode, Pi,
Crush, Grok, omp, Copilot, Kimi, Muse, MCode, fx, Vibe, DeepSeek/dsh-tui).
Screen-only agents get `working`/`needs you`/`idle`, and a screen-detected
working→idle transition shows `Done` in the navigator until the pane is focused
or ten minutes elapse (while that navigator instance stays open). This `Done`
state is navigator-only; Zellij's tab bar and Inbox still show native attention.

`screen_fallback false` in the plugin configuration restores the previous
app-label-only path (`zj __space-agents`). Put it in the `LaunchOrFocusPlugin`
block (or the plugin alias) alongside the existing `floating`/`open_new` keys.

The manifests under `src/screen/manifests/herdr/` are copied from herdr commit
`ee5bfb50`; regenerate the embedded `src/screen/manifests.json` with:

```sh
python3 ../../../scripts/sync-screen-manifests.py
```

A tiny opener instance moves itself to the top-left corner, opens the
navigator with Zellij's popup API (`capture_all_keys`, outside-click close)
and exits. The navigator is therefore a real popup: keys go to it, not to the
terminal underneath, and it cannot lose focus to a split pane behind it.

## Build

```sh
../../../scripts/build-zellij-spaces.sh
```

or manually:

```sh
cargo test --no-default-features
cargo build --release --target wasm32-wasip1
install -m 644 target/wasm32-wasip1/release/zellij-spaces.wasm \
  ../../plugins/zellij-spaces.wasm
```

The tracked artifact is `../zellij-spaces.wasm`; `scripts/setup-zellij.sh`
installs it and seeds `~/.cache/zellij/permissions.kdl` for its absolute path.
The embedded screen detector adds `regex` and the manifests, so the release WASM
is roughly 2.5 MB.

## Keys

`↑↓` / `jk` move, `←→` / `hl` change space, `Enter` opens, `/` searches,
`n` creates, `r` renames the current space, `x` closes, `Space` collapses,
`1..9` jumps to a space, `b/w/d/i/a` filter attention, `H` opens the selected
agent's hook detail, `Ctrl-r` refreshes, `?` shows help and `Esc` clears/closes.
Mouse clicks open a row; the wheel moves the selection. In the hook overlay,
`↑↓`/`PgUp`/`PgDn` scroll and `H`/`Esc` close.
