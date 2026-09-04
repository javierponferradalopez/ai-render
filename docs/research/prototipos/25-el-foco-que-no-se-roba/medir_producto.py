#!/usr/bin/env python3
"""La misma medición, sobre el **producto**: `flipchart` hablando MCP de verdad
por stdio, como lo lanza Claude Code.

El producto no tiene comando de diagnóstico —y no debe tenerlo—, así que quien
mira es `zsonda` desde fuera, y quién tiene el teclado lo dice el sistema.

Uso: FLIPCHART=/ruta/flipchart ZSONDA=/ruta/zsonda python3 medir_producto.py
"""
import json, os, subprocess, sys, time

FLIPCHART = os.environ["FLIPCHART"]
ZSONDA = os.environ["ZSONDA"]
TERMINAL = os.environ.get("TERMINAL_APP", "Alacritty")
DIAGRAMA = "flowchart LR\n  servidor[Servidor MCP] --> visor[Visor]\n"


class Sesion:
    """Un host MCP mínimo: lo justo para que el servidor se crea que hay
    conversación."""

    def __init__(self):
        self.n = 0
        self.p = subprocess.Popen([FLIPCHART], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, text=True, bufsize=1)
        self.rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                "clientInfo": {"name": "medida", "version": "0"}})
        self.notifica("notifications/initialized")

    def rpc(self, method, params):
        self.n += 1
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.n,
                                       "method": method, "params": params}) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())

    def notifica(self, method):
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": method}) + "\n")
        self.p.stdin.flush()

    def show(self, view_id):
        r = self.rpc("tools/call", {"name": "show",
                                    "arguments": {"view_id": view_id, "diagram": DIAGRAMA}})
        contenido = r["result"]["content"][0]["text"]
        return ("rechazo" if r["result"].get("isError") else "dibujada"), contenido.split("\n")[0]


def frontal():
    guion = ('tell application "System Events" to get name of '
             'first application process whose frontmost is true')
    return subprocess.run(["osascript", "-e", guion], capture_output=True, text=True).stdout.strip()


def terminal_delante():
    for _ in range(4):
        subprocess.run(["open", "-a", TERMINAL], capture_output=True)
        time.sleep(1.0)
        if frontal().lower() == TERMINAL.lower():
            return True
    return False


def sonda(pid):
    salida = subprocess.run([ZSONDA, str(pid), TERMINAL], capture_output=True, text=True).stdout
    return json.loads(salida)


def observa(pid, segundos):
    """La sonda desde fuera cuesta un proceso por muestra, así que se muestrea a
    5 Hz en vez de a 30: basta para ver si la ventana sube y si se queda."""
    fin = time.time() + segundos
    serie = []
    while time.time() < fin:
        serie.append(sonda(pid))
        time.sleep(0.2)
    encima = [m["encima_del_terminal"] for m in serie]
    # Quién más se puso delante: sin esto una caída del orden Z no distingue
    # "la ventana se cae sola" de "otra app ha entrado en escena".
    fuera = {TERMINAL.lower(), "flipchart"}
    terceros = sorted({m["delante"] for m in serie
                       if m["delante"] and m["delante"].lower() not in fuera})
    return {
        "muestras": len(serie),
        "aparece": any(m["mi_z"] is not None for m in serie),
        "traza_encima": "".join({True: "^", False: ".", None: "_"}[e] for e in encima),
        "encima_al_final": encima[-1],
        "app_con_el_teclado": frontal(),
        "terceros": terceros,
    }


def main():
    print(f"terminal delante: {terminal_delante()}")
    sesion = Sesion()
    pid = sesion.p.pid
    print(f"flipchart pid={pid}, antes del primer show: {json.dumps(sonda(pid))}")

    print(f"\nprimer show -> {sesion.show('actual')}")
    print("  " + json.dumps(observa(pid, 3.0), ensure_ascii=False))

    terminal_delante()
    print(f"\nsegundo show, con el terminal delante -> {sesion.show('propuesto')}")
    print("  " + json.dumps(observa(pid, 2.0), ensure_ascii=False))

    print("\nmuere la sesión (EOF en stdin)")
    sesion.p.stdin.close()
    t0 = time.time()
    try:
        print(f"  salió con código {sesion.p.wait(timeout=15)} tras {time.time() - t0:.1f}s")
    except subprocess.TimeoutExpired:
        sesion.p.kill()
        print("  !! SIGUE VIVO")


main()
