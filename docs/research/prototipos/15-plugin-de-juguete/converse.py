#!/usr/bin/env python3
"""Sesion stream-json: espera WAIT segundos, envia un mensaje de usuario, recoge
el stream hasta el result. Deja stdout en <base>.out."""
import json, subprocess, sys, time, threading

wait = float(sys.argv[1]); base = sys.argv[2]; cmd = sys.argv[3:]
msg = {"type": "user", "message": {"role": "user", "content": "Responde solo: ok"}}
t0 = time.time()
out = open(base + ".out", "wb")
p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=out, stderr=open(base + ".err", "wb"))
time.sleep(wait)
try:
    p.stdin.write((json.dumps(msg) + "\n").encode()); p.stdin.flush()
except Exception as e:
    print("write failed:", e)
try:
    p.stdin.close()
except Exception:
    pass
try:
    rc = p.wait(120)
except subprocess.TimeoutExpired:
    p.kill(); rc = "killed"
print("rc=%s wall=%.2f" % (rc, time.time() - t0))
