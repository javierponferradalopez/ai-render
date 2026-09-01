#!/usr/bin/env python3
"""Hace de Claude Code: lanza el spike como proceso hijo con stdin/stdout
pipeados (sin tty, sin app bundle) y observa qué le pasa a la ventana.

Uso: python3 host.py [--policy accessory] [--eager] [--keep]
"""
import json, os, subprocess, sys, time, shutil, signal

BIN = os.environ.get("SPIKE_BIN")
SHOTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "capturas")

def observe_external(pid):
    """Lo que el sistema ve del proceso, sin preguntarle a él."""
    out = {}
    try:
        r = subprocess.run(["lsappinfo", "info", "-only", "name,pid,StatusLabel", str(pid)],
                           capture_output=True, text=True, timeout=5)
        out["lsappinfo"] = r.stdout.strip() or r.stderr.strip()
    except Exception as e:
        out["lsappinfo"] = f"err: {e}"
    try:
        r = subprocess.run(["lsappinfo", "front"], capture_output=True, text=True, timeout=5)
        front = r.stdout.strip()
        out["front_asn"] = front
        r2 = subprocess.run(["lsappinfo", "info", "-only", "name,pid", front],
                            capture_output=True, text=True, timeout=5)
        out["front"] = r2.stdout.strip()
    except Exception as e:
        out["front"] = f"err: {e}"
    return out

def shot(name):
    os.makedirs(SHOTS, exist_ok=True)
    path = os.path.join(SHOTS, f"{name}.png")
    subprocess.run(["screencapture", "-x", path], capture_output=True)
    return path

class Spike:
    def __init__(self, args):
        self.p = subprocess.Popen([BIN] + args, stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  text=True, bufsize=1)
        self.n = 0
    def call(self, method, params=None, timeout=5):
        self.n += 1
        msg = {"id": self.n, "method": method}
        if params: msg["params"] = params
        self.p.stdin.write(json.dumps(msg) + "\n"); self.p.stdin.flush()
        line = self.p.stdout.readline()
        return json.loads(line) if line else None

def main():
    args = sys.argv[1:]
    keep = "--keep" in args
    args = [a for a in args if a != "--keep"]
    print(f"== lanzo el spike como hijo: {' '.join(args) or '(sin flags)'}")
    s = Spike(args)
    pid = s.p.pid
    print(f"   pid={pid}  padre={os.getpid()}  tty en stdin del hijo: pipe (no tty)")

    time.sleep(1.5)
    print("\n-- T1: arrancado, ANTES de cualquier show")
    r = s.call("ping")
    print(f"   auto:   {r['result'] if r else 'sin respuesta'}")
    for k, v in observe_external(pid).items():
        print(f"   {k}: {v}")

    print("\n-- T2: primer show (aquí debería nacer la ventana)")
    t0 = time.time()
    r = s.call("show", {"view": "actual", "nodes": 5})
    dt = (time.time() - t0) * 1000
    print(f"   respuesta en {dt:.1f} ms: {r['result'] if r else None}")
    time.sleep(1.5)
    r = s.call("ping")
    print(f"   auto:   {r['result'] if r else None}")
    for k, v in observe_external(pid).items():
        print(f"   {k}: {v}")
    print(f"   captura: {shot('t2-tras-primer-show')}")

    print("\n-- T3: segunda vista, y el servidor sigue respondiendo")
    print(f"   show:  {s.call('show', {'view': 'propuesto', 'nodes': 8})['result']}")
    print(f"   ping:  {s.call('ping')['result']}")
    time.sleep(1.0)
    print(f"   captura: {shot('t3-dos-vistas')}")

    print("\n-- T4: clear (estado 'la pizarra está vacía')")
    print(f"   clear: {s.call('clear')['result']}")
    time.sleep(1.0)
    print(f"   captura: {shot('t4-clear')}")

    if keep:
        print("\n-- me quedo vivo 60s (--keep). Ctrl-C para acabar.")
        time.sleep(60)

    print("\n-- T5: cierro stdin = el host muere")
    s.p.stdin.close()
    time.sleep(1.0)
    print(f"   captura: {shot('t5-sesion-terminada')}")
    t0 = time.time()
    try:
        code = s.p.wait(timeout=15)
        print(f"   el hijo salió solo con código {code} tras {time.time()-t0:.1f}s")
    except subprocess.TimeoutExpired:
        print("   !! el hijo NO murió al cerrarse stdin; lo mato")
        s.p.kill(); code = None
    time.sleep(0.5)
    print(f"   captura: {shot('t6-tras-la-muerte')}")
    print("\n-- stderr del hijo:")
    print(s.p.stderr.read())

if __name__ == "__main__":
    if not BIN or not os.path.exists(BIN):
        sys.exit("define SPIKE_BIN con la ruta al binario")
    main()
