#!/usr/bin/env python3
"""Resurrection and agent-status support for the Zellij Spaces navigator.

Jobs:

* ``hook`` is wired into Zellij's ``post_command_discovery_hook``.  When
  Zellij serializes a pane that is running an interactive coding agent, the
  command is wrapped so that the restored pane reopens the agent instead of a
  bare shell.
* ``wrap`` is the command the restored pane actually runs.  It finds the most
  recent conversation for the agent in the pane's cwd and adds that agent's
  resume switch (or its "continue latest" switch when no id can be found).  It
  also primes the pane as idle before starting the agent.
* ``prepare-restore`` removes ``start_suspended`` from a saved session layout
  so Zellij runs the saved commands immediately instead of showing the
  ``Waiting to run`` banner and waiting for Enter.
* ``session-hook`` is the harness-facing entry point.  It records the exact
  conversation id reported by the harness on the pane's Zellij resurrect
  command, and emits the status named by its event argument (idle on start,
  working on a prompt, done at the end of a turn).
* ``space-agents`` reports the live agent app for each pane across all running
  sessions.  Zellij's peer metadata cache omits program status, so the Spaces
  navigator uses this to avoid labelling idle agents as plain terminals.

The helper only reads local session stores.  When a harness reports its exact id
through ``session-hook``, the id is stored on the Zellij pane itself.  Otherwise
a conversation is scoped to the pane's working directory, which is how the
agents themselves scope their "latest conversation" behaviour.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import select
import shlex
import shutil
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

try:
    import zj_hook_core as hook_core
except Exception:  # the helper still works without the optional hook module
    hook_core = None


HOME = Path.home()
XDG_CACHE = Path(os.environ.get("XDG_CACHE_HOME") or HOME / ".cache")
XDG_DATA = Path(os.environ.get("XDG_DATA_HOME") or HOME / ".local/share")
CLAUDE_HOME = Path(os.environ.get("CLAUDE_CONFIG_DIR") or HOME / ".claude")
CODEX_HOME = Path(os.environ.get("CODEX_HOME") or HOME / ".codex")
GROK_HOME = Path(os.environ.get("GROK_HOME") or HOME / ".grok")
DSH_HOME = Path(os.environ.get("DSH_HOME") or HOME / ".dsh")
GEMINI_HOME = Path(os.environ.get("GEMINI_HOME") or HOME / ".gemini")
ZELLIJ_CACHE = Path(
    os.environ.get("ZJ_ZELLIJ_CACHE_DIR")
    or XDG_CACHE / "zellij" / "contract_version_1" / "session_info"
)
ALIASES = {
    "agy": "antigravity",
    "dsh": "deepseek",
    "kimi-code": "kimi",
    "cursor": "cursor-agent",
}

# App names that agents themselves put in their OSC 7501 records. Keeping the
# same spelling means our launch-time records and the agent's own records do
# not alternate between two labels for one pane.
APP_NAMES = {
    "claude": "claude-code",
    "kimi-cli": "kimi",
    "dsh": "deepseek",
    "dsh-tui": "deepseek",
    "cursor": "cursor-agent",
}

AGENTS = {
    "claude",
    "codex",
    "opencode",
    "pi",
    "qwen",
    "copilot",
    "kimi",
    "kimi-cli",
    "mcode",
    "muse",
    "vibe",
    "gemini",
    "cline",
    "cursor-agent",
    "grok",
    "crush",
    "omp",
    "kilo",
    "antigravity",
    "deepseek",
    "dsh-tui",
    "fx",
    "fx-router",
}

SHELLS = {"sh", "bash", "zsh", "dash", "fish", "nu", "ksh", "tcsh", "csh"}


def basename(value: str) -> str:
    value = value.rstrip("/")
    return os.path.basename(value)


def current_directory() -> str:
    try:
        return os.getcwd()
    except OSError:
        return os.environ.get("PWD") or str(HOME)


def read_hook_payload(timeout: float = 0.5) -> dict:
    """Read one JSON object from stdin without blocking forever.

    Claude and Grok can keep the hook's stdin open after writing the payload,
    so ``sys.stdin.read()`` would hang until the hook timeout.  Read whatever
    arrives, parse as soon as the object is complete, and give up at the
    deadline.  A tty means there is no payload at all.
    """
    stream = sys.stdin
    if stream is None:
        return {}
    try:
        if stream.isatty():
            return {}
    except (OSError, ValueError):
        return {}
    data = b""
    deadline = time.monotonic() + max(0.0, timeout)
    while time.monotonic() < deadline:
        remaining = max(0.0, deadline - time.monotonic())
        try:
            ready, _, _ = select.select([stream], [], [], min(0.05, remaining))
        except (OSError, ValueError):
            break
        if not ready:
            if data.strip():
                try:
                    parsed = json.loads(data.decode("utf-8", "replace"))
                except ValueError:
                    continue
                return parsed if isinstance(parsed, dict) else {}
            continue
        try:
            chunk = os.read(stream.fileno(), 65536)
        except OSError:
            break
        if not chunk:
            break
        data += chunk
        try:
            parsed = json.loads(data.decode("utf-8", "replace"))
        except ValueError:
            continue
        return parsed if isinstance(parsed, dict) else {}
    if not data.strip():
        return {}
    try:
        parsed = json.loads(data.decode("utf-8", "replace"))
    except ValueError:
        return {}
    return parsed if isinstance(parsed, dict) else {}


def extract_session_id(payload: dict) -> str | None:
    keys = (
        "session_id",
        "sessionId",
        "sessionID",
        "thread_id",
        "threadId",
        "conversation_id",
        "conversationId",
    )
    for key in keys:
        value = payload.get(key)
        if isinstance(value, str) and value:
            return value
    for container in ("session", "thread", "conversation", "data", "_meta"):
        nested = payload.get(container)
        if isinstance(nested, dict):
            for key in keys:
                value = nested.get(key)
                if isinstance(value, str) and value:
                    return value
    value = payload.get("id")
    return value if isinstance(value, str) and value else None


def zellij_binary() -> str:
    return os.environ.get("ZELLIJ_BIN") or "zellij"


def pane_command_candidates() -> list[list[str]]:
    pane_id = os.environ.get("ZELLIJ_PANE_ID")
    if not pane_id:
        return []
    try:
        number = int(re.sub(r"\D", "", pane_id))
    except ValueError:
        return []
    try:
        result = subprocess.run(
            [zellij_binary(), "action", "list-panes", "--json", "--all"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        panes = json.loads(result.stdout)
    except Exception:
        return []
    if not isinstance(panes, list):
        return []
    matches = [
        pane
        for pane in panes
        if isinstance(pane, dict)
        and pane.get("id") == number
        and not pane.get("is_plugin")
    ]
    if not matches:
        return []
    pane = next((pane for pane in matches if pane.get("is_focused")), matches[0])
    candidates: list[str] = []
    for key in ("pane_command", "terminal_command"):
        value = pane.get(key)
        if isinstance(value, str) and value:
            candidates.append(value)
    command_state = pane.get("command_state") or {}
    for section in ("running", "last"):
        command = (command_state.get(section) or {}).get("command") or {}
        value = command.get("text")
        if isinstance(value, str) and value:
            candidates.append(value)
    commands: list[list[str]] = []
    for candidate in candidates:
        try:
            argv = shlex.split(candidate)
        except ValueError:
            continue
        if argv and argv not in commands:
            commands.append(argv)
    return commands


def process_ancestor_commands() -> list[list[str]]:
    commands: list[list[str]] = []
    pid = os.getpid()
    seen: set[int] = set()
    while pid > 1 and pid not in seen:
        seen.add(pid)
        try:
            stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8", errors="replace")
            fields = stat[stat.rfind(")") + 2:].split()
            ppid = int(fields[1])
        except Exception:
            break
        try:
            raw = Path(f"/proc/{ppid}/cmdline").read_bytes()
            argv = [
                part.decode("utf-8", errors="replace")
                for part in raw.split(b"\0")
                if part
            ]
            if argv:
                commands.append(argv)
        except Exception:
            pass
        pid = ppid
    return commands


def is_wrapper_command(argv: list[str]) -> bool:
    return any(basename(token) in {"zj", "harness"} for token in argv[:4])


def choose_base_command(agent: str, candidates: list[list[str]]) -> list[str]:
    usable = [
        argv
        for argv in candidates
        if "__resume-wrap" not in argv
        and "session-hook" not in argv
        and detect_agent(argv) == agent
        and not is_noninteractive(agent, argv[1:])
    ]
    wrappers = [argv for argv in usable if is_wrapper_command(argv)]
    if wrappers:
        base = wrappers[0]
    elif usable:
        base = usable[0]
    else:
        base = [agent]
    return [base[0]] + strip_resume_args(agent, base[1:])


def display_app(agent_name: str) -> str:
    """Canonical app label used in OSC 7501 records and the navigator."""
    name = basename(agent_name).strip().lower()
    normalized = normalize(name)
    if normalized:
        name = normalized
    return APP_NAMES.get(name, name)


def report_tty_paths() -> list[str]:
    """Find a tty that belongs to the pane, not a daemon's detached child.

    Agents and plugins may run in a server process whose own fds are pipes.
    Walking up to the TUI process still finds the pane's pts, so status
    records land where Zellij is watching them.
    """
    paths: list[str] = []
    configured = os.environ.get("ZJ_TTY")
    if configured:
        paths.append(configured)
    paths.append("/dev/tty")
    pid = os.getpid()
    seen: set[int] = set()
    while pid > 1 and pid not in seen:
        seen.add(pid)
        try:
            target = os.readlink(f"/proc/{pid}/fd/0")
        except OSError:
            target = ""
        if target.startswith(("/dev/pts/", "/dev/tty")):
            paths.append(f"/proc/{pid}/fd/0")
        try:
            stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8", errors="replace")
            fields = stat[stat.rfind(")") + 2:].split()
            pid = int(fields[1])
        except Exception:
            break
    return paths


def emit_program_status(
    state: str, app: str, msg: str | None = None, kind: str | None = None
) -> None:
    """Write one OSC 7501 record to the pane's terminal."""
    body = f"state={state}:app={app}"
    if kind:
        body += ":kind=" + kind
    if msg:
        body += ":msg=" + base64.b64encode(msg.encode("utf-8")).decode("ascii")
    sequence = f"\033]7501;{body}\033\\"
    for path in report_tty_paths():
        try:
            with open(path, "w", encoding="utf-8", errors="replace") as tty:
                tty.write(sequence)
                tty.flush()
            return
        except OSError:
            continue
    # Hook stdout is consumed by the agent; never leak protocol bytes there.


def emit_pane_title(title: str) -> None:
    """Give a restored pane an agent label before its first status record.

    The stock Inbox labels an attention item with the pane title captured when
    the record arrives. A just-restored pane is still titled with its
    `zj __resume-wrap ...` command, so set the app name first.
    """
    if not title:
        return
    sequence = f"\033]2;{title}\033\\"
    for path in report_tty_paths():
        try:
            with open(path, "w", encoding="utf-8", errors="replace") as tty:
                tty.write(sequence)
                tty.flush()
            return
        except OSError:
            continue


def prime_agent_status(agent: str) -> None:
    """Register the agent as idle while keeping it visible in the Inbox.

    Zellij only remembers panes that reported a non-idle record at least once.
    A short working record followed by idle gives the pane a remembered idle
    state, so a freshly opened or resumed chat appears as idle in the Inbox
    instead of disappearing entirely.
    """
    app = display_app(agent)
    emit_program_status("working", app)
    emit_program_status("idle", app)


def emit_agent_event(agent: str, event: str | None) -> None:
    if not event:
        return
    app = display_app(agent)
    if event in {"start", "resume", "session-start"}:
        prime_agent_status(agent)
    elif event in {"prompt", "working", "busy"}:
        emit_program_status("working", app)
    elif event in {"stop", "done", "turn-end"}:
        emit_program_status("done", app)
    elif event in {"error", "failed"}:
        emit_program_status("error", app)
    elif event in {"blocked", "needs-you", "permission"}:
        emit_program_status("blocked", app)


def command_agent(command: str) -> str | None:
    """Detect an agent from a pane's stringified command line."""
    try:
        argv = shlex.split(command)
    except ValueError:
        return None
    if len(argv) >= 3 and basename(argv[0]) == "zj" and argv[1] == "__resume-wrap":
        try:
            inner = base64.b64decode(argv[2].encode("ascii")).decode("utf-8")
        except Exception:
            return None
        try:
            argv = shlex.split(inner)
        except ValueError:
            return None
    return detect_agent(argv)


def running_session_names_result() -> tuple[list[str], bool]:
    """Return (running session names, whether list-sessions succeeded)."""
    try:
        result = subprocess.run(
            [zellij_binary(), "list-sessions", "-n"],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except Exception:
        return [], False
    if result.returncode != 0:
        return [], False
    names: list[str] = []
    for line in result.stdout.splitlines():
        line = line.strip()
        if not line or "EXITED" in line:
            continue
        fields = line.split()
        if fields:
            names.append(fields[0])
    return names, True


def running_session_names() -> list[str]:
    return running_session_names_result()[0]


def session_panes_result(session: str) -> tuple[list[dict], bool]:
    """Return (panes, whether list-panes succeeded)."""
    try:
        result = subprocess.run(
            [
                zellij_binary(),
                "--session",
                session,
                "action",
                "list-panes",
                "--json",
                "--all",
            ],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except Exception:
        return [], False
    if result.returncode != 0:
        return [], False
    try:
        panes = json.loads(result.stdout)
    except Exception:
        return [], False
    if not isinstance(panes, list):
        return [], False
    return panes, True


def session_panes(session: str) -> list[dict]:
    return session_panes_result(session)[0]


def pane_app(pane: dict) -> str | None:
    attention = pane.get("attention") or {}
    detail = attention.get("detail") or {}
    app = detail.get("app")
    if isinstance(app, str) and app.strip():
        return display_app(app)
    for record in reversed(pane.get("program_status") or []):
        app = record.get("app")
        if isinstance(app, str) and app.strip():
            return display_app(app)
    for key in ("terminal_command", "pane_command"):
        command = pane.get(key)
        if isinstance(command, str) and command.strip():
            agent = command_agent(command)
            if agent:
                return display_app(agent)
    return None


def space_agents() -> int:
    """Report the agent app for each live pane, across all running spaces.

    Peer session manifests in Zellij's metadata cache omit program status and
    terminal commands, so idle agents in other spaces would otherwise be
    indistinguishable from plain terminals. Live list-panes output still has
    that information.
    """
    payload: dict[str, dict[str, str]] = {}
    for session in running_session_names():
        apps: dict[str, str] = {}
        for pane in session_panes(session):
            if pane.get("is_plugin"):
                continue
            app = pane_app(pane)
            if not app:
                continue
            if app in {"terminal", "shell", "zj", "sh", "bash", "zsh", "fish"}:
                continue
            pane_id = pane.get("id")
            if isinstance(pane_id, int):
                apps[str(pane_id)] = app
        if apps:
            payload[session] = apps
    print(json.dumps(payload, sort_keys=True))
    return 0


SNAPSHOT_MAX_PANES = 64
SNAPSHOT_MAX_LINES = 200
SNAPSHOT_MAX_BYTES = 32768
SNAPSHOT_DEADLINE_SECS = 8.0
SNAPSHOT_SHELL_LABELS = {"terminal", "shell", "zj", "sh", "bash", "zsh", "fish"}


def trim_screen_text(text: str) -> str:
    """Trim padding and keep the bottom of an over-long viewport."""
    lines = [line.rstrip() for line in text.splitlines()]
    while lines and not lines[-1].strip():
        lines.pop()
    if len(lines) > SNAPSHOT_MAX_LINES:
        lines = lines[-SNAPSHOT_MAX_LINES:]
    rendered = "\n".join(lines)
    encoded = rendered.encode("utf-8")
    if len(encoded) <= SNAPSHOT_MAX_BYTES:
        return rendered
    # Manifests read from the bottom, so keep the tail on a UTF-8 boundary.
    tail = encoded[-SNAPSHOT_MAX_BYTES:]
    return tail.decode("utf-8", errors="ignore")


def dump_pane_screen(session: str, pane_id: int, timeout: float) -> tuple[bool, str]:
    try:
        result = subprocess.run(
            [
                zellij_binary(),
                "--session",
                session,
                "action",
                "dump-screen",
                "--pane-id",
                f"terminal_{pane_id}",
            ],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=max(0.2, timeout),
        )
    except Exception:
        return False, ""
    if result.returncode != 0:
        return False, ""
    return True, trim_screen_text(result.stdout)


def space_snapshot() -> int:
    """Report viewport text for agent panes across every running space.

    The Spaces navigator cannot read peer sessions through the plugin API, and
    a blocking current-session scrollback call would need ReadPaneContents.
    This helper runs `zellij --session NAME action dump-screen` instead, so one
    snapshot covers every space and adds no plugin permission.

    Failures are explicit: ``ok`` says list-sessions succeeded, each session
    carries ``ok`` for list-panes, and each pane carries ``dump_ok``. The
    navigator retains the previous reading when a dump fails and only prunes
    panes that are absent from a successful session listing.
    """
    sessions, sessions_ok = running_session_names_result()
    payload: dict[str, dict] = {}
    remaining = SNAPSHOT_MAX_PANES
    deadline = time.monotonic() + SNAPSHOT_DEADLINE_SECS

    for session in sessions:
        panes, panes_ok = session_panes_result(session)
        entry: dict = {"ok": panes_ok, "panes": {}}
        payload[session] = entry
        if not panes_ok:
            continue
        for pane in panes:
            if pane.get("is_plugin") or pane.get("exited") or pane.get("is_held"):
                continue
            app = pane_app(pane)
            if not app or app in SNAPSHOT_SHELL_LABELS:
                continue
            pane_id = pane.get("id")
            if not isinstance(pane_id, int):
                continue
            screen = ""
            dump_ok = False
            if remaining > 0 and time.monotonic() < deadline:
                remaining -= 1
                dump_ok, screen = dump_pane_screen(
                    session, pane_id, deadline - time.monotonic()
                )
            entry["panes"][str(pane_id)] = {
                "app": app,
                "title": pane.get("title") or "",
                "screen": screen,
                "dump_ok": dump_ok,
            }

    print(
        json.dumps(
            {"ok": sessions_ok, "sessions": payload},
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
    )
    return 0


def record_resume_target(agent: str, payload: dict) -> None:
    """Store the harness's exact conversation id on this pane.

    Runs only for session-bearing events.  ``pane_command_candidates`` starts a
    ``zellij action list-panes`` subprocess (5 s timeout), so tool events must
    never reach this path.
    """
    session_id = extract_session_id(payload)
    if not session_id:
        return
    pane_id = os.environ.get("ZELLIJ_PANE_ID")
    if not pane_id:
        return
    try:
        candidates = process_ancestor_commands() + pane_command_candidates()
        base = choose_base_command(agent, candidates)
        encoded = base64.b64encode(shlex.join(base).encode("utf-8")).decode("ascii")
        subprocess.run(
            [
                zellij_binary(),
                "action",
                "set-resurrect-command",
                "--pane-id",
                pane_id,
                "--",
                "zj",
                "__resume-wrap",
                encoded,
                session_id,
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=5,
        )
    except Exception:
        return


def agent_is_noninteractive(agent: str) -> bool:
    """True when the nearest agent ancestor is a non-interactive run.

    Use the nearest agent command, not every ancestor: a ``claude -p`` nested
    inside an interactive Claude session must still be treated as headless.
    """
    candidates = process_ancestor_commands()
    agent_commands = [argv for argv in candidates if detect_agent(argv) == agent]
    if not agent_commands:
        return False
    return is_noninteractive(agent, agent_commands[0][1:])


_OSC_STATES = {
    "working": "working",
    "blocked": "blocked",
    "done": "done",
    "failed": "error",
    "idle": "idle",
}


def emit_hook_status(event: dict) -> None:
    """Project a normalized hook event onto OSC 7501."""
    if event.get("child"):
        return  # child activity must not change the pane's main state
    app = event.get("app") or "zj"
    raw = event.get("event")
    state = event.get("state")
    kind = event.get("kind") if state == "blocked" else None
    text = event.get("text") or None
    if raw == "session_start":
        # Prime the stock Inbox exactly like the legacy session-hook path.
        emit_program_status("working", app)
        emit_program_status("idle", app)
        return
    if raw == "session_end":
        emit_program_status("idle", app, text)
        return
    osc = _OSC_STATES.get(state)
    if osc:
        emit_program_status(osc, app, text, kind)


def _hook_target() -> tuple[str, int] | None:
    session = os.environ.get("ZELLIJ_SESSION_NAME") or ""
    pane_raw = os.environ.get("ZELLIJ_PANE_ID") or ""
    match = re.search(r"(\d+)", pane_raw)
    if not session or not match:
        return None
    try:
        return session, int(match.group(1))
    except (TypeError, ValueError):
        return None


def hook_client(
    agent_name: str,
    event: str | None = None,
    payload: dict | None = None,
    transport: bool = True,
) -> int:
    """Harness-facing entry point: normalize, emit OSC, store/spool, exit 0."""
    agent = normalize(agent_name) or agent_name
    if payload is None:
        payload = read_hook_payload()
    if agent_is_noninteractive(agent):
        return 0
    target = _hook_target()
    if target is None:
        # Outside Zellij there is no pane to attribute the event to.  Do not
        # emit OSC either: an Orca-hosted agent answers OSC 7501 itself.
        return 0
    if hook_core is None or hook_core.disabled():
        emit_agent_event(agent, event)
        return 0
    normalized = hook_core.normalize_event(agent, None, event, payload, os.environ)
    if normalized is None:
        return 0
    emit_hook_status(normalized)
    if normalized.get("event") in {"session_start", "prompt", "stop", "session_end"} and not normalized.get(
        "child"
    ):
        record_resume_target(agent, payload)
    if transport:
        session, pane = target
        envelope = {
            "session": session,
            "pane": pane,
            "agent": agent,
            "event": event,
            "payload": payload,
            "normalized": normalized,
        }
        if not hook_core.publish(envelope):
            hook_core.append_spool({"session": session, "pane": pane, **normalized})
            hook_core.ensure_server(os.path.abspath(__file__))
    return 0


def session_hook(agent_name: str, event: str | None = None) -> int:
    """Backward-compatible entry point used by older harness configs."""
    return hook_client(agent_name, event)



def normalize(name: str) -> str | None:
    name = basename(name).lower()
    name = re.sub(r"\.(js|mjs|cjs|py|exe|cmd|bat)$", "", name)
    name = ALIASES.get(name, name)
    if name in AGENTS:
        return name
    return None


def infer_agent(command: str) -> str | None:
    """Map an executable path or command name to a known agent."""
    name = basename(command)
    normalized = normalize(name)
    if normalized:
        return normalized

    lowered = command.lower()
    hints = [
        ("claude", "claude"),
        ("codex", "codex"),
        ("opencode", "opencode"),
        ("kilo", "kilo"),
        ("gemini", "gemini"),
        ("qwen", "qwen"),
        ("copilot", "copilot"),
        ("kimi-cli", "kimi-cli"),
        ("kimi-code", "kimi"),
        ("kimi", "kimi"),
        ("minimax", "mcode"),
        ("mcode", "mcode"),
        ("muse", "muse"),
        ("mistral-vibe", "vibe"),
        ("vibe", "vibe"),
        ("cline", "cline"),
        ("cursor-agent", "cursor-agent"),
        ("grok", "grok"),
        ("crush", "crush"),
        ("antigravity", "antigravity"),
        ("deepseek", "deepseek"),
        ("dsh-tui", "dsh-tui"),
        ("dsh", "deepseek"),
        ("fx-router", "fx"),
        ("/fx", "fx"),
        ("omp", "omp"),
        ("/pi", "pi"),
    ]
    for needle, agent in hints:
        if needle in lowered:
            return agent
    return None


def detect_shell_script(argv: list[str]) -> str | None:
    """Recognize an agent launched through a shell script wrapper."""
    index = 1
    while index < len(argv):
        token = argv[index]
        if token in {"-c", "-lc", "-ic", "-cl"}:
            return None
        if token.startswith("-"):
            index += 1
            continue
        break
    if index >= len(argv):
        return None
    script = argv[index]
    name = basename(script)
    if name == "zj" and index + 1 < len(argv) and argv[index + 1] == "__shim":
        hold = index + 2 < len(argv) and argv[index + 2] == "--hold"
        bin_index = index + 4 if hold else index + 3
        if bin_index < len(argv):
            return infer_agent(argv[bin_index])
        return infer_agent(argv[index + 2] if index + 2 < len(argv) else script)
    if name == "harness" and index + 1 < len(argv):
        if argv[index + 1] == "profile" and index + 2 < len(argv):
            return profile_agent(argv[index + 2])
        return normalize(argv[index + 1])
    return infer_agent(script)


def detect_agent(argv: list[str]) -> str | None:
    """Find the agent a command line belongs to, or None for normal commands."""
    if not argv:
        return None

    exe = basename(argv[0])
    if exe in SHELLS:
        return detect_shell_script(argv)

    if exe == "harness":
        if len(argv) >= 2:
            if argv[1] == "profile":
                if len(argv) >= 3 and not argv[2].startswith("-"):
                    return profile_agent(argv[2])
                return None
            return normalize(argv[1])
        return None

    if exe == "zj":
        if len(argv) >= 3 and argv[1] == "__shim":
            hold = argv[2] == "--hold"
            offset = 3 if hold else 2
            if len(argv) > offset + 1:
                return infer_agent(argv[offset + 1])
            if len(argv) > offset:
                return infer_agent(argv[offset])
        return None

    if exe in {"env", "command", "exec", "nohup", "time", "sudo"}:
        index = 1
        while index < len(argv):
            token = argv[index]
            if token.startswith("-") or (exe == "env" and "=" in token):
                index += 1
                continue
            break
        if index < len(argv):
            return detect_agent(argv[index:])
        return None

    if exe in {"node", "bun", "deno"}:
        if len(argv) >= 2:
            return infer_agent(argv[1])
        return None

    direct = infer_agent(argv[0])
    if direct:
        return direct

    # Accept a path to an agent launched by an interpreter or generic wrapper
    # (for example `python3 /path/to/agent`), but do not scan free text such as
    # `echo claude` and turn it into an agent launch.
    for token in argv[1:]:
        if token.startswith("-") or "/" not in token:
            continue
        if not os.path.exists(token):
            continue
        candidate = infer_agent(token)
        if candidate:
            return candidate
    return None


def profile_agent(instance: str) -> str | None:
    harness = shutil.which("harness")
    if not harness:
        return None
    try:
        result = subprocess.run(
            [harness, "profile", "--describe", instance],
            capture_output=True,
            text=True,
            timeout=8,
            cwd=str(HOME),
        )
    except Exception:
        return None
    if result.returncode != 0:
        return None
    try:
        described = shlex.split(result.stdout.strip())
    except ValueError:
        return None
    if not described:
        return None
    return detect_agent(described)


def session_id_from_json_files(paths: list[Path], cwd: str) -> str | None:
    if not paths:
        return None
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    for path in sorted(paths, key=key, reverse=True):
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                for _ in range(5):
                    line = handle.readline()
                    if not line:
                        break
                    try:
                        record = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if not isinstance(record, dict):
                        continue
                    if record.get("cwd") not in (None, cwd):
                        continue
                    for key_name in ("sessionId", "session_id", "id"):
                        value = record.get(key_name)
                        if isinstance(value, str) and value:
                            return value
        except OSError:
            continue
    # No inline id: the file stem is the id for Claude/Qwen style stores.
    try:
        return sorted(paths, key=key, reverse=True)[0].stem
    except Exception:
        return None


def encoded_path(cwd: str) -> str:
    return re.sub(r"[^A-Za-z0-9]", "-", cwd)


def discover_claude(cwd: str) -> str | None:
    projects = CLAUDE_HOME / "projects"
    directory = projects / encoded_path(cwd)
    candidates = list(directory.glob("*.jsonl")) if directory.is_dir() else []
    if candidates:
        return session_id_from_json_files(candidates, cwd)
    # Fall back to scanning projects if Claude used a different path encoding.
    candidates = []
    for path in projects.glob("*/*.jsonl"):
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                for _ in range(5):
                    line = handle.readline()
                    if not line:
                        break
                    if '"cwd"' in line:
                        record = json.loads(line)
                        if record.get("cwd") == cwd:
                            candidates.append(path)
                        break
        except Exception:
            continue
    return session_id_from_json_files(candidates, cwd)


def discover_codex(cwd: str) -> str | None:
    sessions = CODEX_HOME / "sessions"
    if not sessions.is_dir():
        return None
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    for path in sorted(sessions.rglob("rollout-*.jsonl"), key=key, reverse=True):
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                first = handle.readline()
            record = json.loads(first)
        except Exception:
            continue
        if record.get("type") != "session_meta":
            continue
        payload = record.get("payload") or {}
        if payload.get("cwd") != cwd:
            continue
        return payload.get("session_id") or payload.get("id")
    return None


def discover_opencode(cwd: str) -> str | None:
    database = XDG_DATA / "opencode" / "opencode.db"
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
    except sqlite3.Error:
        return None
    try:
        for table in ("session_v2", "session"):
            try:
                row = connection.execute(
                    f"select id from {table} where directory = ? "
                    "order by time_updated desc limit 1",
                    (cwd,),
                ).fetchone()
            except sqlite3.Error:
                continue
            if row and row[0]:
                return str(row[0])
    finally:
        connection.close()
    return None


def discover_pi(cwd: str) -> str | None:
    sessions = HOME / ".pi" / "agent" / "sessions"
    if not sessions.is_dir():
        return None
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    for path in sorted(sessions.rglob("*.jsonl"), key=key, reverse=True):
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                first = handle.readline()
            record = json.loads(first)
        except Exception:
            continue
        if record.get("type") != "session" or record.get("cwd") != cwd:
            continue
        return record.get("id") or path.stem
    return None


def discover_qwen(cwd: str) -> str | None:
    projects = HOME / ".qwen" / "projects"
    directory = projects / encoded_path(cwd) / "chats"
    candidates = (
        [path for path in directory.glob("*.jsonl") if not path.name.endswith(".runtime.json")]
        if directory.is_dir()
        else []
    )
    if candidates:
        return session_id_from_json_files(candidates, cwd)
    candidates = []
    for path in projects.glob("*/chats/*.jsonl"):
        if path.name.endswith(".runtime.json"):
            continue
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                for _ in range(5):
                    line = handle.readline()
                    if not line:
                        break
                    if '"cwd"' in line:
                        record = json.loads(line)
                        if record.get("cwd") == cwd:
                            candidates.append(path)
                        break
        except Exception:
            continue
    return session_id_from_json_files(candidates, cwd)


def discover_copilot(cwd: str) -> str | None:
    root = HOME / ".copilot" / "session-state"
    if not root.is_dir():
        return None
    found: list[tuple[str, str, float]] = []
    for workspace in root.glob("*/workspace.yaml"):
        session_id = None
        session_cwd = None
        updated = ""
        try:
            for line in workspace.read_text(encoding="utf-8", errors="replace").splitlines():
                if ":" not in line:
                    continue
                key, value = line.split(":", 1)
                value = value.strip()
                if key.strip() == "id":
                    session_id = value
                elif key.strip() == "cwd":
                    session_cwd = value
                elif key.strip() == "updated_at":
                    updated = value
        except OSError:
            continue
        if session_id and session_cwd == cwd:
            try:
                modified = workspace.stat().st_mtime
            except OSError:
                modified = 0.0
            found.append((modified, updated or "", session_id))
    if not found:
        return None
    return max(found)[2]


def discover_kimi(cwd: str) -> str | None:
    kimi = shutil.which("kimi")
    if not kimi:
        return None
    try:
        result = subprocess.run(
            [kimi, "session", "list", "--json", "--cwd", cwd, "--limit", "1"],
            capture_output=True,
            text=True,
            timeout=10,
            cwd=cwd,
        )
        payload = json.loads(result.stdout)
    except Exception:
        return None
    if isinstance(payload, list) and payload and isinstance(payload[0], dict):
        value = payload[0].get("id")
        if isinstance(value, str) and value:
            return value
    return None


def discover_mcode(cwd: str) -> str | None:
    database = HOME / ".minimax" / "v2" / "sqlite" / "runtime-state.sqlite"
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        row = connection.execute(
            "select session_id from local_runtime_sessions "
            "where workspace_dir = ? "
            "and coalesce(session_kind, 'conversation') != 'task' "
            "order by updated_at_ms desc limit 1",
            (cwd,),
        ).fetchone()
        connection.close()
    except sqlite3.Error:
        return None
    return str(row[0]) if row and row[0] else None


def discover_muse(cwd: str) -> str | None:
    database = XDG_DATA / "muse" / "session-index.db"
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        row = connection.execute(
            "select session_id from sessions where workspace_root = ? "
            "order by updated_at_us desc limit 1",
            (cwd,),
        ).fetchone()
        connection.close()
    except sqlite3.Error:
        return None
    return str(row[0]) if row and row[0] else None


def discover_vibe(cwd: str) -> str | None:
    root = HOME / ".vibe" / "logs" / "session" / "unified"
    if not root.is_dir():
        return None
    found: list[tuple[str, str]] = []
    for meta_path in root.glob("*/meta.json"):
        try:
            record = json.loads(meta_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        environment = record.get("environment") or {}
        working = environment.get("working_directory") or record.get("origin_directory")
        if working != cwd:
            continue
        session_id = record.get("session_id")
        if isinstance(session_id, str) and session_id:
            timestamp = record.get("end_time") or record.get("start_time") or ""
            found.append((str(timestamp), session_id))
    if not found:
        return None
    return max(found)[1]


def discover_cline(cwd: str) -> str | None:
    root = HOME / ".cline" / "data" / "sessions"
    if not root.is_dir():
        return None
    found: list[tuple[str, str]] = []
    for path in root.glob("*/*.json"):
        if path.name.endswith(".messages.json"):
            continue
        try:
            record = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        if record.get("cwd") != cwd:
            continue
        session_id = record.get("session_id")
        if isinstance(session_id, str) and session_id:
            timestamp = record.get("started_at") or ""
            try:
                modified = path.stat().st_mtime
            except OSError:
                modified = 0.0
            found.append((modified, str(timestamp), session_id))
    if not found:
        return None
    return max(found)[2]


def discover_grok(cwd: str) -> str | None:
    root = GROK_HOME / "sessions"
    if not root.is_dir():
        return None
    found: list[tuple[str, str]] = []
    for summary in root.glob("*/*/summary.json"):
        try:
            record = json.loads(summary.read_text(encoding="utf-8"))
        except Exception:
            continue
        info = record.get("info") or {}
        if info.get("cwd") != cwd:
            continue
        session_id = info.get("id")
        if isinstance(session_id, str) and session_id:
            timestamp = record.get("last_active_at") or record.get("updated_at") or ""
            found.append((str(timestamp), session_id))
    if not found:
        return None
    return max(found)[1]


def discover_crush(cwd: str) -> str | None:
    database = Path(cwd) / ".crush" / "crush.db"
    registry = XDG_DATA / "crush" / "projects.json"
    if not database.exists() and registry.exists():
        try:
            projects = json.loads(registry.read_text(encoding="utf-8")).get("projects") or []
        except Exception:
            projects = []
        matches: list[tuple[int, Path]] = []
        for project in projects:
            project_path = str(project.get("path") or "")
            data_dir = str(project.get("data_dir") or "")
            if not project_path or not data_dir:
                continue
            if cwd == project_path or cwd.startswith(project_path.rstrip("/") + "/"):
                matches.append((len(project_path), Path(data_dir) / "crush.db"))
        if matches:
            database = max(matches, key=lambda item: item[0])[1]
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        row = connection.execute(
            "select id from sessions order by updated_at desc limit 1"
        ).fetchone()
        connection.close()
    except sqlite3.Error:
        return None
    return str(row[0]) if row and row[0] else None


def discover_omp(cwd: str) -> str | None:
    root = HOME / ".omp" / "agent" / "sessions"
    if not root.is_dir():
        return None
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    for path in sorted(root.rglob("*.jsonl"), key=key, reverse=True):
        try:
            with path.open("r", encoding="utf-8", errors="replace") as handle:
                for _ in range(5):
                    line = handle.readline()
                    if not line:
                        break
                    record = json.loads(line)
                    if record.get("type") == "session":
                        if record.get("cwd") != cwd:
                            break
                        return record.get("id") or path.stem
        except Exception:
            continue
    return None


def discover_kilo(cwd: str) -> str | None:
    database = XDG_DATA / "kilo" / "kilo.db"
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
    except sqlite3.Error:
        return None
    try:
        for table in ("session", "session_v2"):
            try:
                row = connection.execute(
                    f"select id from {table} where directory = ? "
                    "order by time_updated desc limit 1",
                    (cwd,),
                ).fetchone()
            except sqlite3.Error:
                continue
            if row and row[0]:
                return str(row[0])
    finally:
        connection.close()
    return None


def discover_antigravity(cwd: str) -> str | None:
    database = HOME / ".gemini" / "antigravity-cli" / "conversation_summaries.db"
    if not database.exists():
        return None
    try:
        connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        rows = connection.execute(
            "select conversation_id, workspace_uris from conversation_summaries "
            "order by last_modified_time desc limit 200"
        ).fetchall()
        connection.close()
    except sqlite3.Error:
        return None
    wanted = f"file://{cwd.rstrip('/')}"
    for conversation_id, workspace_uris in rows:
        try:
            uris = json.loads(workspace_uris or "[]")
        except json.JSONDecodeError:
            uris = [str(workspace_uris or "")]
        if any(str(uri).rstrip("/") == wanted for uri in uris):
            return str(conversation_id)
    return None


def discover_deepseek(cwd: str) -> str | None:
    sessions = DSH_HOME / "sessions"
    if not sessions.is_dir():
        return None
    encoded = "--" + cwd.strip("/").replace("/", "-") + "--"
    directory = sessions / encoded
    if not directory.is_dir():
        return None
    candidates = [path for path in directory.iterdir() if path.is_dir()]
    if not candidates:
        return None
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    return sorted(candidates, key=key, reverse=True)[0].name


def gemini_session_files() -> list[Path]:
    root = GEMINI_HOME / "tmp"
    if not root.is_dir():
        return []
    return list(root.glob("*/chats/session-*.jsonl"))


def gemini_session_header(path: Path) -> dict:
    try:
        with path.open("r", encoding="utf-8", errors="replace") as handle:
            record = json.loads(handle.readline())
    except Exception:
        return {}
    return record if isinstance(record, dict) else {}


def discover_gemini(cwd: str) -> str | None:
    wanted_hash = hashlib.sha256(cwd.encode("utf-8")).hexdigest()
    def key(path: Path) -> float:
        try:
            return path.stat().st_mtime
        except OSError:
            return 0.0
    for path in sorted(gemini_session_files(), key=key, reverse=True):
        header = gemini_session_header(path)
        if header.get("projectHash") != wanted_hash:
            continue
        session_id = header.get("sessionId")
        if isinstance(session_id, str) and session_id:
            return session_id
    return None


def gemini_session_file(session_id: str) -> str | None:
    for path in gemini_session_files():
        header = gemini_session_header(path)
        if header.get("sessionId") == session_id:
            return str(path)
    return None


def discover_fx(cwd: str) -> str | None:
    binary = shutil.which("fx")
    if not binary:
        candidate = HOME / ".local" / "libexec" / "fx" / "fx-router"
        binary = str(candidate) if candidate.exists() else None
    if not binary:
        return None
    try:
        result = subprocess.run(
            [binary, "sessions", "--json", "--limit", "1"],
            capture_output=True,
            text=True,
            timeout=10,
            cwd=cwd,
        )
        payload = json.loads(result.stdout)
    except Exception:
        return None
    sessions = payload.get("sessions") if isinstance(payload, dict) else None
    if sessions and isinstance(sessions[0], dict):
        value = sessions[0].get("id")
        if isinstance(value, str) and value:
            return value
    return None


DISCOVERY = {
    "claude": discover_claude,
    "codex": discover_codex,
    "opencode": discover_opencode,
    "pi": discover_pi,
    "qwen": discover_qwen,
    "copilot": discover_copilot,
    "gemini": discover_gemini,
    "kimi": discover_kimi,
    "mcode": discover_mcode,
    "muse": discover_muse,
    "vibe": discover_vibe,
    "cline": discover_cline,
    "grok": discover_grok,
    "crush": discover_crush,
    "omp": discover_omp,
    "kilo": discover_kilo,
    "antigravity": discover_antigravity,
    "deepseek": discover_deepseek,
    "dsh-tui": discover_deepseek,
    "fx": discover_fx,
    "fx-router": discover_fx,
}


def discover(agent: str, cwd: str) -> str | None:
    function = DISCOVERY.get(agent)
    if not function:
        return None
    try:
        return function(cwd)
    except Exception:
        return None


def is_noninteractive(agent: str, args: list[str]) -> bool:
    """Do not turn one-shot/headless invocations into interactive resumptions."""
    if agent == "cline" and not any(arg in {"-i", "--tui"} for arg in args):
        return True
    if agent in {"codex", "opencode", "crush", "mcode", "muse"}:
        leading = [arg for arg in args[:2] if not arg.startswith("-")]
        if any(arg in {"exec", "run"} for arg in leading):
            return True
    if agent in {"fx", "fx-router"}:
        leading = [arg for arg in args[:2] if not arg.startswith("-")]
        if any(arg in {"ask", "session", "sessions"} for arg in leading):
            return True
    print_flags = {
        "claude": {"-p", "--print"},
        "pi": {"-p", "--print"},
        "qwen": {"-p", "--prompt"},
        "copilot": {"-p", "--prompt"},
        "gemini": {"-p", "--prompt"},
        "vibe": {"-p", "--prompt"},
        "grok": {"-p", "--single"},
        "omp": {"-p", "--print"},
        "kimi": {"-p", "--prompt"},
        "kimi-cli": {"-p", "--prompt"},
        "antigravity": {"-p", "--prompt"},
        "deepseek": {"-p", "--prompt"},
        "dsh-tui": {"-p", "--prompt"},
    }
    return any(arg in print_flags.get(agent, set()) for arg in args)


def resume_args(agent: str, session_id: str | None) -> list[str]:
    if agent == "claude":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "codex":
        return ["resume", session_id] if session_id else ["resume", "--last"]
    if agent == "opencode":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "pi":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "qwen":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "copilot":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "kimi":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "kimi-cli":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "mcode":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "muse":
        return ["resume", session_id] if session_id else ["resume", "--last"]
    if agent == "vibe":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "gemini":
        if session_id:
            path = gemini_session_file(session_id)
            if path:
                return ["--session-file", path]
        return ["--resume", "latest"]
    if agent == "cline":
        return ["--id", session_id] if session_id else []
    if agent == "cursor-agent":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "grok":
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent == "crush":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "omp":
        return [f"--resume={session_id}"] if session_id else ["--continue"]
    if agent == "kilo":
        return ["--session", session_id] if session_id else ["--continue"]
    if agent == "antigravity":
        return ["--conversation", session_id] if session_id else ["--continue"]
    if agent in {"deepseek", "dsh-tui"}:
        return ["--resume", session_id] if session_id else ["--continue"]
    if agent in {"fx", "fx-router"}:
        return ["--continue"]
    return []


SHORT_RESUME_FLAGS = {
    "claude": {"-c", "-r"},
    "opencode": {"-c", "-s"},
    "pi": {"-c", "-r"},
    "qwen": {"-c", "-r"},
    "copilot": {"-r"},
    "kimi": {"-c", "-S"},
    "kimi-cli": {"-c", "-r"},
    "mcode": {"-c"},
    "vibe": {"-c"},
    "gemini": {"-r"},
    "grok": {"-c", "-r"},
    "crush": {"-C", "-s"},
    "omp": {"-c", "-r"},
    "kilo": {"-c", "-s"},
    "antigravity": {"-c"},
    "deepseek": {"-c"},
    "dsh-tui": {"-c"},
}


def strip_resume_args(agent: str, args: list[str]) -> list[str]:
    """Remove a previous resume target so the hook's current id wins."""
    value_flags = {"--resume", "--session", "--conversation", "--id", "--session-file"}
    short_value_flags = {
        "claude": {"-r"},
        "opencode": {"-s"},
        "pi": {"-r"},
        "qwen": {"-r"},
        "copilot": {"-r"},
        "kimi": {"-S"},
        "kimi-cli": {"-r"},
        "gemini": {"-r"},
        "grok": {"-r"},
        "crush": {"-s"},
        "omp": {"-r"},
        "kilo": {"-s"},
    }
    toggle_flags = {"--continue"}
    short_toggle_flags = {
        "claude": {"-c"},
        "opencode": {"-c"},
        "pi": {"-c"},
        "qwen": {"-c"},
        "kimi": {"-c"},
        "kimi-cli": {"-c"},
        "mcode": {"-c"},
        "vibe": {"-c"},
        "grok": {"-c"},
        "crush": {"-C"},
        "omp": {"-c"},
        "kilo": {"-c"},
        "antigravity": {"-c"},
        "deepseek": {"-c"},
        "dsh-tui": {"-c"},
    }

    def is_resume_subcommand() -> bool:
        return agent in {"codex", "muse"}

    cleaned: list[str] = []
    index = 0
    while index < len(args):
        token = args[index]
        if token in value_flags or token in short_value_flags.get(agent, set()):
            index += 2
            continue
        if any(token.startswith(flag + "=") for flag in value_flags):
            index += 1
            continue
        if token in toggle_flags or token in short_toggle_flags.get(agent, set()):
            index += 1
            continue
        if is_resume_subcommand() and token == "resume":
            index += 1
            if index < len(args) and not args[index].startswith("-"):
                index += 1
            elif index < len(args) and args[index] == "--last":
                index += 1
            continue
        if agent in {"fx", "fx-router"} and token == "session":
            if index + 1 < len(args) and args[index + 1] == "resume":
                index += 2
                if index < len(args) and not args[index].startswith("-"):
                    index += 1
                continue
        cleaned.append(token)
        index += 1
    return cleaned


def already_resuming(agent: str, args: list[str]) -> bool:
    if agent == "codex" and "resume" in args:
        return True
    if agent == "muse" and "resume" in args:
        return True
    for flag in ("--resume", "--continue", "--session", "--conversation", "--id", "--session-file"):
        if flag in args or any(arg.startswith(flag + "=") for arg in args):
            return True
    return any(arg in SHORT_RESUME_FLAGS.get(agent, set()) for arg in args)


def exec_command(argv: list[str], original: str) -> None:
    if not argv:
        return
    try:
        os.execvp(argv[0], argv)
    except OSError:
        os.execvp("/bin/sh", ["sh", "-c", original])


def isolate_agent(agent: str, argv: list[str]) -> list[str]:
    """Give OpenCode a private server so panes do not share OSC 7501 status."""
    if agent != "opencode" or not argv:
        return argv
    for arg in argv[1:]:
        if arg in {"--standalone", "--server"} or arg.startswith("--server="):
            return argv
    if basename(argv[0]) == "zj" and len(argv) > 1 and argv[1] == "__shim":
        # The shim adds the flag to the child command it launches.
        return argv
    for index, token in enumerate(argv):
        if basename(token).lower() == "opencode":
            return argv[: index + 1] + ["--standalone"] + argv[index + 1 :]
    return argv


def wrap(encoded: str, explicit_session_id: str | None = None) -> int:
    try:
        original = base64.b64decode(encoded.encode("ascii")).decode("utf-8")
    except Exception as error:
        print(f"zj: could not decode resurrect command: {error}", file=sys.stderr)
        return 2
    try:
        argv = shlex.split(original)
    except ValueError:
        os.execvp("/bin/sh", ["sh", "-c", original])
        return 0

    agent = detect_agent(argv)
    if not agent:
        exec_command(argv, original)
        return 0
    if is_noninteractive(agent, argv[1:]):
        exec_command(argv, original)
        return 0
    # A restored pane should come back idle, not working, and should get a
    # remembered idle status so it is visible in the Inbox. Set the title
    # first so the Inbox labels it with the agent rather than the wrapper.
    emit_pane_title(display_app(agent))
    prime_agent_status(agent)
    if explicit_session_id:
        argv = [argv[0]] + strip_resume_args(agent, argv[1:])
        extra = resume_args(agent, explicit_session_id)
        if extra:
            argv = argv + extra
        exec_command(isolate_agent(agent, argv), original)
        return 0
    if already_resuming(agent, argv[1:]):
        exec_command(isolate_agent(agent, argv), original)
        return 0

    session_id = discover(agent, current_directory())
    extra = resume_args(agent, session_id)
    if extra:
        argv = argv + extra
    exec_command(isolate_agent(agent, argv), original)
    return 0


def hook() -> int:
    original = os.environ.get("RESURRECT_COMMAND", "").strip()
    if not original:
        return 0
    if "__resume-wrap" in original:
        print(original)
        return 0
    try:
        argv = shlex.split(original)
    except ValueError:
        print(original)
        return 0
    agent = detect_agent(argv)
    if not agent:
        print(original)
        return 0
    if is_noninteractive(agent, argv[1:]):
        print(original)
        return 0
    encoded = base64.b64encode(original.encode("utf-8")).decode("ascii")
    print(f"zj __resume-wrap {encoded}")
    return 0


def kdl_unescape(value: str) -> str:
    replacements = {
        "\\": "\\",
        '"': '"',
        "/": "/",
        "b": "\b",
        "f": "\f",
        "n": "\n",
        "r": "\r",
        "t": "\t",
    }
    return re.sub(r"\\(.)", lambda match: replacements.get(match.group(1), match.group(1)), value)


def parse_kdl_properties(header: str) -> dict[str, str]:
    body = header.split("{", 1)[0]
    properties: dict[str, str] = {}
    pattern = re.compile(r'([A-Za-z_][\w-]*)=("(?:[^"\\]|\\.)*"|[^\s]+)')
    for match in pattern.finditer(body):
        key = match.group(1)
        value = match.group(2)
        if value.startswith('"') and value.endswith('"'):
            value = kdl_unescape(value[1:-1])
        properties[key] = value
    return properties


def parse_kdl_strings(text: str) -> list[str]:
    values: list[str] = []
    index = 0
    length = len(text)
    while index < length:
        if text[index].isspace():
            index += 1
            continue
        if text[index] != '"':
            start = index
            while index < length and not text[index].isspace():
                index += 1
            values.append(text[start:index])
            continue
        index += 1
        buffer: list[str] = []
        while index < length:
            character = text[index]
            if character == "\\" and index + 1 < length:
                following = text[index + 1]
                buffer.append({
                    "b": "\b",
                    "f": "\f",
                    "n": "\n",
                    "r": "\r",
                    "t": "\t",
                    '"': '"',
                    "\\": "\\",
                    "/": "/",
                }.get(following, following))
                index += 2
                continue
            if character == '"':
                index += 1
                break
            buffer.append(character)
            index += 1
        values.append("".join(buffer))
    return values


def brace_delta(line: str) -> int:
    depth = 0
    in_string = False
    escaped = False
    for character in line:
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
        elif character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
    return depth


def rewrite_layout(text: str) -> str:
    """Make saved commands auto-run and route agents through the resume wrapper."""
    lines = text.splitlines(keepends=True)
    output: list[str] = []
    index = 0
    while index < len(lines):
        line = lines[index]
        if not re.match(r"^[ \t]*pane\b", line) or "command=" not in line:
            output.append(line)
            index += 1
            continue

        depth = brace_delta(line)
        end = index + 1
        while depth > 0 and end < len(lines):
            depth += brace_delta(lines[end])
            end += 1
        block = lines[index:end]

        header = block[0]
        properties = parse_kdl_properties(header)
        command = properties.get("command")
        args: list[str] = []
        args_index = None
        for offset in range(1, len(block)):
            match = re.match(r"^([ \t]*)args\s+(.*?)\s*$", block[offset], re.MULTILINE)
            if match:
                args = parse_kdl_strings(match.group(2))
                args_index = offset
                break

        if command:
            argv = [command] + args
            agent = detect_agent(argv)
            if agent and not is_noninteractive(agent, args):
                original = shlex.join(argv)
                encoded = base64.b64encode(original.encode("utf-8")).decode("ascii")
                header = re.sub(
                    r'command=("(?:[^"\\]|\\.)*")',
                    'command="zj"',
                    header,
                    count=1,
                )
                indentation = " " * (len(header) - len(header.lstrip()) + 4)
                new_args = f'{indentation}args "__resume-wrap" "{encoded}"\n'
                if args_index is not None:
                    block[args_index] = new_args
                else:
                    block.insert(1, new_args)
                block[0] = header

        block = [
            line
            for line in block
            if not re.match(r"^[ \t]*start_suspended\b", line)
        ]
        block[0] = re.sub(r"[ \t]+start_suspended=(?:true|false)", "", block[0])
        output.extend(block)
        index = end
    return "".join(output)


def session_info_directories() -> list[Path]:
    if os.environ.get("ZJ_ZELLIJ_CACHE_DIR"):
        return [ZELLIJ_CACHE]
    base = ZELLIJ_CACHE.parent.parent  # ~/.cache/zellij
    directories = sorted(base.glob("*/session_info"))
    if ZELLIJ_CACHE not in directories:
        directories.append(ZELLIJ_CACHE)
    return directories


def session_index_databases() -> list[Path]:
    if os.environ.get("ZJ_ZELLIJ_CACHE_DIR"):
        database = ZELLIJ_CACHE.parent / "session_index.sqlite"
    else:
        base = ZELLIJ_CACHE.parent.parent  # ~/.cache/zellij
        databases = sorted(base.glob("*/session_index.sqlite"))
        database = ZELLIJ_CACHE.parent / "session_index.sqlite"
        if database not in databases:
            databases.append(database)
        return [path for path in databases if path.is_file()]
    return [database] if database.is_file() else []


def indexed_layout(session_name: str) -> Path | None:
    """Resolve a saved layout through Zellij's session index.

    Zellij writes session-metadata.kdl only on a clean shutdown, so a space
    whose server was killed (or that was saved mid-run) can still be
    resurrectable while lacking metadata. The index always has the name-to-
    directory mapping.
    """
    for database in session_index_databases():
        try:
            connection = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        except sqlite3.Error:
            continue
        try:
            row = connection.execute(
                "select id from sessions where name = ?", (session_name,)
            ).fetchone()
        except sqlite3.Error:
            row = None
        finally:
            connection.close()
        if not row or not row[0]:
            continue
        directory_id = str(row[0])
        for session_info in session_info_directories():
            layout = session_info / directory_id / "session-layout.kdl"
            if layout.is_file():
                return layout
    return None


def find_saved_layout(session_name: str) -> Path | None:
    indexed = indexed_layout(session_name)
    if indexed is not None:
        return indexed
    candidates: list[Path] = []
    for session_info in session_info_directories():
        if not session_info.is_dir():
            continue
        for directory in session_info.iterdir():
            if not directory.is_dir():
                continue
            metadata = directory / "session-metadata.kdl"
            layout = directory / "session-layout.kdl"
            if not metadata.is_file() or not layout.is_file():
                continue
            try:
                text = metadata.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            match = re.search(r'^\s*name\s+"((?:[^"\\]|\\.)*)"', text, re.MULTILINE)
            if not match:
                continue
            if kdl_unescape(match.group(1)) == session_name:
                candidates.append(layout)
    if not candidates:
        return None
    return max(candidates, key=lambda path: path.stat().st_mtime)


def prepare_restore(session_name: str) -> int:
    layout = find_saved_layout(session_name)
    if not layout:
        return 0
    try:
        text = layout.read_text(encoding="utf-8")
    except OSError as error:
        print(f"zj: could not read {layout}: {error}", file=sys.stderr)
        return 1

    updated = rewrite_layout(text)

    if updated != text:
        try:
            temporary = layout.with_name(f".{layout.name}.zj-restore")
            temporary.write_text(updated, encoding="utf-8")
            os.replace(temporary, layout)
        except OSError as error:
            print(f"zj: could not update {layout}: {error}", file=sys.stderr)
            return 1
    return 0


def resume_command(agent: str, cwd: str) -> int:
    normalized = normalize(agent) or agent
    session_id = discover(normalized, cwd)
    print(json.dumps({
        "agent": normalized,
        "cwd": cwd,
        "session_id": session_id,
        "args": resume_args(normalized, session_id),
    }))
    return 0


def space_hooks_command(argv: list[str]) -> int:
    if hook_core is None:
        print(json.dumps({"ok": False, "version": 0, "panes": {}, "sessions": {}}))
        return 0
    if argv and argv[0] == "--detail":
        if len(argv) < 3:
            print("usage: zj __space-hooks --detail SESSION PANE", file=sys.stderr)
            return 2
        try:
            pane: object = int(argv[2])
        except ValueError:
            pane = argv[2]
        print(
            json.dumps(
                hook_core.space_hooks_detail(argv[1], pane),
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            )
        )
        return 0
    print(
        json.dumps(
            hook_core.space_hooks_summary(),
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
    )
    return 0


def hook_server_command(argv: list[str]) -> int:
    if hook_core is None:
        return 2
    daemon = "--daemon" in argv
    tcp_port: int | None = None
    if "--tcp" in argv:
        index = argv.index("--tcp")
        if index + 1 < len(argv):
            try:
                tcp_port = int(argv[index + 1])
            except ValueError:
                tcp_port = None
    if daemon:
        return hook_core.daemonize()
    return hook_core.serve(foreground=True, tcp_port=tcp_port)


def hooks_command(argv: list[str]) -> int:
    if hook_core is None:
        print("zj hooks: hook core unavailable", file=sys.stderr)
        return 2
    sub = argv[0] if argv else "status"
    if sub == "status":
        return hook_core.hooks_status()
    if sub == "stop":
        return hook_core.hooks_stop()
    if sub in {"start", "restart"}:
        return hook_core.hooks_start(os.path.abspath(__file__))
    if sub == "serve":
        return hook_core.serve(foreground=True)
    print("usage: zj hooks [status|start|stop|restart|serve]", file=sys.stderr)
    return 2


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(
            "usage: zj-resume <hook|hooks|hook-server|wrap|prepare-restore|session-hook|space-agents|space-snapshot|space-hooks|resume-command> ...",
            file=sys.stderr,
        )
        return 2
    command = argv[1]
    if command in {"hook", "agent-hook"}:
        if len(argv) >= 3:
            return hook_client(argv[2], argv[3] if len(argv) == 4 else None)
        # No agent argument: Zellij's post_command_discovery_hook.
        return hook()
    if command == "hooks":
        return hooks_command(argv[2:])
    if command in {"hook-server", "hook_server"}:
        return hook_server_command(argv[2:])
    if command in {"space-hooks", "space_hooks"}:
        return space_hooks_command(argv[2:])
    if command == "wrap":
        if len(argv) not in (3, 4):
            print("usage: zj __resume-wrap <base64> [session-id]", file=sys.stderr)
            return 2
        return wrap(argv[2], argv[3] if len(argv) == 4 else None)
    if command in {"prepare-restore", "prepare_restore"}:
        if len(argv) != 3:
            print("usage: zj __prepare-restore <session>", file=sys.stderr)
            return 2
        return prepare_restore(argv[2])
    if command in {"session-hook", "session_hook"}:
        if len(argv) not in (3, 4):
            print("usage: zj session-hook <agent> [start|prompt|stop]", file=sys.stderr)
            return 2
        return session_hook(argv[2], argv[3] if len(argv) == 4 else None)
    if command in {"space-agents", "space_agents"}:
        return space_agents()
    if command in {"space-snapshot", "space_snapshot"}:
        return space_snapshot()
    if command in {"resume-command", "resume_command"}:
        if len(argv) not in (3, 4):
            print("usage: zj resume-command <agent> [cwd]", file=sys.stderr)
            return 2
        return resume_command(argv[2], argv[3] if len(argv) == 4 else current_directory())
    print(f"zj-resume: unknown command {command}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
