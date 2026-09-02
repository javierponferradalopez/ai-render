#!/usr/bin/env python3
"""Sirve ./srv/doc.json en 127.0.0.1 y obedece a ./srv/mode: ok | garbage | 500."""
import http.server, socketserver, os, sys, pathlib
HERE = pathlib.Path(__file__).parent
class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def do_GET(self):
        mode = (HERE / "mode").read_text().strip() if (HERE / "mode").exists() else "ok"
        if mode == "500":
            self.send_response(500); self.end_headers(); self.wfile.write(b"boom"); return
        if mode == "garbage":
            body = b"esto no es json en absoluto <html>"
        else:
            body = (HERE / "doc.json").read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers(); self.wfile.write(body)
socketserver.TCPServer.allow_reuse_address = True
port = int(sys.argv[1])
with socketserver.TCPServer(("127.0.0.1", port), H) as s:
    s.serve_forever()
