#!/usr/bin/env python3
"""Mide cuánto tarda la pizarra en reflejar un `show` cuando la ventana NO
tiene el foco -- que es el caso normal: el usuario está en su terminal."""
import json, os, subprocess, sys, time
BIN = os.environ["SPIKE_BIN"]
args = sys.argv[1:]

def activar(app):
    subprocess.run(["osascript", "-e", f'tell application "{app}" to activate'],
                   capture_output=True)

p = subprocess.Popen([BIN]+args, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                     stderr=subprocess.PIPE, text=True, bufsize=1)
n = 0
def call(m, params=None):
    global n; n += 1
    msg = {"id": n, "method": m}
    if params: msg["params"] = params
    p.stdin.write(json.dumps(msg)+"\n"); p.stdin.flush()
    return json.loads(p.stdout.readline())["result"]

time.sleep(0.8)
call("show", {"view": "v0", "nodes": 3})
time.sleep(1.0)
print("con foco en la pizarra:", call("ping"))

# el usuario vuelve a su terminal
activar("Finder")
time.sleep(1.0)
print("tras devolver el foco :", call("ping"))

print("\nlatencia de repaint con la ventana en segundo plano:")
print(f"{'intento':>8} {'frames antes':>13} {'ms hasta repintar':>19}")
peor = 0.0
for i in range(6):
    antes = call("ping")
    call("show", {"view": f"v{i+1}", "nodes": 4})
    t0 = time.time()
    espera = None
    while time.time() - t0 < 10:
        time.sleep(0.05)
        ahora = call("ping")
        if ahora["frames"] > antes["frames"] and ahora["views"] == antes["views"] + 1:
            espera = (time.time() - t0) * 1000
            break
    peor = max(peor, espera if espera is not None else 10000)
    print(f"{i+1:>8} {antes['frames']:>13} {espera if espera is None else round(espera):>19}")
    time.sleep(0.5)
print(f"\npeor caso: {peor:.0f} ms")

print("\nmuerte del host con la ventana en segundo plano:")
p.stdin.close()
t0 = time.time()
try:
    code = p.wait(timeout=20)
    print(f"  salió solo con código {code} tras {time.time()-t0:.1f}s")
except subprocess.TimeoutExpired:
    print("  !! NO murió en 20s"); p.kill()
print(p.stderr.read())
