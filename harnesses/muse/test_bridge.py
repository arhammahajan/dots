import contextlib
import gzip
import importlib.util
import json
import tempfile
import threading
import unittest
from http.server import ThreadingHTTPServer
from pathlib import Path
from unittest.mock import Mock, patch

import httpx

spec = importlib.util.spec_from_file_location("bridge", Path(__file__).with_name("opencode-go-bridge.py"))
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class CatalogTests(unittest.TestCase):
    def test_bridge_options_do_not_become_upstream_request_fields(self):
        with patch.object(bridge.litellm, "responses") as call:
            bridge.translated_request({"model": "claude-test", "input": "test"}, "test", "session", "anthropic")
        self.assertNotIn("skip_mcp_handler", call.call_args.kwargs)
        self.assertTrue(call.call_args.kwargs["_skip_mcp_handler"])

    def test_inventory_keeps_models_missing_from_metadata_and_survives_outage(self):
        def get(url, **_kwargs):
            body = {"data": [{"id": "known"}, {"id": "new-model"}]}
            if url == "https://models.dev/api.json":
                body = {"opencode-go": {"models": {"known": {"limit": {"context": 1000000, "output": 64000}}}}}
            return httpx.Response(200, json=body, request=httpx.Request("GET", url))

        with tempfile.TemporaryDirectory() as directory:
            with patch.object(bridge, "CACHE", Path(directory) / "catalog.json"):
                with patch.object(bridge, "CLIENT", Mock(get=get)):
                    catalog = bridge.Catalog()
                    body = catalog.get()
                    self.assertEqual([m["id"] for m in body["data"]], ["known", "new-model"])
                    self.assertEqual(body["data"][0]["metadata"]["muse-code"]["limit"]["context"], 1000000)
                catalog.updated = 0
                with patch.object(bridge, "CLIENT", Mock(get=Mock(side_effect=httpx.ConnectError("offline")))):
                    self.assertEqual(catalog.get(), body)
                    self.assertEqual(bridge.Catalog().get(), body)


class HTTPTests(unittest.TestCase):
    def setUp(self):
        self.catalog_patch = patch.object(bridge, "CATALOG", Mock(get=Mock(return_value={"data": []})))
        self.catalog_patch.start()
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), bridge.Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.url = "http://127.0.0.1:" + str(self.server.server_port)

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()
        self.catalog_patch.stop()

    def test_native_catalog_path_with_query(self):
        with patch.object(bridge, "CATALOG", Mock(get=Mock(return_value={"data": [{"id": "all-models"}]}))):
            response = httpx.get(self.url + "/muse-code/models?client=muse")
        self.assertEqual(response.json()["data"][0]["id"], "all-models")

    def test_requests_without_credentials_are_rejected(self):
        response = httpx.post(self.url + "/v1/responses", json={"model": "model"})
        self.assertEqual(response.status_code, 401)

    def test_gemini_stream_has_monotonic_sequence_numbers(self):
        events = [{"type": "response.created"}, {"type": "response.output_text.delta", "delta": "OK"},
                  {"type": "response.completed", "sequence_number": 1}]
        with patch.object(bridge, "translated_request", Mock(return_value=iter(events))) as call:
            response = httpx.post(self.url + "/v1/responses", json={"model": "gemini-test", "stream": True},
                                  headers={"Authorization": "Bearer test", "x-opencode-session": "stable"})
        parsed = [json.loads(line[6:]) for line in response.text.splitlines() if line.startswith("data: ")]
        self.assertEqual([event["sequence_number"] for event in parsed], [1, 2, 3])
        self.assertEqual(call.call_args.args[1:], ("test", "stable", "gemini"))

    def test_native_responses_preserve_payload_and_use_cache_key_for_routing(self):
        payload = {"model": "glm-test", "prompt_cache_key": "conversation-1", "input": "test"}

        @contextlib.contextmanager
        def stream(method, url, **kwargs):
            self.assertEqual(method, "POST")
            self.assertEqual(url, bridge.UPSTREAM + "/openai/v1/responses")
            self.assertEqual(json.loads(kwargs["content"]), payload)
            self.assertEqual(kwargs["headers"]["x-opencode-session"], "conversation-1")
            yield httpx.Response(200, content=b'data: {"type":"response.completed"}\n\n',
                                 headers={"Content-Type": "text/event-stream"})

        with patch.object(bridge, "CLIENT", Mock(stream=stream)):
            response = httpx.post(self.url + "/v1/responses", content=gzip.compress(json.dumps(payload).encode()),
                                  headers={"Authorization": "Bearer test", "Content-Encoding": "gzip"})
        self.assertEqual(response.status_code, 200)
        self.assertIn('"response.completed"', response.text)

    def test_chat_only_model_falls_back_before_sending_response_headers(self):
        @contextlib.contextmanager
        def stream(*_args, **_kwargs):
            yield httpx.Response(400, json={"error": {"type": "ModelProtocolUnsupported"}})

        events = [{"type": "response.created"}, {"type": "response.completed"}]
        with patch.object(bridge, "CLIENT", Mock(stream=stream)):
            with patch.object(bridge, "PROTOCOL_OVERRIDES", {}) as overrides:
                with patch.object(bridge, "translated_request", Mock(return_value=iter(events))) as call:
                    response = httpx.post(self.url + "/v1/responses", json={"model": "chat-only", "stream": True},
                                          headers={"Authorization": "Bearer test"})
                self.assertEqual(overrides["chat-only"], "chat")
        self.assertEqual(response.status_code, 200)
        self.assertEqual(call.call_args.args[-1], "chat")
        self.assertIn('"response.completed"', response.text)


if __name__ == "__main__":
    unittest.main()
