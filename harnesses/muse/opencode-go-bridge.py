#!/usr/bin/env python3
"""Expose OpenCode Go's live inventory and API protocols through Muse's Responses API."""

import gzip
import json
import logging
import os
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

os.environ.setdefault("LITELLM_LOCAL_MODEL_COST_MAP", "True")
import httpx
import litellm

litellm.telemetry = False
litellm.suppress_debug_info = True
# Never log SDK requests, credentials, or prompt bodies to the service journal.
logging.disable(logging.CRITICAL)

UPSTREAM = "https://opencode.ai/inference/go"
USER_AGENT = "muse-code/1.4.13 opencode-go-bridge/1"
CACHE = Path.home() / ".cache/muse-opencode-go/catalog.json"
CLIENT = httpx.Client(timeout=httpx.Timeout(600, connect=20), follow_redirects=False)
PROTOCOL_OVERRIDES = {}


class Catalog:
    def __init__(self):
        self.lock = threading.Lock()
        self.updated = 0
        self.body = None
        try:
            self.body = json.loads(CACHE.read_text())
        except (OSError, ValueError):
            pass

    def get(self):
        with self.lock:
            if self.body is not None and time.monotonic() - self.updated < 300:
                return self.body
            try:
                inventory = CLIENT.get(
                    UPSTREAM + "/v1/models", headers={"User-Agent": USER_AGENT}, timeout=20
                )
                inventory.raise_for_status()
                try:
                    metadata = CLIENT.get("https://models.dev/api.json", timeout=20)
                    metadata.raise_for_status()
                    known = metadata.json()["opencode-go"]["models"]
                except (httpx.HTTPError, KeyError, ValueError):
                    known = {}
                rows = []
                for model in inventory.json()["data"]:
                    model_id = model["id"]
                    info = known.get(model_id, {})
                    muse = {
                        "is_hidden": False,
                        "display_label": info.get("name", model_id),
                        "limit": info.get("limit", {"context": 128000, "output": 16384}),
                    }
                    if info.get("release_date"):
                        muse["release_date"] = info["release_date"]
                    npm = info.get("provider", {}).get("npm", "")
                    protocol = "responses"
                    if npm == "@ai-sdk/anthropic" or model_id.startswith(("claude-", "minimax-m2")):
                        protocol = "anthropic"
                    elif npm == "@ai-sdk/google" or model_id.startswith("gemini-"):
                        protocol = "gemini"
                    rows.append({"id": model_id, "object": "model", "metadata": {
                        "muse-code": muse, "opencode-go-bridge": {"protocol": protocol}
                    }})
                if not rows:
                    raise ValueError("Empty OpenCode Go inventory")
                self.body = {"object": "list", "data": rows}
                CACHE.parent.mkdir(parents=True, exist_ok=True)
                temporary = CACHE.with_suffix(".tmp")
                temporary.write_text(json.dumps(self.body))
                temporary.replace(CACHE)
                self.updated = time.monotonic()
            except (httpx.HTTPError, KeyError, ValueError, OSError):
                if self.body is None:
                    raise
                # Keep the last successful inventory during an upstream outage.
                self.updated = time.monotonic() - 240
            return self.body


CATALOG = Catalog()


def translated_request(payload, token, session_id, protocol):
    """Use LiteLLM's maintained Responses/tool/multimodal protocol conversion."""
    params = dict(payload)
    provider = {"gemini": "gemini", "anthropic": "anthropic", "chat": "openai"}[protocol]
    params["model"] = provider + "/" + params["model"]
    if protocol == "chat":
        params["use_chat_completions_api"] = True
    if isinstance(params.get("reasoning"), dict):
        params["reasoning"] = dict(params["reasoning"])
        if params["reasoning"].get("effort") in ("xhigh", "max", "ultra"):
            params["reasoning"]["effort"] = "high"
    return litellm.responses(
        **params,
        api_key=token,
        api_base=UPSTREAM + {"gemini": "/google/v1beta", "anthropic": "/anthropic", "chat": "/openai/v1"}[protocol],
        extra_headers={
            "Authorization": "Bearer " + token,
            "User-Agent": USER_AGENT,
            "x-opencode-session": session_id,
        },
        timeout=600,
        drop_params=True,
        _skip_mcp_handler=True,
    )


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        pass

    def json_response(self, status, body):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.close_connection = True
        self.wfile.write(data)

    def do_GET(self):
        try:
            path = urlsplit(self.path).path
            if path == "/health":
                return self.json_response(200, {"status": "ok"})
            if path in ("/muse-code/models", "/v1/muse-code/models", "/v1/models"):
                return self.json_response(200, CATALOG.get())
            self.json_response(404, {"error": {"message": "Unknown bridge route: " + path}})
        except (BrokenPipeError, ConnectionResetError):
            pass
        except Exception:
            self.json_response(502, {"error": {"message": "OpenCode inventory is unavailable"}})

    def do_POST(self):
        if urlsplit(self.path).path not in ("/responses", "/v1/responses"):
            return self.json_response(404, {"error": {"message": "Unknown bridge route"}})
        auth = self.headers.get("Authorization", "")
        if not auth.startswith("Bearer ") or not auth[7:].strip():
            return self.json_response(401, {"error": {"message": "A bearer API key is required"}})
        self.response_started = False
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 < length <= 64 * 1024 * 1024:
                return self.json_response(413, {"error": {"message": "Invalid request size"}})
            raw = self.rfile.read(length)
            if self.headers.get("Content-Encoding", "").lower() == "gzip":
                raw = gzip.decompress(raw)
            payload = json.loads(raw)
            model = payload.get("model")
            if not isinstance(model, str) or not model:
                return self.json_response(400, {"error": {"message": "A model ID is required"}})
            session_id = self.headers.get("x-opencode-session")
            if not session_id:
                session_id = payload.get("prompt_cache_key") or str(uuid.uuid4())
            catalog = CATALOG.get()
            protocol = next((row["metadata"].get("opencode-go-bridge", {}).get("protocol", "responses")
                             for row in catalog["data"] if row["id"] == model), "responses")
            if model.startswith("gemini-"):
                protocol = "gemini"
            protocol = PROTOCOL_OVERRIDES.get(model, protocol)
            if protocol != "responses":
                response = translated_request(payload, auth[7:], session_id, protocol)
                return self.relay_translation(response, payload)
            else:
                # OpenCode translates its other model families to Responses.
                # Preserve their streams and encrypted reasoning without conversion.
                with CLIENT.stream(
                    "POST", UPSTREAM + "/openai/v1/responses", content=raw,
                    headers={
                        "Authorization": auth,
                        "Content-Type": "application/json",
                        "User-Agent": USER_AGENT,
                        "x-opencode-session": session_id,
                    },
                ) as response:
                    if response.status_code == 400:
                        response.read()
                        try:
                            error_type = response.json().get("error", {}).get("type")
                        except ValueError:
                            error_type = None
                        if error_type == "ModelProtocolUnsupported":
                            # Older or newly listed Chat Completions models may
                            # lack Responses support and complete metadata.
                            translated = translated_request(payload, auth[7:], session_id, "chat")
                            PROTOCOL_OVERRIDES[model] = "chat"
                            return self.relay_translation(translated, payload)
                    self.send_response(response.status_code)
                    self.send_header("Content-Type", response.headers.get("Content-Type", "application/json"))
                    self.send_header("Connection", "close")
                    self.end_headers()
                    self.response_started = True
                    self.close_connection = True
                    for chunk in response.iter_bytes():
                        self.wfile.write(chunk)
                        self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass
        except Exception as error:
            status = getattr(error, "status_code", 502)
            status = status if isinstance(status, int) and 400 <= status <= 599 else 502
            message = "OpenCode Go request failed"
            upstream_response = getattr(error, "response", None)
            if upstream_response is not None:
                try:
                    detail = upstream_response.json().get("error", {}).get("message")
                    if isinstance(detail, str):
                        message = detail.replace(auth[7:], "<redacted>")[:2000]
                except (ValueError, AttributeError):
                    pass
            body = {"error": {"type": type(error).__name__, "message": message}}
            if self.response_started:
                try:
                    self.write_event({"type": "error", **body})
                except OSError:
                    pass
            else:
                self.json_response(status, body)

    def relay_translation(self, response, payload):
        if not payload.get("stream"):
            return self.json_response(200, response.model_dump(mode="json", exclude_none=True))
        iterator = iter(response)
        try:
            # Fail before committing HTTP 200 if stream initialization fails.
            first = next(iterator)
            self.event_sequence = 0
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Connection", "close")
            self.end_headers()
            self.response_started = True
            self.close_connection = True
            self.write_event(first)
            for event in iterator:
                self.write_event(event)
        finally:
            close = getattr(response, "close", None)
            if close:
                close()

    def write_event(self, event):
        if hasattr(event, "model_dump"):
            event = event.model_dump(mode="json", exclude_none=True)
        self.event_sequence = getattr(self, "event_sequence", 0) + 1
        event["sequence_number"] = self.event_sequence
        self.wfile.write(("data: " + json.dumps(event) + "\n\n").encode())
        self.wfile.flush()


if __name__ == "__main__":
    ThreadingHTTPServer(("127.0.0.1", 52187), Handler).serve_forever()
