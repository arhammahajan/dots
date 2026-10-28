"""Exercise the fx bridge against a fake model-router, including SSE relay."""
import http.client
import importlib.util
import json
from pathlib import Path
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit
from uuid import uuid4

spec = importlib.util.spec_from_file_location("fx_proxy", Path(__file__).resolve().parents[1] / "router-proxy.py")
proxy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proxy)

STREAM = (b'data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1",'
          b'"type":"function","function":{"name":"read_file","arguments":"{}"}}]}}]}\n\n'
          b'data: [DONE]\n\n')


class BridgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.requests = []

        class Upstream(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_POST(self):
                cls.requests.append((self.path, dict(self.headers),
                                     json.loads(self.rfile.read(int(self.headers["Content-Length"])))))
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("x-model-router-loss", "example")
                self.end_headers()
                self.wfile.write(STREAM[:40])
                self.wfile.flush()
                self.wfile.write(STREAM[40:])

        cls.upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
        proxy.TARGET = urlsplit(f"http://127.0.0.1:{cls.upstream.server_port}")
        cls.bridge = proxy.Server(("127.0.0.1", 0), proxy.Proxy)
        for server in (cls.upstream, cls.bridge):
            threading.Thread(target=server.serve_forever, daemon=True).start()

    @classmethod
    def tearDownClass(cls):
        for server in (cls.bridge, cls.upstream):
            server.shutdown()
            server.server_close()

    def request(self, session, messages, path="/v1/chat/completions", effort=None):
        connection = http.client.HTTPConnection("127.0.0.1", self.bridge.server_port, timeout=5)
        try:
            payload = {"model": "local-llama", "stream": True, "messages": messages}
            if effort is not None:
                payload["reasoning_effort"] = effort
            connection.request("POST", path, json.dumps(payload),
                               {"Authorization": "Bearer " + session, "Content-Type": "application/json"})
            response = connection.getresponse()
            return response.status, dict(response.getheaders()), response.read()
        finally:
            connection.close()

    def test_stream_tools_and_instruction_order_survive_forwarding(self):
        session = "fx-" + str(uuid4())
        tail = [{"role": "user", "content": "read it"},
                {"role": "assistant", "tool_calls": [{"id": "call_1"}]},
                {"role": "tool", "tool_call_id": "call_1", "content": "file contents"}]
        status, headers, body = self.request(session, [
            {"role": "system", "content": "base instructions"},
            {"role": "system", "content": "workspace instructions"}, *tail])
        self.assertEqual(status, 200)
        self.assertEqual(body, STREAM)
        self.assertEqual(headers["x-model-router-loss"], "example")
        path, forwarded, payload = self.requests[-1]
        self.assertEqual(path, "/v1/chat/completions")
        self.assertEqual(forwarded["x-session-id"], session)
        self.assertEqual(forwarded["Authorization"], "Bearer unused")
        self.assertEqual(payload["messages"], [
            {"role": "system", "content": "base instructions\n\nworkspace instructions"}, *tail])

    def test_affinity_is_stable_within_a_launch_and_separate_between_launches(self):
        first, second = ["fx-" + str(uuid4()) for _ in range(2)]
        for session in (first, first, second):
            self.assertEqual(self.request(session, [{"role": "user", "content": "hello"}])[0], 200)
        self.assertEqual([r[1]["x-session-id"] for r in self.requests[-3:]], [first, first, second])

    def test_later_system_messages_keep_their_position(self):
        messages = [{"role": "system", "content": "first"},
                    {"role": "user", "content": "hello"},
                    {"role": "system", "content": "later"}]
        self.request("fx-" + str(uuid4()), messages)
        self.assertEqual(self.requests[-1][2]["messages"], messages)

    def test_invalid_affinity_and_unrelated_paths_never_reach_router(self):
        before = len(self.requests)
        self.assertEqual(self.request("unused", [])[0], 401)
        self.assertEqual(self.request("fx-" + str(uuid4()), [], path="/other")[0], 404)
        self.assertEqual(len(self.requests), before)

    def test_explicit_effort_reaches_router_without_remapping(self):
        for effort in ("none", "minimal", "high", "max"):
            status, _, _ = self.request("fx-" + str(uuid4()), [
                {"role": "system", "content": "one"},
                {"role": "system", "content": "two"},
                {"role": "user", "content": "hello"}], effort=effort)
            self.assertEqual(status, 200)
            self.assertEqual(self.requests[-1][2]["reasoning_effort"], effort)


if __name__ == "__main__":
    unittest.main()
