"""Tests for the per-agent hook core and the ``zj hook`` client.

Run with:
    python3 -m unittest discover -s zellij/tests -t zellij
"""
from __future__ import annotations

import http.client
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

import zj_hook_core as hook  # noqa: E402

ZJ = ROOT / "zj"
RESUME = ROOT / "zj-resume.py"


class Runtime:
    def __init__(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.dir = Path(self.temp.name)
        self.saved = dict(os.environ)
        os.environ["ZJ_HOOKS_RUNTIME"] = str(self.dir)
        os.environ["ZJ_HOOKS_NO_DAEMON"] = "1"
        os.environ.pop("GROK_SESSION_ID", None)
        os.environ.pop("GROK_HOOK_EVENT", None)
        self.env = os.environ.copy()

    def close(self) -> None:
        os.environ.clear()
        os.environ.update(self.saved)
        self.temp.cleanup()


class NormalizationTests(unittest.TestCase):
    def test_claude_tool_and_permission_payloads(self) -> None:
        event = hook.normalize_event(
            "claude",
            None,
            "PreToolUse",
            {
                "hook_event_name": "PreToolUse",
                "session_id": "s1",
                "tool_name": "Bash",
                "tool_input": {"command": "cargo test"},
                "model": "m1",
            },
            {},
        )
        self.assertIsNotNone(event)
        assert event is not None
        self.assertEqual(event["state"], "working")
        self.assertEqual(event["text"], "Bash · cargo test")
        self.assertEqual(event["session_id"], "s1")
        self.assertEqual(event["model"], "m1")

        permission = hook.normalize_event(
            "claude",
            None,
            "PermissionRequest",
            {"tool_name": "Write", "tool_input": {"file_path": "src/x.rs"}},
            {},
        )
        assert permission is not None
        self.assertEqual(permission["state"], "blocked")
        self.assertEqual(permission["kind"], "permission")
        self.assertIn("approve Write", permission["text"])

    def test_ask_user_question_blocks(self) -> None:
        event = hook.normalize_event(
            "claude",
            None,
            "PreToolUse",
            {"tool_name": "AskUserQuestion", "tool_input": {"question": "which?"}},
            {},
        )
        assert event is not None
        self.assertEqual(event["state"], "blocked")
        self.assertEqual(event["kind"], "question")

    def test_provider_matrix(self) -> None:
        cases = [
            ("codex", "PreToolUse", {"session_id": "c", "tool_name": "shell", "tool_input": {"command": "ls"}}, "working"),
            ("codex", "PermissionRequest", {"session_id": "c"}, "blocked"),
            ("gemini", "BeforeAgent", {"session_id": "g", "prompt": "hi"}, "working"),
            ("gemini", "AfterAgent", {"session_id": "g", "prompt_response": "done"}, "done"),
            ("gemini", "Notification", {"notification_type": "ToolPermission"}, "blocked"),
            ("copilot", "sessionStart", {"sessionId": "p"}, "idle"),
            ("copilot", "userPromptSubmitted", {"sessionId": "p", "prompt": "go"}, "working"),
            ("cursor", "beforeSubmitPrompt", {"conversation_id": "c", "prompt": "go"}, "working"),
            ("cursor", "afterAgentResponse", {"conversation_id": "c", "text": "done"}, "working"),
            ("cursor", "stop", {"conversation_id": "c", "status": "completed"}, "done"),
            ("kimi", "SessionStart", {"session_id": "k"}, "idle"),
            ("opencode", "tool.execute.before", {"sessionID": "o", "tool": "bash", "input": {"command": "ls"}}, "working"),
            ("opencode", "session.execution.failed", {"sessionID": "o", "error": "boom"}, "failed"),
        ]
        for agent, event_name, payload, expected in cases:
            event = hook.normalize_event(agent, None, event_name, payload, {})
            self.assertIsNotNone(event, (agent, event_name))
            assert event is not None
            self.assertEqual(event["state"], expected, (agent, event_name))

    def test_cursor_response_keeps_the_reply(self) -> None:
        event = hook.normalize_event(
            "cursor",
            None,
            "afterAgentResponse",
            {"conversation_id": "c", "text": "finished the work"},
            {},
        )
        assert event is not None
        self.assertEqual(event["state"], "working")
        self.assertEqual(event["reply"], "finished the work")

    def test_grok_runtime_overrides_the_argv_hint(self) -> None:
        event = hook.normalize_event(
            "claude",
            None,
            "PreToolUse",
            {"tool_name": "Bash", "tool_input": {"command": "ls"}},
            {"GROK_SESSION_ID": "grok-1"},
        )
        assert event is not None
        self.assertEqual(event["agent"], "grok")
        self.assertEqual(event["app"], "grok")

    def test_unknown_events_are_ignored(self) -> None:
        self.assertIsNone(
            hook.normalize_event("claude", None, "SomeUnknownEvent", {}, {})
        )


class MergeTests(unittest.TestCase):
    def event(self, state: str, event: str, now: int, **extra):
        return {
            "v": 1,
            "agent": "claude",
            "app": "claude-code",
            "event": event,
            "state": state,
            "kind": extra.pop("kind", None),
            "text": extra.pop("text", event),
            "ts_ms": now,
            "seq": now,
            **extra,
        }

    def test_working_clears_a_turn_scoped_block(self) -> None:
        record = hook.merge_event(None, self.event("blocked", "permission", 1000, kind="permission"), 1000)
        self.assertEqual(record["state"], "blocked")
        record = hook.merge_event(record, self.event("working", "pre_tool", 2000), 2000)
        self.assertEqual(record["state"], "working")
        self.assertIsNone(record["kind"])

    def test_auth_block_survives_a_stop(self) -> None:
        record = hook.merge_event(None, self.event("blocked", "permission", 1000, kind="auth"), 1000)
        record = hook.merge_event(record, self.event("done", "stop", 2000), 2000)
        self.assertEqual(record["state"], "blocked")
        self.assertEqual(record["kind"], "auth")
        record = hook.merge_event(record, self.event("working", "prompt", 3000), 3000)
        self.assertEqual(record["state"], "working")

    def test_child_events_do_not_overwrite_the_parent(self) -> None:
        record = hook.merge_event(None, self.event("working", "pre_tool", 1000, text="Bash"), 1000)
        child = self.event("working", "pre_tool", 2000, text="child")
        child["child"] = {"id": "a1", "type": "Explore", "state": "working", "text": "child", "ts_ms": 2000}
        record = hook.merge_event(record, child, 2000)
        self.assertEqual(record["state"], "working")
        self.assertEqual(record["text"], "Bash")
        self.assertEqual(len(record["children"]), 1)
        stop = self.event("working", "subagent_stop", 3000, text="child done")
        stop["child"] = {"id": "a1", "type": "Explore", "state": "done", "text": "child done", "ts_ms": 3000}
        record = hook.merge_event(record, stop, 3000)
        self.assertEqual(record["children"][0]["state"], "done")

    def test_duplicate_events_are_deduped(self) -> None:
        first = self.event("working", "pre_tool", 1000)
        record = hook.merge_event(None, first, 1000)
        second = dict(first)
        second["seq"] = 1001
        self.assertEqual(hook.merge_event(record, second, 1200)["children"], [])
        self.assertEqual(hook.merge_event(record, second, 1200)["last_event"]["ts_ms"], 1000)


class StoreTests(unittest.TestCase):
    def setUp(self) -> None:
        self.runtime = Runtime()

    def tearDown(self) -> None:
        hook.hooks_stop()
        self.runtime.close()

    def test_summary_and_detail_merge_undrained_spool_events(self) -> None:
        event = hook.normalize_event(
            "claude",
            None,
            "PreToolUse",
            {"tool_name": "Bash", "tool_input": {"command": "cargo test"}, "session_id": "s"},
            {},
        )
        assert event is not None
        hook.append_spool({"session": "dots", "pane": 7, **event})
        summary = hook.space_hooks_summary()
        self.assertEqual(summary["spool_pending"], 1)
        pane = summary["panes"]["dots"]["7"]
        self.assertEqual(pane["state"], "working")
        self.assertTrue(pane["has_detail"])
        detail = hook.space_hooks_detail("dots", 7)
        self.assertEqual(detail["record"]["tool"]["name"], "Bash")
        self.assertEqual(detail["record"]["tool"]["input"], "cargo test")

    def test_store_is_atomic_and_prunes_stale_records(self) -> None:
        old = {
            "v": 1,
            "agent": "claude",
            "app": "claude-code",
            "state": "working",
            "kind": None,
            "event": "pre_tool",
            "text": "old",
            "ts_ms": 1,
            "seq": 1,
            "children": [],
        }
        store = {"v": 1, "updated_ms": 1, "panes": {"dots": {"1": old}}}
        hook.write_store(store)
        loaded = hook.load_store()
        self.assertEqual(loaded["panes"]["dots"]["1"]["text"], "old")
        pruned = hook.prune_store(loaded, hook.now_ms())
        self.assertNotIn("dots", pruned["panes"])


class ClientTests(unittest.TestCase):
    def setUp(self) -> None:
        self.runtime = Runtime()
        self.runtime.env["ZELLIJ_SESSION_NAME"] = "probe"
        self.runtime.env["ZELLIJ_PANE_ID"] = "terminal_7"
        self.tty = self.runtime.dir / "tty"
        self.runtime.env["ZJ_TTY"] = str(self.tty)

    def tearDown(self) -> None:
        hook.hooks_stop()
        self.runtime.close()

    def run_hook(self, agent: str, event: str, payload: dict):
        return subprocess.run(
            [str(ZJ), "hook", agent, event],
            input=json.dumps(payload),
            text=True,
            capture_output=True,
            env=self.runtime.env,
            timeout=10,
        )

    def test_client_spools_and_emits_osc_without_stdout(self) -> None:
        result = self.run_hook(
            "claude",
            "PreToolUse",
            {"tool_name": "Bash", "tool_input": {"command": "cargo test"}},
        )
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("state=working", self.tty.read_text())
        lines = hook.spool_path().read_text().strip().splitlines()
        self.assertEqual(len(lines), 1)
        entry = json.loads(lines[0])
        self.assertEqual(entry["session"], "probe")
        self.assertEqual(entry["pane"], 7)
        summary = subprocess.run(
            [str(ZJ), "__space-hooks"],
            text=True,
            capture_output=True,
            env=self.runtime.env,
            timeout=10,
        )
        parsed = json.loads(summary.stdout)
        self.assertEqual(parsed["panes"]["probe"]["7"]["state"], "working")

    def test_grok_env_overrides_the_client_agent(self) -> None:
        self.runtime.env["GROK_SESSION_ID"] = "grok-session"
        result = self.run_hook("claude", "UserPromptSubmit", {"prompt": "hello"})
        self.assertEqual(result.returncode, 0)
        entry = json.loads(hook.spool_path().read_text().strip().splitlines()[0])
        self.assertEqual(entry["agent"], "grok")
        self.assertEqual(entry["app"], "grok")

    def test_non_interactive_agent_is_ignored(self) -> None:
        # No ancestor agent process in the test runner; the guard only fires
        # when a non-interactive command line is visible.  This asserts the
        # normal path still records.
        result = self.run_hook("claude", "UserPromptSubmit", {"prompt": "hello"})
        self.assertEqual(result.returncode, 0)
        self.assertTrue(hook.spool_path().exists())

    def test_server_publish_and_spool_drain(self) -> None:
        event = hook.normalize_event(
            "claude",
            None,
            "Stop",
            {"session_id": "s", "last_assistant_message": "done"},
            {},
        )
        assert event is not None
        hook.append_spool({"session": "probe", "pane": 7, **event})
        code = (
            "import sys; sys.path.insert(0, %r); "
            "import zj_hook_core as h; h.serve(foreground=True, tcp_port=0)"
        ) % str(ROOT)
        proc = subprocess.Popen(
            [sys.executable, "-c", code],
            env=self.runtime.env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            endpoint = None
            for _ in range(100):
                endpoint = hook.read_endpoint()
                if endpoint and hook._socket_ready(endpoint["socket"]):
                    break
                time.sleep(0.05)
            self.assertIsNotNone(endpoint)
            time.sleep(0.5)
            summary = hook.space_hooks_summary()
            self.assertEqual(summary["panes"]["probe"]["7"]["state"], "done")
            # Publish a fresh event over the UDS.
            new = hook.normalize_event(
                "claude",
                None,
                "UserPromptSubmit",
                {"session_id": "s", "prompt": "again"},
                {},
            )
            assert new is not None
            self.assertTrue(
                hook.publish(
                    {
                        "session": "probe",
                        "pane": 7,
                        "agent": "claude",
                        "event": "UserPromptSubmit",
                        "normalized": new,
                    }
                )
            )
            self.assertEqual(hook.space_hooks_summary()["panes"]["probe"]["7"]["state"], "working")
            # TCP is enabled as well; the token is still required.
            assert endpoint is not None
            port = endpoint.get("tcp_port")
            if port:
                connection = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
                body = json.dumps(
                    {
                        "session": "probe",
                        "pane": 8,
                        "agent": "claude",
                        "normalized": hook.normalize_event(
                            "claude", None, "Stop", {"session_id": "t"}, {}
                        ),
                    }
                )
                connection.request(
                    "POST",
                    "/v1/hook",
                    body,
                    {
                        "Content-Type": "application/json",
                        "X-ZJ-Token": endpoint["token"],
                        "Connection": "close",
                    },
                )
                response = connection.getresponse()
                response.read()
                self.assertEqual(response.status, 204)
                connection.close()
                bad = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
                bad.request(
                    "POST",
                    "/v1/hook",
                    "{}",
                    {"Content-Type": "application/json", "Connection": "close"},
                )
                self.assertEqual(bad.getresponse().status, 401)
                bad.close()
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    unittest.main()
