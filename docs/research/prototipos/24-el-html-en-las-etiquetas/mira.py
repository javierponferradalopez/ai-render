#!/usr/bin/env python3
"""Abre la Pizarra de verdad y captura la ventana caso a caso.

No hay atajo aquí a propósito: el criterio del ticket es ver los constructos
*dibujados*, no descritos, así que esto habla con el binario real por stdio
—como haría el host— y fotografía lo que sale en pantalla.

    python3 mira.py casos/fc-01-br-cerrado.mmd casos/fc-04-negrita.mmd
"""
import json, os, subprocess, sys, time

RAIZ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../.."))
BIN = os.path.join(RAIZ, "target/release/flipchart")
CAPTURAS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "capturas")


def peticion(proceso, cuerpo):
    proceso.stdin.write(json.dumps(cuerpo) + "\n")
    proceso.stdin.flush()
    while True:
        linea = proceso.stdout.readline()
        if not linea:
            return None
        respuesta = json.loads(linea)
        if respuesta.get("id") == cuerpo.get("id"):
            return respuesta


def main(rutas):
    os.makedirs(CAPTURAS, exist_ok=True)
    proceso = subprocess.Popen(
        [BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True
    )
    peticion(proceso, {
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "mira", "version": "0"}},
    })
    proceso.stdin.write(json.dumps(
        {"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
    proceso.stdin.flush()

    for numero, ruta in enumerate(rutas, start=2):
        nombre = os.path.splitext(os.path.basename(ruta))[0]
        with open(ruta) as fichero:
            fuente = fichero.read()
        respuesta = peticion(proceso, {
            "jsonrpc": "2.0", "id": numero, "method": "tools/call",
            "params": {"name": "show",
                       "arguments": {"view_id": nombre, "diagram": fuente}},
        })
        texto = respuesta["result"]["content"][0]["text"]
        print(f"== {nombre}\n{texto}")
        time.sleep(1.5)  # la ventana rasteriza en su hilo; no hay señal de vuelta
        captura = subprocess.run(
            ["screencapture", "-x", os.path.join(CAPTURAS, f"{nombre}.png")],
            capture_output=True, text=True,
        )
        if captura.returncode:
            print("  sin captura:", captura.stderr.strip() or
                  "screencapture necesita permiso de grabación de pantalla")

    # El Proceso de la pizarra vive hasta que muere la sesión MCP y no se va al
    # cerrar stdin: aquí la sesión la acaba quien la abrió.
    proceso.stdin.close()
    proceso.terminate()
    proceso.wait(timeout=10)


if __name__ == "__main__":
    main(sys.argv[1:] or sys.exit(__doc__))
