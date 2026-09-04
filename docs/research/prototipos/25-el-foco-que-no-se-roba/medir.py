#!/usr/bin/env python3
"""Hace de Claude Code: lanza el spike como proceso hijo por stdio y sin tty, y
mide en cada momento del ciclo **quién tiene la pantalla y quién el teclado**.

Los cuatro momentos que el ticket manda comprobar, por variante de aparición:
  1. el primer `show`, con el terminal delante
  2. un `show` sobre la ventana ya en pie
  3. el ⌘W del usuario
  4. el primer `show` tras ese ⌘W

Lo que se mira en cada momento, a 30 Hz:
  - **el teclado**: `NSApp.isActive` y `isKeyWindow`, leídos de AppKit dentro
    del proceso. Si nunca son ciertos, no puede haberse comido una tecla: el
    teclado lo entrega el WindowServer a la app activa.
  - **la pantalla**: el orden Z real del WindowServer, medido **relativo a la
    ventana del terminal**. El z absoluto no vale — cualquier tercera app que
    pase por delante lo mueve sin que nada de esto haya cambiado.

Uso: SPIKE_BIN=/ruta/foco-spike python3 medir.py [variante ...] [--tiradas N]
     variante = activate|key|regardless [+quiet] [+dNNN]  (dNNN = --delay-ms NNN)
"""
import json, os, statistics, subprocess, sys, time

BIN = os.environ["SPIKE_BIN"]
TERMINAL = os.environ.get("TERMINAL_APP", "Alacritty")
HZ = 30.0
argv = [a for a in sys.argv[1:] if not a.startswith("--")]
TIRADAS = int(os.environ.get("TIRADAS", "3"))
VARIANTES = argv or ["activate", "key", "regardless", "regardless+quiet"]


def argumentos(variante):
    """`regardless+quiet+d300` = `regardless`, sin el activate de winit al
    arrancar, y apareciendo 300 ms después de que el `show` llegue."""
    partes = variante.split("+")
    args = [BIN, "--appear", partes[0]]
    for extra in partes[1:]:
        if extra == "quiet":
            args.append("--quiet-launch")
        elif extra == "frame":
            args.append("--after-frame")
        elif extra.startswith("d"):
            args += ["--delay-ms", extra[1:]]
    return args


class Spike:
    def __init__(self, variante, log):
        self.n = 0
        self.p = subprocess.Popen(argumentos(variante), stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=log, text=True, bufsize=1)

    def call(self, method, params=None):
        self.n += 1
        msg = {"id": self.n, "method": method}
        if params:
            msg["params"] = params
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())["result"]

    def muestrea(self, segundos):
        """Serie temporal: es la única forma de cazar un foco que se roba y se
        devuelve antes de que nadie mire."""
        fin = time.time() + segundos
        serie = []
        while time.time() < fin:
            serie.append(self.call("probe"))
            time.sleep(1.0 / HZ)
        return serie


def frontal():
    guion = ('tell application "System Events" to get name of '
             'first application process whose frontmost is true')
    return subprocess.run(["osascript", "-e", guion], capture_output=True, text=True).stdout.strip()


def dock_icono(proceso="foco-spike"):
    """¿Tiene icono en el Dock? Una app `Regular` no es "background only"; una
    `Accessory` sí. Es la mitad del paquete que el ticket quiere quedarse."""
    guion = f'tell application "System Events" to get background only of application process "{proceso}"'
    r = subprocess.run(["osascript", "-e", guion], capture_output=True, text=True)
    if r.returncode != 0:
        return f"no visible para System Events: {r.stderr.strip()[:60]}"
    return {"false": "sí", "true": "no"}.get(r.stdout.strip(), r.stdout.strip())


def terminal_delante():
    """Deja el terminal donde el usuario lo tiene cuando el agente dibuja:
    delante y con el teclado. Sin esto el escenario no es el del ticket."""
    for _ in range(4):
        subprocess.run(["open", "-a", TERMINAL], capture_output=True)
        time.sleep(1.0)
        if frontal().lower() == TERMINAL.lower():
            return True
    return False


def encima_del_terminal(muestra):
    """¿Tapa la pizarra la ventana del terminal? None si falta alguna de las
    dos en pantalla."""
    yo = muestra["mi_z"]
    if yo is None:
        return None
    ventanas = muestra["en_pantalla"]
    suyo = next((i for i, v in enumerate(ventanas) if v["owner"].lower() == TERMINAL.lower()), None)
    if suyo is None:
        return None
    return yo < suyo


def terceros(serie):
    """Quién más se puso delante mientras se medía. El escritorio está vivo: si
    una tercera app entra en escena, la tirada no dice lo que se quería
    preguntar y se marca."""
    fuera = {TERMINAL.lower(), "foco-spike"}
    delante = {m["delante"]["owner"] for m in serie if m.get("delante")}
    return sorted(o for o in delante if o.lower() not in fuera)


def episodio(nombre, serie):
    ultima = serie[-1]
    encima = [encima_del_terminal(m) for m in serie]
    visto = [e for e in encima if e is not None]
    return {
        "momento": nombre,
        "muestras": len(serie),
        # la pantalla
        "aparece": any(m["mi_z"] is not None for m in serie),
        "encima_del_terminal_al_final": encima[-1],
        "encima_del_terminal_pct": round(100 * sum(visto) / len(visto)) if visto else None,
        # el teclado, en sus tres caras
        "app_activa_alguna_vez": any(m["app_activa"] for m in serie),
        "ventana_key_alguna_vez": any(m["ventana_key"] for m in serie),
        "delante_de_todo_alguna_vez": any(m["mi_z"] == 0 for m in serie),
        # el estado del que dibuja, para saber si lo anterior es fresco
        "policy": ultima["policy"],
        "visible": ultima["ventana_visible"],
        "rancio_p90_ms": round(statistics.quantiles([m["now"] - m["estado_de"] for m in serie],
                                                    n=10)[-1] * 1000),
        # la traza, para distinguir "nunca sube" de "sube y se cae"
        "traza_encima": "".join({True: "^", False: ".", None: "_"}[e] for e in encima),
        "traza_activa": "".join("A" if m["app_activa"] else "." for m in serie),
        "traza_visible": "".join("V" if m["ventana_visible"] else "." for m in serie),
        "terceros": terceros(serie),
    }


def tirada(variante, log):
    limpio = terminal_delante()
    spike = Spike(variante, log)
    time.sleep(0.4)
    partida = spike.call("probe")
    episodios = [{"momento": "0. antes del primer show",
                  "aparece": partida["mi_z"] is not None,
                  "delante": partida["delante"]["owner"] if partida["delante"] else None,
                  "ventanas": len(partida["en_pantalla"])}]

    spike.call("show", {"view": "actual"})
    episodios.append(episodio("1. primer show", spike.muestrea(3.0)))
    # Confirmación desde fuera: quién tiene el teclado según el sistema, y si
    # el icono del Dock —la mitad que se quiere conservar— está ahí.
    episodios.append({"momento": "1b. desde fuera",
                      "app_con_el_teclado": frontal(),
                      "icono_en_el_dock": dock_icono()})

    terminal_delante()
    spike.call("show", {"view": "propuesto"})
    episodios.append(episodio("2. show sobre ventana en pie", spike.muestrea(2.0)))

    terminal_delante()
    spike.call("close")
    time.sleep(1.0)
    tras_cerrar = spike.call("probe")
    episodios.append({"momento": "3. ⌘W del usuario",
                      "aparece": tras_cerrar["mi_z"] is not None,
                      "visible": tras_cerrar["ventana_visible"],
                      "delante": tras_cerrar["delante"]["owner"] if tras_cerrar["delante"] else None})

    terminal_delante()
    spike.call("show", {"view": "tercera"})
    episodios.append(episodio("4. primer show tras el ⌘W", spike.muestrea(3.0)))

    spike.p.stdin.close()
    try:
        codigo = spike.p.wait(timeout=10)
    except subprocess.TimeoutExpired:
        spike.p.kill()
        codigo = "SIGUE VIVO"
    intrusos = sorted({t for e in episodios for t in e.get("terceros", [])})
    return {"terminal_delante": limpio, "salida": codigo, "intrusos": intrusos,
            "limpia": limpio and not intrusos, "episodios": episodios}


def main():
    todo = []
    for variante in VARIANTES:
        for i in range(TIRADAS):
            with open(f"/tmp/foco-{variante}-{i}.stderr", "w") as log:
                r = tirada(variante, log)
            r["variante"], r["tirada"] = variante, i
            todo.append(r)
            print(f"\n=== {variante} #{i} (salida {r['salida']}, limpia: {r['limpia']}"
                  f"{', intrusos: ' + ', '.join(r['intrusos']) if r['intrusos'] else ''}) ===")
            for e in r["episodios"]:
                print("  " + json.dumps(e, ensure_ascii=False))
    with open("/tmp/foco-medido.json", "w") as f:
        json.dump(todo, f, indent=2, ensure_ascii=False)
    print("\ncrudo en /tmp/foco-medido.json")


main()
