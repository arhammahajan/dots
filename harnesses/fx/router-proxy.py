#!/usr/bin/env python3
"""Adapt fx Chat requests to the existing loopback model-router."""
import http.client
import json
import os
import re
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

TARGET = urlsplit(os.environ.get("MODEL_ROUTER_URL", "http://127.0.0.1:4000"))
HOP = {"host", "connection", "content-length", "transfer-encoding", "keep-alive",
       "proxy-authenticate", "proxy-authorization", "te", "trailer", "upgrade"}
MAX_BODY = 32 * 1024 * 1024
SESSION = re.compile(r"Bearer (fx-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12})")


def normalize(payload):
    """Keep instruction order while using one leading system message."""
    messages = payload.get("messages", [])
    count = 0
    for message in messages:
        if message.get("role") != "system" or not isinstance(message.get("content"), str):
            break
        count += 1
    if count > 1:
        payload = dict(payload, messages=[{
            "role": "system",
            "content": "\n\n".join(message["content"] for message in messages[:count]),
        }, *messages[count:]])
    return payload


class Proxy(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return
        session = SESSION.fullmatch(self.headers.get("Authorization", ""))
        if session is None:
            self.send_error(401, "Run fx through fx-with-router")
            return
        if self.headers.get("Transfer-Encoding"):
            self.send_error(400, "Content-Length is required")
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 < length <= MAX_BODY:
                self.send_error(413)
                return
            payload = json.loads(self.rfile.read(length))
            if not isinstance(payload, dict) or not isinstance(payload.get("messages"), list):
                raise ValueError("Expected a Chat request")
            if not all(isinstance(message, dict) for message in payload["messages"]):
                raise ValueError("Expected message objects")
            body = json.dumps(normalize(payload)).encode()
        except (ValueError, TypeError):
            self.send_error(400, "Invalid Chat request")
            return
        # The bearer value is a public affinity ID, never an upstream credential.
        headers = {"Content-Type": "application/json", "Accept": "text/event-stream",
                   "Authorization": "Bearer unused", "x-session-id": session[1]}
        factory = http.client.HTTPSConnection if TARGET.scheme == "https" else http.client.HTTPConnection
        conn = factory(TARGET.hostname, TARGET.port, timeout=600)
        started = False
        try:
            conn.request("POST", TARGET.path.rstrip("/") + self.path, body, headers)
            response = conn.getresponse()
            self.send_response(response.status)
            for key, value in response.getheaders():
                if key.lower() not in HOP:
                    self.send_header(key, value)
            self.end_headers()
            started = True
            while chunk := response.read1(65536):
                self.wfile.write(chunk)
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass
        except (OSError, http.client.HTTPException):
            if not started:
                self.send_error(502, "Model router unavailable")
        finally:
            conn.close()


class Server(ThreadingHTTPServer):
    daemon_threads = True

    def get_request(self):
        connection, address = super().get_request()
        connection.settimeout(600)
        return connection, address


if __name__ == "__main__":
    with Server(("127.0.0.1", 4001), Proxy) as server:
        server.serve_forever()
