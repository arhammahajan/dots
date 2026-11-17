#!/usr/bin/env python3
"""Shared per-agent hook core for the Zellij Spaces navigator.

The harness-facing client lives in ``zj-resume.py`` (it needs the OSC 7501
emitter and the resurrect-command logic).  This module owns the parts that both
the client and the background listener need:

* provider event canonicalization and payload normalization;
* merging an event into a pane record;
* the per-pane store and the append-only spool;
* HTTP-over-Unix-socket publication and the optional loopback listener;
* the ``__space-hooks`` summary/detail views used by the plugin.

Nothing here executes input from a payload.  The listener only parses JSON,
merges a record and writes files.
"""
from __future__ import annotations

import errno
import fcntl
import http.client
import json
from http.server import BaseHTTPRequestHandler
import os
import re
import secrets
import signal
import socket
import socketserver
import struct
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse
from pathlib import Path
from typing import Any, Mapping

VERSION = 1

# Freshness / pruning windows (milliseconds).
WORKING_STALE_MS = 120_000
DONE_TTL_MS = 30 * 60_000
FAILED_TTL_MS = 30 * 60_000
BLOCK_TTL_MS = 30 * 60_000
AUTH_TTL_MS = 60 * 60_000
SPOOL_TTL_MS = 60 * 60_000

# Field / file bounds.
MAX_TEXT = 240
MAX_FIELD = 4096
MAX_OUTPUT = 2048
MAX_MODEL = 200
MAX_SESSION = 200
MAX_CWD = 1000
MAX_CHILDREN = 32
MAX_EVENT_BYTES = 16 * 1024
MAX_SPOOL_BYTES = 1024 * 1024
MAX_SPOOL_LINES = 2048
MAX_BODY = 256 * 1024
DEDUPE_MS = 500
START_THROTTLE_S = 10

_CONTROL = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")

ALIASES = {
    "agy": "antigravity",
    "dsh": "deepseek",
    "kimi-code": "kimi",
    "cursor": "cursor-agent",
}

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
    "kiro",
    "droid",
    "amp",
}


def now_ms() -> int:
    return int(time.time() * 1000)


def _basename(value: str) -> str:
    value = value.rstrip("/")
    return os.path.basename(value)


# ---------------------------------------------------------------------------
# Runtime paths
# ---------------------------------------------------------------------------


def disabled() -> bool:
    return os.environ.get("ZJ_HOOKS_DISABLE", "").lower() in {"1", "true", "yes"}


def runtime_dir() -> Path:
    configured = os.environ.get("ZJ_HOOKS_RUNTIME")
    if configured:
        return Path(configured).expanduser()
    xdg = os.environ.get("XDG_RUNTIME_DIR")
    if xdg:
        return Path(xdg) / "zj"
    return Path(tempfile.gettempdir()) / f"zj-{os.getuid()}"


def store_path() -> Path:
    configured = os.environ.get("ZJ_HOOKS_STORE")
    return Path(configured).expanduser() if configured else runtime_dir() / "hooks.store.json"


def endpoint_path() -> Path:
    return runtime_dir() / "hooks.endpoint.json"


def socket_path() -> Path:
    return runtime_dir() / "h.sock"


def spool_path() -> Path:
    return runtime_dir() / "hooks-spool.jsonl"


def lock_path() -> Path:
    """Server-ownership lock.  Store writes use their own lock."""
    return runtime_dir() / "hooks.server.lock"


def store_lock_path() -> Path:
    return runtime_dir() / "hooks.store.lock"


def startstamp_path() -> Path:
    return runtime_dir() / "hooks.startstamp"


def log_path() -> Path:
    return runtime_dir() / "hooks.log"


def ensure_dir() -> Path:
    directory = runtime_dir()
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    try:
        os.chmod(directory, 0o700)
    except OSError:
        pass
    return directory


def _atomic_write(path: Path, data: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    fd, temp_name = tempfile.mkstemp(dir=str(path.parent), prefix=f".{path.name}.", suffix=".tmp")
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(temp_name, mode)
        os.replace(temp_name, path)
    except Exception:
        try:
            os.unlink(temp_name)
        except OSError:
            pass
        raise


def _read_json(path: Path) -> Any:
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return None


def _log(message: str) -> None:
    try:
        ensure_dir()
        path = log_path()
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{time.strftime('%Y-%m-%dT%H:%M:%S')} {message}\n")
        try:
            os.chmod(path, 0o600)
        except OSError:
            pass
    except OSError:
        pass


# ---------------------------------------------------------------------------
# Normalization
# ---------------------------------------------------------------------------


def canonical_agent(agent: str, env: Mapping[str, str] | None = None) -> str:
    env = env or os.environ
    override = env.get("ZJ_HOOK_AGENT")
    if override:
        agent = override
    # Grok reads ~/.claude/settings.json and ~/.cursor/hooks.json.  Those hooks
    # execute with Grok's runner variables, so trust the runtime over the argv
    # hint.  Do not use GROK_HOME: users export it globally.
    if env.get("GROK_SESSION_ID") or env.get("GROK_HOOK_EVENT"):
        return "grok"
    name = _basename(agent or "").lower()
    name = re.sub(r"\.(js|mjs|cjs|py|exe|cmd|bat)$", "", name)
    name = ALIASES.get(name, name)
    return name or "unknown"


def display_app(agent: str) -> str:
    name = canonical_agent(agent, {})
    return APP_NAMES.get(name, name)


_EVENT_ALIASES: dict[str, str] = {}


def _alias(canonical: str, *names: str) -> None:
    for name in names:
        _EVENT_ALIASES[re.sub(r"[^a-z0-9]", "", name.lower())] = canonical


_alias("session_start", "SessionStart", "session_start", "start")
_alias("session_end", "SessionEnd", "session_end", "end")
_alias(
    "prompt",
    "UserPromptSubmit",
    "userPromptSubmitted",
    "user_prompt_submit",
    "beforeSubmitPrompt",
    "BeforeAgent",
    "prompt",
    "busy",
    "sessionBusy",
    "session.execution.started",
)
_alias(
    "pre_tool",
    "PreToolUse",
    "preToolUse",
    "pre_tool_use",
    "BeforeTool",
    "beforeShellExecution",
    "beforeMCPExecution",
    "toolCall",
    "tool.call",
    "tool.execute.before",
)
_alias(
    "post_tool",
    "PostToolUse",
    "postToolUse",
    "post_tool_use",
    "AfterTool",
    "PostToolBatch",
    "afterAgentThought",
    "toolResult",
    "tool.result",
    "tool.execute.after",
)
_alias("tool_failure", "PostToolUseFailure", "postToolUseFailure", "tool_failure", "ToolError")
_alias(
    "permission",
    "PermissionRequest",
    "permissionRequest",
    "permissionRequested",
    "PermissionDenied",
)
_alias(
    "stop",
    "Stop",
    "stop",
    "agentStop",
    "AfterAgent",
    "Interrupt",
    "StopCancelled",
    "SessionIdle",
    "turnEnd",
    "session.execution.succeeded",
    "session.execution.interrupted",
    "session.idle",
)
_alias("response", "afterAgentResponse", "AgentResponse", "agentResponse")
_alias(
    "stop_failure",
    "StopFailure",
    "stopFailure",
    "ErrorOccurred",
    "session.execution.failed",
)
_alias("subagent_start", "SubagentStart", "subagentStart", "SubagentStart")
_alias("subagent_stop", "SubagentStop", "subagentStop", "SubagentEnd", "TeammateIdle")
_alias("compact", "PreCompact", "PostCompact", "PreCompress", "compact")
_alias("notification", "Notification", "notification")
_alias("elicitation", "Elicitation")
_alias("elicitation_result", "ElicitationResult")
_alias("working", "working", "agent.start")
_alias("done", "done", "agent.end")
_alias("error", "error", "failed")
_alias("blocked", "blocked", "needs_you", "needsYou")


def canonical_event(raw: str | None) -> str | None:
    if not raw:
        return None
    key = re.sub(r"[^a-z0-9]", "", str(raw).lower())
    return _EVENT_ALIASES.get(key)


def _first(payload: Mapping[str, Any], keys: tuple[str, ...]) -> Any:
    for key in keys:
        if key in payload:
            value = payload[key]
            if value is not None and value != "":
                return value
    for container in ("data", "session", "thread", "conversation", "_meta"):
        nested = payload.get(container)
        if isinstance(nested, Mapping):
            value = _first(nested, keys)
            if value is not None:
                return value
    return None


def _first_str(payload: Mapping[str, Any], keys: tuple[str, ...]) -> str | None:
    value = _first(payload, keys)
    if isinstance(value, str) and value.strip():
        return value.strip()
    return None


def _one_line(value: Any, limit: int = MAX_TEXT, keep_tail: bool = False) -> str:
    if value is None:
        return ""
    if not isinstance(value, str):
        try:
            value = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        except (TypeError, ValueError):
            value = str(value)
    value = _CONTROL.sub(" ", value)
    value = re.sub(r"\s+", " ", value).strip()
    if len(value) <= limit:
        return value
    if keep_tail:
        return "…" + value[-(limit - 1):]
    return value[: max(1, limit - 1)] + "…"


def _field(value: Any, limit: int = MAX_FIELD, keep_tail: bool = False) -> str | None:
    text = _one_line(value, limit, keep_tail)
    return text or None


def _tool_text(value: Any) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        return " ".join(_tool_text(item) for item in value if item)
    if isinstance(value, Mapping):
        for key in (
            "command",
            "file_path",
            "path",
            "pattern",
            "url",
            "query",
            "description",
            "prompt",
            "content",
            "new_string",
            "old_string",
        ):
            candidate = value.get(key)
            if isinstance(candidate, str) and candidate.strip():
                return candidate.strip()
        for candidate in value.values():
            if isinstance(candidate, str) and candidate.strip():
                return candidate.strip()
    return ""


def build_preview(tool: str | None, tool_input: Any) -> str:
    tool = (tool or "").strip()
    preview = _tool_text(tool_input)
    preview = _one_line(preview, MAX_TEXT)
    if tool and preview:
        if len(preview) > 120:
            preview = preview[:119] + "…"
        return f"{tool} · {preview}"
    return tool or preview


def _looks_auth(payload: Mapping[str, Any]) -> bool:
    text = " ".join(
        str(value)
        for value in (
            payload.get("message"),
            payload.get("reason"),
            payload.get("notification_type"),
            payload.get("notificationType"),
            payload.get("text"),
        )
        if isinstance(value, str)
    ).lower()
    return any(
        needle in text
        for needle in ("sign in", "sign-in", "log in", "login", "authenticate", "auth required", "api key")
    )


_ASK_TOOLS = {
    "askuserquestion",
    "ask_user_question",
    "request_user_input",
    "requestuserinput",
    "ask_user",
    "askuser",
}


def _tool_is_ask(tool: str | None) -> bool:
    return (tool or "").strip().lower() in _ASK_TOOLS


def event_state(
    event: str,
    payload: Mapping[str, Any],
    existing: Mapping[str, Any] | None = None,
) -> tuple[str | None, str | None]:
    if event == "session_start":
        return "idle", None
    if event == "session_end":
        return "idle", None
    if event == "prompt":
        return "working", None
    if event == "pre_tool":
        tool = _first_str(payload, ("tool_name", "toolName", "name", "tool"))
        if _tool_is_ask(tool):
            return "blocked", "question"
        return "working", None
    if event in {"post_tool", "tool_failure", "compact", "response"}:
        return "working", None
    if event == "permission":
        return "blocked", "auth" if _looks_auth(payload) else "permission"
    if event == "elicitation":
        return "blocked", "question"
    if event == "elicitation_result":
        return "working", None
    if event == "stop":
        return "done", None
    if event == "stop_failure":
        return "failed", None
    if event in {"subagent_start", "subagent_stop"}:
        return "working", None
    if event == "notification":
        notification = _first_str(
            payload, ("notification_type", "notificationType", "type", "notification")
        )
        lowered = (notification or "").lower()
        message = _first_str(payload, ("message", "body", "text", "title")) or ""
        message_lower = message.lower()
        if (
            "permission" in lowered
            or "toolpermission" in lowered
            or "approval" in lowered
            or "permission" in message_lower
            or "approve" in message_lower
        ):
            return "blocked", "permission"
        if "elicitation" in lowered or "question" in lowered:
            return "blocked", "question"
        if "idle" in lowered:
            # idle_prompt is a turn-end report in Claude, Qwen and Grok.  Treat
            # it as done; the config's Stop/AfterAgent events normally arrive
            # first and set the same state.
            return "done", None
        if "complete" in lowered:
            return "done", None
        if _looks_auth(payload) or "auth" in (message or "").lower():
            return "blocked", "auth"
        return None, None
    if event in {"working", "blocked", "done", "error", "idle"}:
        if event == "working":
            return "working", None
        if event == "blocked":
            return "blocked", "permission"
        if event == "done":
            return "done", None
        if event == "error":
            return "failed", None
        return "idle", None
    return None, None


def _session_id(payload: Mapping[str, Any]) -> str | None:
    value = _first(
        payload,
        (
            "session_id",
            "sessionId",
            "sessionID",
            "thread_id",
            "threadId",
            "conversation_id",
            "conversationId",
            "id",
        ),
    )
    if isinstance(value, str) and value.strip():
        return value.strip()[:MAX_SESSION]
    return None


def _child_info(
    payload: Mapping[str, Any], event: str, tool: str | None, text: str
) -> dict[str, Any] | None:
    agent_id = _first_str(payload, ("agent_id", "agentId", "parent_agent_id", "parentAgentId"))
    agent_type = _first_str(payload, ("agent_type", "agentType", "subagent_type", "subagentType"))
    if event not in {"subagent_start", "subagent_stop"} and not agent_id and not agent_type:
        return None
    child_id = agent_id or agent_type or _session_id(payload) or "child"
    if event == "subagent_stop":
        state = "done"
    elif event == "subagent_start":
        state = "working"
    else:
        state = "working"
    timestamp = now_ms()
    return {
        "id": child_id[:MAX_SESSION],
        "type": (agent_type or "")[:MAX_MODEL],
        "state": state,
        "text": text,
        "since_ms": timestamp,
        "ts_ms": timestamp,
    }


def _children_background(payload: Mapping[str, Any]) -> int:
    total = 0
    for key in ("background_tasks", "backgroundTasks", "session_crons", "sessionCrons"):
        value = payload.get(key)
        if isinstance(value, list):
            total += len(value)
    return total


def normalize_event(
    agent: str,
    app: str | None,
    raw_event: str | None,
    payload: Mapping[str, Any] | None,
    env: Mapping[str, str] | None = None,
    existing: Mapping[str, Any] | None = None,
) -> dict[str, Any] | None:
    payload = payload if isinstance(payload, Mapping) else {}
    env = env or os.environ
    agent = canonical_agent(agent, env)
    app = app or display_app(agent)
    raw = raw_event or _first_str(
        payload, ("hook_event_name", "hookEventName", "hook_type", "hookType", "event", "type")
    )
    event = canonical_event(raw)
    if not event:
        return None
    state, kind = event_state(event, payload, existing)
    if state is None:
        return None

    tool = _first_str(payload, ("tool_name", "toolName", "name", "tool"))
    tool_input = _first(
        payload,
        ("tool_input", "toolInput", "toolArgs", "input", "arguments", "args"),
    )
    tool_output = _first(
        payload,
        (
            "tool_response",
            "toolResponse",
            "tool_output",
            "toolOutput",
            "tool_result",
            "toolResult",
            "output",
            "result",
        ),
    )
    prompt = _first_str(
        payload,
        (
            "prompt",
            "user_prompt",
            "userPrompt",
            "initial_prompt",
            "initialPrompt",
            "user_message",
            "message",
        ),
    )
    reply = _first_str(
        payload,
        (
            "last_assistant_message",
            "lastAssistantMessage",
            "prompt_response",
            "last_message",
            "assistant_message",
            "assistant_response",
            "text",
        ),
    )
    error = _first_str(
        payload,
        (
            "error",
            "error_details",
            "errorDetails",
            "error_message",
            "errorMessage",
            "reason",
            "message",
        ),
    )

    preview = build_preview(tool, tool_input)
    text = preview
    if event == "permission":
        text = f"approve {preview}" if preview else (_one_line(error) or "needs approval")
    elif event == "elicitation":
        text = f"answer: {_one_line(prompt or error) or 'needs an answer'}"
    elif event == "prompt":
        text = f"prompt: {_one_line(prompt) or 'working'}"
    elif event == "tool_failure":
        text = f"tool failed: {_one_line(error or preview) or 'failed'}"
    elif event == "response":
        text = _one_line(reply, MAX_TEXT, keep_tail=True) or "responding"
    elif event == "stop":
        text = _one_line(reply, MAX_TEXT, keep_tail=True) or "turn complete"
    elif event == "stop_failure":
        text = f"error: {_one_line(error) or 'turn failed'}"
    elif event == "session_start":
        text = "session started"
    elif event == "session_end":
        text = "session ended"
    elif event == "compact":
        text = "compacting context"
    elif event == "notification":
        text = _one_line(
            _first_str(payload, ("message", "body", "text", "title")) or "notification"
        )
    elif event == "subagent_start":
        text = f"subagent: {_one_line(_first_str(payload, ('description', 'prompt')) or tool or 'started')}"
    elif event == "subagent_stop":
        text = f"subagent done: {_one_line(_first_str(payload, ('description', 'prompt')) or tool or 'finished')}"
    elif event == "idle":
        text = "idle"
    if not text:
        text = event.replace("_", " ")

    child = _child_info(payload, event, tool, text)
    return {
        "v": VERSION,
        "agent": agent,
        "app": app,
        "event": event,
        "raw_event": _one_line(raw or "", MAX_MODEL),
        "state": state,
        "kind": kind,
        "text": _one_line(text, MAX_TEXT),
        "prompt": _field(prompt, MAX_FIELD),
        "tool": (tool or "")[:MAX_MODEL],
        "tool_input": _field(_tool_text(tool_input), MAX_FIELD),
        "tool_output": _field(tool_output, MAX_OUTPUT, keep_tail=True),
        "reply": _field(reply, MAX_FIELD, keep_tail=True),
        "model": _field(_first_str(payload, ("model", "model_name", "modelName")), MAX_MODEL),
        "session_id": _session_id(payload),
        "cwd": _field(_first_str(payload, ("cwd", "working_directory", "workingDirectory", "workspaceRoot")), MAX_CWD),
        "background": _children_background(payload),
        "child": child,
        "ts_ms": now_ms(),
        "seq": time.time_ns(),
    }


# ---------------------------------------------------------------------------
# Merge + store
# ---------------------------------------------------------------------------


def _empty_record() -> dict[str, Any]:
    return {
        "v": VERSION,
        "agent": "",
        "app": "",
        "state": "idle",
        "kind": None,
        "event": "",
        "raw_event": "",
        "state_since_ms": 0,
        "turn_started_ms": 0,
        "ts_ms": 0,
        "text": "",
        "prompt": None,
        "tool": "",
        "tool_input": None,
        "tool_output": None,
        "reply": None,
        "model": None,
        "session_id": None,
        "cwd": None,
        "background": 0,
        "children": [],
        "last_event": None,
    }


def merge_event(
    existing: Mapping[str, Any] | None, event: Mapping[str, Any], now: int | None = None
) -> dict[str, Any]:
    now = now_ms() if now is None else now
    record = dict(existing) if existing else _empty_record()
    record.setdefault("children", [])

    last = record.get("last_event")
    if isinstance(last, Mapping):
        same = (
            last.get("event") == event.get("event")
            and last.get("agent") == event.get("agent")
            and abs(now - int(last.get("ts_ms") or 0)) <= DEDUPE_MS
        )
        if same:
            return record

    child = event.get("child")
    if isinstance(child, Mapping):
        children = [item for item in record.get("children", []) if isinstance(item, Mapping)]
        child_id = child.get("id")
        replaced = False
        for index, item in enumerate(children):
            if item.get("id") == child_id:
                merged = {**item, **child}
                if item.get("state") == child.get("state"):
                    merged["since_ms"] = item.get("since_ms") or child.get("since_ms")
                children[index] = merged
                replaced = True
                break
        if not replaced:
            children.append(dict(child))
        record["children"] = children[-MAX_CHILDREN:]
        record["ts_ms"] = now
        record["seq"] = event.get("seq", now)
        record["last_event"] = {
            "event": event.get("event"),
            "agent": event.get("agent"),
            "ts_ms": now,
        }
        if not record.get("agent"):
            record["agent"] = event.get("agent") or ""
            record["app"] = event.get("app") or ""
        return record

    state = event.get("state") or record.get("state") or "idle"
    kind = event.get("kind")
    previous_state = record.get("state")
    # An auth block survives ordinary tool/turn events; only a fresh turn or a
    # session boundary clears it.
    if record.get("kind") == "auth" and state not in {"idle"} and event.get("event") not in {
        "session_start",
        "session_end",
        "prompt",
    }:
        state = "blocked"
        kind = "auth"
    if state == "working":
        kind = None

    if previous_state != state or event.get("event") == "session_start":
        record["state_since_ms"] = now
    if event.get("event") in {"prompt", "session_start"}:
        record["turn_started_ms"] = now
    if event.get("event") in {"session_start", "session_end"}:
        record["children"] = []
        record["reply"] = None
    if event.get("event") == "prompt":
        record["reply"] = None

    record["state"] = state
    record["kind"] = kind
    record["agent"] = event.get("agent") or record.get("agent") or ""
    record["app"] = event.get("app") or record.get("app") or ""
    record["event"] = event.get("event") or record.get("event") or ""
    record["raw_event"] = event.get("raw_event") or record.get("raw_event") or ""
    record["ts_ms"] = now
    record["seq"] = event.get("seq", now)
    record["text"] = event.get("text") or record.get("text") or ""
    if event.get("prompt") is not None:
        record["prompt"] = event.get("prompt")
    if event.get("tool") is not None:
        record["tool"] = event.get("tool")
    if event.get("tool_input") is not None:
        record["tool_input"] = event.get("tool_input")
    if event.get("tool_output") is not None:
        record["tool_output"] = event.get("tool_output")
    if event.get("reply") is not None:
        record["reply"] = event.get("reply")
    if event.get("model") is not None:
        record["model"] = event.get("model")
    if event.get("session_id") is not None:
        record["session_id"] = event.get("session_id")
    if event.get("cwd") is not None:
        record["cwd"] = event.get("cwd")
    if event.get("background") is not None:
        record["background"] = event.get("background")
    record["last_event"] = {
        "event": event.get("event"),
        "agent": event.get("agent"),
        "ts_ms": now,
    }
    return record


def _record_is_stale(record: Mapping[str, Any], now: int) -> bool:
    state = record.get("state")
    kind = record.get("kind")
    ts = int(record.get("ts_ms") or 0)
    since = int(record.get("state_since_ms") or ts)
    if state in {"working", "idle"}:
        return now - ts > WORKING_STALE_MS
    if state == "done":
        return now - since > DONE_TTL_MS
    if state == "failed":
        return now - since > FAILED_TTL_MS
    if state == "blocked":
        return now - since > (AUTH_TTL_MS if kind == "auth" else BLOCK_TTL_MS)
    return now - ts > WORKING_STALE_MS


def prune_store(store: Mapping[str, Any], now: int | None = None) -> dict[str, Any]:
    now = now_ms() if now is None else now
    panes = store.get("panes") if isinstance(store, Mapping) else None
    if not isinstance(panes, Mapping):
        return {"v": VERSION, "updated_ms": now, "panes": {}}
    cleaned: dict[str, dict[str, Any]] = {}
    for session, session_panes in panes.items():
        if not isinstance(session_panes, Mapping):
            continue
        kept: dict[str, Any] = {}
        for pane, record in session_panes.items():
            if isinstance(record, Mapping) and not _record_is_stale(record, now):
                kept[str(pane)] = dict(record)
        if kept:
            cleaned[str(session)] = kept
    return {"v": VERSION, "updated_ms": now, "panes": cleaned}


def load_store() -> dict[str, Any]:
    data = _read_json(store_path())
    if not isinstance(data, Mapping) or not isinstance(data.get("panes"), Mapping):
        return {"v": VERSION, "updated_ms": now_ms(), "panes": {}}
    return {
        "v": VERSION,
        "updated_ms": int(data.get("updated_ms") or 0),
        "panes": {
            str(session): {
                str(pane): dict(record)
                for pane, record in panes.items()
                if isinstance(record, Mapping)
            }
            for session, panes in data["panes"].items()
            if isinstance(panes, Mapping)
        },
    }


def _store_lock():
    ensure_dir()
    handle = open(store_lock_path(), "a+")
    fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
    return handle


def _unlock(handle) -> None:
    try:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
    finally:
        handle.close()


def write_store(store: Mapping[str, Any]) -> None:
    payload = json.dumps(store, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    _atomic_write(store_path(), payload.encode("utf-8"), 0o600)


def merge_into_store(session: str, pane: Any, event: Mapping[str, Any]) -> None:
    now = now_ms()
    handle = _store_lock()
    try:
        store = load_store()
        panes = store.setdefault("panes", {}).setdefault(str(session), {})
        previous = panes.get(str(pane))
        panes[str(pane)] = merge_event(previous, event, now)
        write_store(prune_store(store, now))
    finally:
        _unlock(handle)


def apply_event_to_copy(
    store: dict[str, Any], entry: Mapping[str, Any], now: int | None = None
) -> None:
    session = entry.get("session")
    pane = entry.get("pane")
    if session is None or pane is None:
        return
    event = {key: value for key, value in entry.items() if key not in {"session", "pane"}}
    panes = store.setdefault("panes", {}).setdefault(str(session), {})
    panes[str(pane)] = merge_event(panes.get(str(pane)), event, now)


# ---------------------------------------------------------------------------
# Spool
# ---------------------------------------------------------------------------


def _spool_line(entry: Mapping[str, Any]) -> bytes:
    payload = json.dumps(entry, ensure_ascii=False, separators=(",", ":"))
    data = (payload + "\n").encode("utf-8")
    if len(data) <= 24 * 1024:
        return data
    trimmed = dict(entry)
    for key, limit in (
        ("prompt", 1024),
        ("tool_input", 1024),
        ("tool_output", 512),
        ("reply", 1024),
        ("text", MAX_TEXT),
    ):
        if isinstance(trimmed.get(key), str):
            trimmed[key] = _one_line(trimmed[key], limit, keep_tail=key == "reply")
    payload = json.dumps(trimmed, ensure_ascii=False, separators=(",", ":"))
    return (payload[: 24 * 1024] + "\n").encode("utf-8")


def append_spool(entry: Mapping[str, Any]) -> None:
    if disabled():
        return
    ensure_dir()
    path = spool_path()
    try:
        fd = os.open(path, os.O_APPEND | os.O_CREAT | os.O_WRONLY, 0o600)
    except OSError:
        return
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        os.write(fd, _spool_line(entry))
        fcntl.flock(fd, fcntl.LOCK_UN)
    except OSError:
        pass
    finally:
        os.close(fd)
    try:
        if path.stat().st_size > MAX_SPOOL_BYTES:
            _trim_spool(path)
    except OSError:
        pass


def _trim_spool(path: Path) -> None:
    try:
        handle = open(path, "r+", encoding="utf-8")
    except OSError:
        return
    try:
        fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
        lines = handle.readlines()
        keep = lines[-MAX_SPOOL_LINES:]
        handle.seek(0)
        handle.truncate()
        handle.writelines(keep)
        handle.flush()
        os.fsync(handle.fileno())
    except OSError:
        pass
    finally:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        except OSError:
            pass
        handle.close()


def read_spool_events(limit: int = MAX_SPOOL_LINES) -> list[dict[str, Any]]:
    path = spool_path()
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            lines = handle.readlines()
    except OSError:
        return []
    events: list[dict[str, Any]] = []
    cutoff = now_ms() - SPOOL_TTL_MS
    for line in lines[-limit:]:
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
        except ValueError:
            continue
        if not isinstance(entry, Mapping):
            continue
        ts = entry.get("ts_ms")
        if isinstance(ts, (int, float)) and ts < cutoff:
            continue
        events.append(dict(entry))
    return events


def drain_spool() -> int:
    path = spool_path()
    if not path.exists():
        return 0
    try:
        handle = open(path, "r+", encoding="utf-8")
    except OSError:
        return 0
    count = 0
    store_lock = None
    try:
        fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
        lines = handle.readlines()
        store_lock = _store_lock()
        store = load_store()
        cutoff = now_ms() - SPOOL_TTL_MS
        for line in lines:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if not isinstance(entry, Mapping):
                continue
            ts = entry.get("ts_ms")
            if isinstance(ts, (int, float)) and ts < cutoff:
                continue
            apply_event_to_copy(store, entry)
            count += 1
        write_store(prune_store(store, now_ms()))
        handle.seek(0)
        handle.truncate()
        handle.flush()
        os.fsync(handle.fileno())
    except OSError:
        pass
    finally:
        if store_lock is not None:
            _unlock(store_lock)
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        except OSError:
            pass
        handle.close()
    return count


# ---------------------------------------------------------------------------
# Summaries for the plugin
# ---------------------------------------------------------------------------


def _summary(record: Mapping[str, Any]) -> dict[str, Any]:
    children = record.get("children")
    child_count = len(children) if isinstance(children, list) else 0
    has_detail = bool(
        record.get("prompt")
        or record.get("tool_input")
        or record.get("tool_output")
        or record.get("reply")
        or child_count
    )
    return {
        "agent": record.get("agent") or "",
        "app": record.get("app") or "",
        "state": record.get("state") or "idle",
        "kind": record.get("kind"),
        "event": record.get("event") or "",
        "ts_ms": int(record.get("ts_ms") or 0),
        "state_since_ms": int(record.get("state_since_ms") or 0),
        "turn_started_ms": int(record.get("turn_started_ms") or 0),
        "text": record.get("text") or "",
        "model": record.get("model"),
        "session_id": record.get("session_id"),
        "cwd": record.get("cwd"),
        "child_count": child_count,
        "has_detail": has_detail,
    }


def _merged_view() -> tuple[dict[str, Any], int]:
    store = load_store()
    events = read_spool_events()
    now = now_ms()
    for entry in events:
        apply_event_to_copy(store, entry, now)
    return prune_store(store, now), len(events)


def space_hooks_summary() -> dict[str, Any]:
    store, pending = _merged_view()
    panes: dict[str, dict[str, Any]] = {}
    for session, session_panes in store.get("panes", {}).items():
        panes[session] = {
            str(pane): _summary(record) for pane, record in session_panes.items()
        }
    return {
        "ok": True,
        "version": VERSION,
        "now_ms": now_ms(),
        "spool_pending": pending,
        "panes": panes,
    }


def space_hooks_detail(session: str, pane: Any) -> dict[str, Any]:
    store, _ = _merged_view()
    record = store.get("panes", {}).get(str(session), {}).get(str(pane))
    if not isinstance(record, Mapping):
        return {"ok": True, "version": VERSION, "record": None}
    detail = dict(record)
    tool_name = detail.get("tool")
    if (isinstance(tool_name, str) and tool_name) or detail.get("tool_input"):
        detail["tool"] = {
            "name": tool_name if isinstance(tool_name, str) else "",
            "input": detail.get("tool_input") or "",
            "output": detail.get("tool_output"),
        }
    else:
        detail.pop("tool", None)
    detail["child_count"] = len(detail.get("children") or [])
    detail["has_detail"] = bool(
        detail.get("prompt")
        or detail.get("tool_input")
        or detail.get("tool_output")
        or detail.get("reply")
        or detail.get("children")
    )
    return {"ok": True, "version": VERSION, "record": detail}


# ---------------------------------------------------------------------------
# Endpoint + client
# ---------------------------------------------------------------------------


def read_endpoint() -> dict[str, Any] | None:
    data = _read_json(endpoint_path())
    if not isinstance(data, Mapping):
        return None
    if data.get("version") != VERSION:
        return None
    return dict(data)


def _socket_ready(path: str, timeout: float = 0.1) -> bool:
    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(timeout)
        sock.connect(path)
        sock.close()
        return True
    except OSError:
        return False


class _UnixHTTPConnection(http.client.HTTPConnection):
    def __init__(self, path: str, timeout: float = 0.2):
        super().__init__("localhost", timeout=timeout)
        self.unix_path = path

    def connect(self) -> None:
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.connect(self.unix_path)


def publish(envelope: Mapping[str, Any]) -> bool:
    if disabled():
        return False
    endpoint = read_endpoint()
    if not endpoint:
        return False
    body = json.dumps(envelope, ensure_ascii=False, separators=(",", ":"))
    headers = {
        "Content-Type": "application/json",
        "X-ZJ-Token": str(endpoint.get("token") or ""),
        "Connection": "close",
    }
    connections: list[http.client.HTTPConnection] = []
    socket_file = endpoint.get("socket")
    if isinstance(socket_file, str) and socket_file:
        connections.append(_UnixHTTPConnection(socket_file, timeout=0.2))
    port = endpoint.get("tcp_port")
    if isinstance(port, int) and port > 0:
        connections.append(http.client.HTTPConnection("127.0.0.1", port, timeout=0.2))
    for connection in connections:
        try:
            connection.request("POST", "/v1/hook", body, headers)
            response = connection.getresponse()
            response.read()
            if 200 <= int(response.status) < 300:
                return True
        except (OSError, http.client.HTTPException):
            pass
        finally:
            try:
                connection.close()
            except OSError:
                pass
    return False


def ensure_server(helper_path: str) -> None:
    """Start the listener without waiting, at most once every 10 seconds."""
    if disabled() or os.environ.get("ZJ_HOOKS_NO_DAEMON", "").lower() in {"1", "true", "yes"}:
        return
    endpoint = read_endpoint()
    if endpoint and isinstance(endpoint.get("socket"), str):
        if _socket_ready(endpoint["socket"]):
            return
    ensure_dir()
    stamp = startstamp_path()
    try:
        if time.time() - stamp.stat().st_mtime < START_THROTTLE_S:
            return
    except OSError:
        pass
    try:
        stamp.touch()
    except OSError:
        pass
    try:
        process = subprocess.Popen(
            [sys.executable, helper_path, "hook-server", "--daemon"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
            close_fds=True,
        )
        _log(f"spawned hook server pid={process.pid}")
    except OSError as error:
        _log(f"could not start hook server: {error}")


# ---------------------------------------------------------------------------
# Listener
# ---------------------------------------------------------------------------


class _HookHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def address_string(self) -> str:  # noqa: D401 - stdlib hook
        try:
            return str(self.client_address[0])
        except (IndexError, TypeError):
            return "unix"

    def log_message(self, fmt: str, *args: Any) -> None:  # noqa: D401
        _log(fmt % args)

    def _send(self, status: int, payload: Mapping[str, Any] | None = None) -> None:
        body = (
            json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
            if payload is not None
            else b""
        )
        reason = {
            200: "OK",
            204: "No Content",
            400: "Bad Request",
            401: "Unauthorized",
            404: "Not Found",
            411: "Length Required",
            413: "Payload Too Large",
        }.get(status, "Error")
        self.wfile.write(f"HTTP/1.1 {status} {reason}\r\n".encode("ascii"))
        if body:
            self.wfile.write(b"Content-Type: application/json\r\n")
            self.wfile.write(f"Content-Length: {len(body)}\r\n".encode("ascii"))
        else:
            self.wfile.write(b"Content-Length: 0\r\n")
        self.wfile.write(b"Connection: close\r\n\r\n")
        if body:
            self.wfile.write(body)
        self.wfile.flush()

    def _authorized(self) -> bool:
        server = self.server  # type: ignore[attr-defined]
        token = getattr(server, "token", "")
        provided = self.headers.get("X-ZJ-Token", "")
        if provided != token:
            return False
        # On Linux, also require the peer to be the same user for UDS.
        if getattr(self.connection, "family", None) == socket.AF_UNIX:
            try:
                raw = self.connection.getsockopt(
                    socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize("3i")
                )
                _pid, uid, _gid = struct.unpack("3i", raw)
                if uid != os.getuid():
                    return False
            except (AttributeError, OSError):
                pass
        return True

    def do_GET(self) -> None:  # noqa: N802 - stdlib hook
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path in {"/health", "/v1/health"}:
            self._send(200, {"ok": True, "version": VERSION})
        else:
            self._send(404, {"ok": False})

    def do_POST(self) -> None:  # noqa: N802 - stdlib hook
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path != "/v1/hook" and not parsed.path.startswith("/v1/hook/"):
            self._send(404, {"ok": False})
            return
        if not self._authorized():
            self._send(401, {"ok": False})
            return
        length_header = self.headers.get("Content-Length")
        if length_header is None:
            self._send(411, {"ok": False})
            return
        try:
            length = int(length_header)
        except ValueError:
            self._send(400, {"ok": False})
            return
        if length < 0 or length > MAX_BODY:
            self._send(413, {"ok": False})
            return
        try:
            raw = self.rfile.read(length)
            envelope = json.loads(raw.decode("utf-8"))
        except (ValueError, UnicodeDecodeError):
            self._send(400, {"ok": False})
            return
        if not isinstance(envelope, Mapping):
            self._send(400, {"ok": False})
            return
        path_agent = (
            urllib.parse.unquote(parsed.path[len("/v1/hook/") :])
            if parsed.path.startswith("/v1/hook/")
            else None
        )
        try:
            accepted = _accept_envelope(envelope, parsed, path_agent, self.headers)
        except Exception as error:  # never let a bad payload kill the listener
            _log(f"hook handler error: {error}")
            accepted = False
        if accepted:
            self._send(204)
        else:
            self._send(400, {"ok": False})


def _query_value(parsed: urllib.parse.ParseResult, name: str) -> str | None:
    values = urllib.parse.parse_qs(parsed.query).get(name)
    return values[0] if values else None


def _accept_envelope(
    envelope: Mapping[str, Any],
    parsed: urllib.parse.ParseResult,
    path_agent: str | None = None,
    headers: Any = None,
) -> bool:
    session = (
        envelope.get("session")
        or (headers.get("X-ZJ-Session") if headers is not None else None)
        or _query_value(parsed, "session")
    )
    pane = envelope.get("pane")
    if pane is None and headers is not None:
        pane = headers.get("X-ZJ-Pane")
    if pane is None:
        pane = _query_value(parsed, "pane")
    if session is None or pane is None:
        return False
    normalized = envelope.get("normalized")
    agent = path_agent or envelope.get("agent") or "unknown"
    if isinstance(normalized, Mapping):
        event = dict(normalized)
    else:
        payload = envelope.get("payload", envelope)
        if not isinstance(payload, Mapping):
            return False
        raw_event = envelope.get("event")
        event = normalize_event(
            str(agent),
            None,
            raw_event if isinstance(raw_event, str) else None,
            payload,
            os.environ,
        )
        if event is None:
            return True  # acknowledged but ignored
    merge_into_store(str(session), pane, event)
    return True


class _ThreadingUnixHTTPServer(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
    daemon_threads = True
    allow_reuse_address = False


class _ThreadingTCPServer(socketserver.ThreadingMixIn, socketserver.TCPServer):
    daemon_threads = True
    allow_reuse_address = True


def _remove_stale(path: Path) -> None:
    endpoint = read_endpoint()
    if endpoint and isinstance(endpoint.get("pid"), int):
        try:
            os.kill(endpoint["pid"], 0)
        except OSError as error:
            if error.errno != errno.ESRCH:
                return
        else:
            return
    try:
        if path.exists() or path.is_socket():
            path.unlink()
    except OSError:
        pass
    try:
        endpoint_path().unlink()
    except OSError:
        pass


def _write_endpoint(token: str, tcp_port: int | None) -> None:
    endpoint = {
        "version": VERSION,
        "protocol": "http/uds",
        "pid": os.getpid(),
        "uid": os.getuid(),
        "socket": str(socket_path()),
        "tcp_port": tcp_port,
        "token": token,
        "started_ms": now_ms(),
    }
    _atomic_write(
        endpoint_path(),
        json.dumps(endpoint, ensure_ascii=False, separators=(",", ":")).encode("utf-8"),
        0o600,
    )


def serve(foreground: bool = True, tcp_port: int | None = None) -> int:
    ensure_dir()
    lock = open(lock_path(), "a+")
    try:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        lock.close()
        return 0
    token = secrets.token_hex(16)
    _remove_stale(socket_path())
    servers: list[socketserver.BaseServer] = []
    unix_server = _ThreadingUnixHTTPServer(str(socket_path()), _HookHandler)
    unix_server.token = token  # type: ignore[attr-defined]
    try:
        os.chmod(socket_path(), 0o600)
    except OSError:
        pass
    servers.append(unix_server)
    bound_port: int | None = None
    if tcp_port:
        try:
            tcp_server = _ThreadingTCPServer(("127.0.0.1", tcp_port), _HookHandler)
            tcp_server.token = token  # type: ignore[attr-defined]
            bound_port = int(tcp_server.server_address[1])
            servers.append(tcp_server)
        except OSError as error:
            _log(f"could not bind TCP listener: {error}")
    _write_endpoint(token, bound_port)
    stop = threading.Event()

    def _stop(_signum=None, _frame=None):
        stop.set()

    previous_handlers = {}
    for sig in (signal.SIGTERM, signal.SIGINT):
        try:
            previous_handlers[sig] = signal.signal(sig, _stop)
        except (ValueError, OSError):
            pass

    drain_spool()

    def _drain_loop():
        while not stop.wait(1.0):
            try:
                drain_spool()
            except Exception as error:
                _log(f"spool drain error: {error}")

    threading.Thread(target=_drain_loop, daemon=True).start()
    threads = []
    for server in servers:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        threads.append(thread)
    if not foreground:
        _log(f"listener started pid={os.getpid()} socket={socket_path()}")
    try:
        while not stop.wait(0.5):
            pass
    finally:
        for server in servers:
            try:
                server.shutdown()
                server.server_close()
            except Exception:
                pass
        for sig, handler in previous_handlers.items():
            try:
                signal.signal(sig, handler)
            except (ValueError, OSError):
                pass
        try:
            socket_path().unlink()
        except OSError:
            pass
        try:
            endpoint_path().unlink()
        except OSError:
            pass
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
        except OSError:
            pass
        lock.close()
    return 0


def daemonize() -> int:
    ensure_dir()
    pid = os.fork()
    if pid > 0:
        return 0
    try:
        os.setsid()
    except OSError:
        # ensure_server already started us in a new session.
        pass
    pid = os.fork()
    if pid > 0:
        os._exit(0)
    os.chdir("/")
    try:
        log = open(log_path(), "a", encoding="utf-8")
    except OSError:
        log = open(os.devnull, "w", encoding="utf-8")
    devnull = open(os.devnull, "r", encoding="utf-8")
    os.dup2(devnull.fileno(), 0)
    os.dup2(log.fileno(), 1)
    os.dup2(log.fileno(), 2)
    result = 0
    try:
        result = serve(foreground=False, tcp_port=None)
    except BaseException:
        import traceback

        traceback.print_exc()
        result = 1
    finally:
        try:
            sys.stdout.flush()
            sys.stderr.flush()
        except Exception:
            pass
        os._exit(result)


# ---------------------------------------------------------------------------
# CLI helpers used by zj-resume.py
# ---------------------------------------------------------------------------


def hooks_status() -> int:
    endpoint = read_endpoint()
    if endpoint:
        pid = endpoint.get("pid")
        alive = False
        if isinstance(pid, int):
            try:
                os.kill(pid, 0)
                alive = True
            except OSError:
                alive = False
        socket_file = endpoint.get("socket")
        ready = isinstance(socket_file, str) and _socket_ready(socket_file)
        print(f"server:  {'running' if alive and ready else 'stale'} (pid {pid})")
        print(f"socket:  {socket_file}")
        port = endpoint.get("tcp_port")
        print(f"tcp:     {'127.0.0.1:' + str(port) if port else 'disabled'}")
    else:
        print("server:  not running")
    store = load_store()
    panes = sum(len(value) for value in store.get("panes", {}).values())
    print(f"store:   {panes} pane(s) ({store_path()})")
    spool = read_spool_events()
    print(f"spool:   {len(spool)} pending event(s) ({spool_path()})")
    print(f"runtime: {runtime_dir()}")
    if disabled():
        print("hooks:   disabled (ZJ_HOOKS_DISABLE)")
    return 0


def hooks_stop() -> int:
    endpoint = read_endpoint()
    if not endpoint:
        return 0
    pid = endpoint.get("pid")
    if isinstance(pid, int):
        try:
            os.kill(pid, signal.SIGTERM)
        except OSError:
            pass
        for _ in range(20):
            try:
                os.kill(pid, 0)
            except OSError:
                break
            time.sleep(0.05)
    for path in (endpoint_path(), socket_path()):
        try:
            path.unlink()
        except OSError:
            pass
    print("zj hooks: stopped")
    return 0


def hooks_start(helper_path: str) -> int:
    endpoint = read_endpoint()
    if endpoint and isinstance(endpoint.get("socket"), str) and _socket_ready(endpoint["socket"]):
        print("zj hooks: already running")
        return 0
    try:
        subprocess.run(
            [sys.executable, helper_path, "hook-server", "--daemon"],
            check=False,
            timeout=2,
        )
    except (OSError, subprocess.SubprocessError):
        pass
    for _ in range(40):
        endpoint = read_endpoint()
        if endpoint and isinstance(endpoint.get("socket"), str) and _socket_ready(endpoint["socket"]):
            print("zj hooks: started")
            return 0
        time.sleep(0.05)
    print("zj hooks: failed to start")
    return 1
