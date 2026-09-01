#!/usr/bin/env python3
"""Arranca `claude` en modo stream-json, mantiene stdin abierto HOLD segundos sin
enviar nada (la sesion vive, arranca sus servidores MCP y no llama al API) y
luego cierra. Deja stdout/stderr en <base>.out/.err."""
import subprocess, sys, time

hold = float(sys.argv[1]); base = sys.argv[2]; cmd = sys.argv[3:]
t0 = time.time()
with open(base + ".out", "wb") as o, open(base + ".err", "wb") as e:
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=o, stderr=e)
    deadline = t0 + hold
    while time.time() < deadline and p.poll() is None:
        time.sleep(0.25)
    early = p.poll()
    try:
        p.stdin.close()
    except Exception:
        pass
    try:
        rc = p.wait(20)
    except subprocess.TimeoutExpired:
        p.kill(); p.wait(); rc = "killed"
print("t0=%.3f exited_early=%s rc=%s wall=%.2f" % (t0, early, rc, time.time() - t0))
