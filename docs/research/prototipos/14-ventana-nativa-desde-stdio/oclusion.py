#!/usr/bin/env python3
"""¿Qué pasa cuando la ventana de la pizarra queda COMPLETAMENTE TAPADA?
Es el caso normal: el usuario trabaja en su terminal a pantalla casi completa.
macOS deja de entregar eventos de dibujo a ventanas ocluidas."""
import json, os, subprocess, sys, time
BIN = os.environ["SPIKE_BIN"]
args = sys.argv[1:]

def osa(script):
    return subprocess.run(["osascript", "-e", script], capture_output=True, text=True).stdout.strip()

pantalla = osa('tell application "Finder" to get bounds of window of desktop') or "0, 0, 1920, 1080"
print(f"pantalla: {pantalla}")

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
print("pizarra al frente:", {k: call("ping")[k] for k in ("frames", "views")})

# tapo la pizarra con una ventana de TextEdit a pantalla completa
osa('tell application "TextEdit" to activate')
time.sleep(1.0)
osa('tell application "TextEdit" to if (count of documents) = 0 then make new document')
time.sleep(0.5)
osa(f'tell application "TextEdit" to set bounds of front window to {{{pantalla}}}')
time.sleep(1.5)
print("pizarra tapada por TextEdit")

print(f"\n{'intento':>8} {'ms hasta repintar':>19}")
peor = 0
for i in range(4):
    antes = call("ping")
    call("show", {"view": f"v{i+1}", "nodes": 4})
    t0 = time.time(); espera = None
    while time.time() - t0 < 12:
        time.sleep(0.05)
        if call("ping")["frames"] > antes["frames"]:
            espera = (time.time()-t0)*1000; break
    peor = max(peor, espera or 12000)
    print(f"{i+1:>8} {'>12000' if espera is None else round(espera):>19}")

print(f"\npeor caso tapada: {peor:.0f} ms")
print("\nmuerte del host CON LA VENTANA TAPADA (¿se queda el proceso colgado?):")
p.stdin.close()
t0 = time.time()
try:
    code = p.wait(timeout=25)
    print(f"  salió solo con código {code} tras {time.time()-t0:.1f}s")
except subprocess.TimeoutExpired:
    print(f"  !! SIGUE VIVO tras 25s -- proceso huérfano con su ventana tapada"); p.kill()
osa('tell application "TextEdit" to close every document without saving')
osa('tell application "TextEdit" to quit')
print(p.stderr.read())
