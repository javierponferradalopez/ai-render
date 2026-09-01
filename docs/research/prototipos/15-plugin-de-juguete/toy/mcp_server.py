#!/usr/bin/env python3
"""Servidor MCP stdio minimo. Registra en TOY_LOG cada mensaje y cada senal."""
import json, os, signal, sys, time

LOG = os.environ.get("TOY_LOG", "/tmp/toy-plugin.log")

def log(msg):
    with open(LOG, "a") as f:
        f.write("%.3f server %s\n" % (time.time(), msg))

def on_signal(signum, _frame):
    log("SIGNAL %s" % signal.Signals(signum).name)
    sys.exit(0)

for s in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
    signal.signal(s, on_signal)

log("server start pid=%d" % os.getpid())

def send(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()

TOOLS = [{
    "name": "toy_ping",
    "description": "Devuelve pong. Herramienta de juguete para medir el peaje fijo.",
    "inputSchema": {"type": "object", "properties": {}},
}]

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except Exception as e:
        log("unparseable %r %s" % (line[:120], e))
        continue
    method = req.get("method")
    log("recv %s id=%s" % (method, req.get("id")))
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": req["id"], "result": {
            "protocolVersion": "2025-06-18",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "toy", "version": "0.0.1"},
        }})
    elif method == "tools/list":
        send({"jsonrpc": "2.0", "id": req["id"], "result": {"tools": TOOLS}})
    elif method == "tools/call":
        send({"jsonrpc": "2.0", "id": req["id"], "result": {
            "content": [{"type": "text", "text": "pong"}]}})
    elif method in ("resources/list", "prompts/list"):
        key = method.split("/")[0]
        send({"jsonrpc": "2.0", "id": req["id"], "result": {key: []}})
    elif method and method.startswith("notifications/"):
        pass
    elif "id" in req:
        send({"jsonrpc": "2.0", "id": req["id"],
              "error": {"code": -32601, "message": "method not found: %s" % method}})

log("server stdin EOF -> exit")
