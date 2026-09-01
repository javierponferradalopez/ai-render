#!/usr/bin/env python3
"""Tanda de medición con la ventana tapada, 3 repeticiones por variante.
Mide lo que decide el diseño:
  - latencia de repaint con la ventana ocluida
  - tiempo desde que muere el host hasta que el proceso desaparece
"""
import json, os, subprocess, sys, time, statistics
BIN = os.environ["SPIKE_BIN"]

def osa(s):
    return subprocess.run(["osascript","-e",s],capture_output=True,text=True).stdout.strip()

PANT = osa('tell application "Finder" to get bounds of window of desktop') or "0, 0, 1920, 1080"

def tapar():
    osa('tell application "TextEdit" to activate')
    time.sleep(0.6)
    osa('tell application "TextEdit" to if (count of documents) = 0 then make new document')
    time.sleep(0.4)
    osa(f'tell application "TextEdit" to set bounds of front window to {{{PANT}}}')
    time.sleep(1.2)

def destapar():
    osa('tell application "TextEdit" to close every document without saving')
    osa('tell application "TextEdit" to quit')
    time.sleep(0.5)

def una_vuelta(args):
    p = subprocess.Popen([BIN]+args, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.DEVNULL, text=True, bufsize=1)
    n = [0]
    def call(m, params=None):
        n[0] += 1
        msg = {"id": n[0], "method": m}
        if params: msg["params"] = params
        p.stdin.write(json.dumps(msg)+"\n"); p.stdin.flush()
        return json.loads(p.stdout.readline())["result"]
    time.sleep(0.7)
    call("show", {"view":"v0","nodes":3})
    time.sleep(0.8)
    tapar()
    lat = []
    for i in range(3):
        antes = call("ping")
        call("show", {"view": f"v{i+1}", "nodes": 4})
        t0 = time.time(); esp = None
        while time.time()-t0 < 15:
            time.sleep(0.05)
            if call("ping")["frames"] > antes["frames"]:
                esp = (time.time()-t0)*1000; break
        lat.append(esp if esp is not None else float("inf"))
        time.sleep(0.4)
    p.stdin.close()
    t0 = time.time()
    try:
        p.wait(timeout=30); muerte = time.time()-t0
    except subprocess.TimeoutExpired:
        p.kill(); muerte = float("inf")
    destapar()
    return lat, muerte

VARIANTES = [
    ("por defecto",              ["--policy","accessory"]),
    ("+ App Nap off",            ["--policy","accessory","--no-app-nap"]),
    ("+ salida dura",            ["--policy","accessory","--hard-exit"]),
    ("+ App Nap off + dura",     ["--policy","accessory","--no-app-nap","--hard-exit"]),
]
print(f"{'variante':<24} {'repaint ocluida (ms)':<34} {'muerte (s)'}")
for nombre, args in VARIANTES:
    lats, muertes = [], []
    for _ in range(3):
        l, m = una_vuelta(args)
        lats += l; muertes.append(m)
    fmt = lambda xs: " ".join("∞" if x == float("inf") else f"{x:.0f}" for x in xs)
    print(f"{nombre:<24} {fmt(lats):<34} {fmt(muertes)}")
