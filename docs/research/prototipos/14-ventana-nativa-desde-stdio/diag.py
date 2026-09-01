#!/usr/bin/env python3
"""Diagnóstico: ¿avanza el event loop cuando el hilo del servidor pide repintar?"""
import json, os, subprocess, sys, time
BIN = os.environ["SPIKE_BIN"]
args = sys.argv[1:]
p = subprocess.Popen([BIN]+args, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                     stderr=subprocess.PIPE, text=True, bufsize=1)
n=0
def call(m, params=None):
    global n; n+=1
    msg={"id":n,"method":m}
    if params: msg["params"]=params
    p.stdin.write(json.dumps(msg)+"\n"); p.stdin.flush()
    return json.loads(p.stdout.readline())["result"]
time.sleep(1.0)
print("t=1.0 antes de show :", call("ping"))
call("show", {"view":"actual","nodes":5})
for t in (0.3, 1.0, 2.0, 4.0):
    time.sleep(t)
    print(f"t=+{t} tras show   :", call("ping"))
call("show", {"view":"propuesto","nodes":8})
time.sleep(1.0)
print("tras 2o show        :", call("ping"))
p.stdin.close()
t0=time.time()
try:
    print("salió con", p.wait(timeout=12), f"tras {time.time()-t0:.1f}s")
except subprocess.TimeoutExpired:
    print(f"NO murió en 12s; lo mato"); p.kill()
print(p.stderr.read())
