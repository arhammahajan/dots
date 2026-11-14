"""Run installed Gemini ACP against an offline HTTP fixture, never an upstream."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import threading
import time
import unittest


ROOT = Path(__file__).resolve().parents[1]
GEMINI_REPO = Path(os.environ.get("GEMINI_CLI_REPO", Path.home() / "dev/gemini-cli"))


@unittest.skipUnless(shutil.which("node") and (GEMINI_REPO / "packages/cli/dist/index.js").is_file(),
                     "Gemini CLI source build is not installed")
class GeminiAcpTest(unittest.TestCase):
    def test_t3_mode_effort_model_switch_and_context_usage(self):
        requests = []
        models = []
        for name, context, efforts in (
            ("deepseek-v4.1-flash", 500000, ["low", "high", "max"]),
            ("grok-4.7", 32000, ["low", "high"]),
        ):
            models.append({"id": name, "metadata": {"router": {
                "limits": {"context_window": {"value": context}},
                "reasoning": {"effort": {"values": {"value": efforts}}},
            }}})

        class Fixture(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def respond(self, body, *, streaming=False):
                wire = (("data: " + json.dumps(body) + "\n\n") if streaming else json.dumps(body)).encode()
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream" if streaming else "application/json")
                self.send_header("Content-Length", str(len(wire)))
                self.end_headers()
                self.wfile.write(wire)

            def do_GET(self):
                self.respond({"data": models})

            def do_POST(self):
                length = int(self.headers.get("Content-Length", "0"))
                body = json.loads(self.rfile.read(length))
                requests.append((self.path, dict(self.headers), body))
                if "countTokens" in self.path:
                    self.respond({"totalTokens": 17})
                    return
                model = self.path.split("/models/")[-1].split(":")[0]
                self.respond({
                    "candidates": [{"index": 0, "content": {"role": "model", "parts": [
                        {"text": "OFFLINE_ACP_OK"},
                    ]}, "finishReason": "STOP"}],
                    "usageMetadata": {"promptTokenCount": 17, "candidatesTokenCount": 2, "totalTokenCount": 19},
                    "modelVersion": model,
                }, streaming="streamGenerateContent" in self.path)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(lambda: (server.shutdown(), server.server_close(), thread.join()))
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp)
            (home / ".gemini").mkdir()
            settings = json.loads((ROOT / "gemini/settings.json").read_text())
            settings["mcpServers"] = {}
            (home / ".gemini/settings.json").write_text(json.dumps(settings))
            provider, = json.loads((ROOT / "gemini/t3-provider.json").read_text()).values()
            env = {**os.environ, **{v["name"]: v["value"] for v in provider["environment"]},
                   "HOME": str(home), "XDG_CONFIG_HOME": str(home / "config"),
                   "XDG_DATA_HOME": str(home / "data"),
                   "GEMINI_CLI_REPO": str(GEMINI_REPO),
                   "MODEL_ROUTER_URL": f"http://127.0.0.1:{server.server_port}"}
            with tempfile.TemporaryFile() as stderr:
                process = subprocess.Popen([str(ROOT / "harness"), "--acp"], cwd=home, env=env,
                                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr)
                selector = selectors.DefaultSelector()
                selector.register(process.stdout, selectors.EVENT_READ)
                pending = b""
                notifications = []

                def rpc(number, method, params, *, expected_error=None):
                    nonlocal pending
                    process.stdin.write((json.dumps({"jsonrpc": "2.0", "id": number,
                                                    "method": method, "params": params}) + "\n").encode())
                    process.stdin.flush()
                    deadline = time.monotonic() + 30
                    while time.monotonic() < deadline:
                        while b"\n" in pending:
                            line, pending = pending.split(b"\n", 1)
                            message = json.loads(line)
                            if message.get("id") == number:
                                if expected_error:
                                    self.assertEqual(message["error"]["code"], -32602)
                                    self.assertIn(expected_error, message["error"]["message"])
                                    return message["error"]
                                self.assertNotIn("error", message, message)
                                return message["result"]
                            notifications.append(message)
                        if not selector.select(1):
                            continue
                        chunk = os.read(process.stdout.fileno(), 65536)
                        if not chunk:
                            stderr.seek(0)
                            self.fail(stderr.read().decode())
                        pending += chunk
                    self.fail(f"ACP timed out: {method}")

                try:
                    rpc(1, "initialize", {"protocolVersion": 1, "clientCapabilities": {}})
                    session = rpc(2, "session/new", {"cwd": str(home), "mcpServers": []})
                    sid = session["sessionId"]
                    choices = session["models"]["availableModels"]
                    self.assertEqual(len(choices), 2)
                    self.assertEqual([m["name"] for m in choices], ["deepseek-v4.1-flash", "grok-4.7"])
                    self.assertTrue(all("@effort=" not in m["modelId"] for m in choices))
                    effort_option, = [o for o in session["configOptions"] if o["id"] == "reasoning_effort"]
                    # T3 propagates uncategorized options to every catalog model.
                    self.assertNotIn("category", effort_option)
                    self.assertEqual([o["value"] for o in effort_option["options"]],
                                     ["auto", "low", "high", "max"])
                    # T3's full-access policy failed at exactly this transition.
                    rpc(3, "session/set_mode", {"sessionId": sid, "modeId": "yolo"})
                    for number, choice, real_model, effort, context in (
                        (10, "deepseek-v4.1-flash", "deepseek-v4.1-flash", "max", 500000),
                        (20, "grok-4.7", "grok-4.7", "high", 32000),
                        (30, "deepseek-v4.1-flash", "deepseek-v4.1-flash", None, 500000),
                    ):
                        with self.subTest(choice=choice):
                            rpc(number, "session/set_config_option", {"sessionId": sid, "configId": "model", "value": choice})
                            if choice == "grok-4.7":
                                rpc(50, "session/set_config_option", {"sessionId": sid,
                                    "configId": "reasoning_effort", "value": "max"},
                                    expected_error="Supported efforts: auto, low, high")
                                request_count = len(requests)
                                # Even a client ignoring the set-option error cannot
                                # generate silently with the previous/default effort.
                                rpc(51, "session/prompt", {"sessionId": sid, "prompt": [
                                    {"type": "text", "text": "This must not generate."},
                                ]}, expected_error="Unsupported reasoning_effort=max")
                                self.assertEqual(len(requests), request_count)
                            rpc(number + 1, "session/set_config_option", {"sessionId": sid,
                                "configId": "reasoning_effort", "value": effort or "auto"})
                            notifications.clear()
                            result = rpc(number + 2, "session/prompt", {"sessionId": sid, "prompt": [
                                {"type": "text", "text": "Say hello. Do not call tools."},
                            ]})
                            self.assertEqual(result["stopReason"], "end_turn")
                            generation = [r for r in requests if "streamGenerateContent" in r[0]][-1]
                            self.assertIn(f"/models/{real_model}:", generation[0])
                            headers = {k.lower(): v for k, v in generation[1].items()}
                            self.assertEqual(headers.get("x-model-router-reasoning-effort"), effort)
                            updates = [m.get("params", {}).get("update", {}) for m in notifications]
                            usage = [u for u in updates if u.get("sessionUpdate") == "usage_update"]
                            self.assertEqual(usage[-1]["size"], context)
                finally:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                    process.stdin.close()
                    process.stdout.close()
                    selector.close()


if __name__ == "__main__":
    unittest.main()
