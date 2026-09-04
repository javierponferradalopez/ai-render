#!/usr/bin/env python3
"""El Límite honesto careado contra el banco, caso a caso.

Corre `flipchart check` —la tubería de verdad, sin ventana— sobre el banco del
prototipo 21 y sobre las sondas de aquí, y compara el desenlace con lo que
`esperado.tsv` dice que ese caso es. De ahí salen los tres números que deciden
si la regla del nodo rastreable vive: **falsos positivos entre los 42 casos
correctos**, **inventos cazados**, y qué regla saltó en cada rechazo.

Uso: python3 careo.py [ruta/al/flipchart]
"""

import subprocess
import sys
from collections import Counter
from pathlib import Path

AQUI = Path(__file__).parent
BANCO = AQUI.parent / "21-lo-que-mmdr-traga"
CORPUS = {"cases": BANCO / "cases", "familias": BANCO / "familias", "sondas": AQUI / "sondas"}
BINARIO = AQUI.parent.parent.parent.parent / "target" / "release" / "flipchart"

# Las dos reglas del Límite honesto se informan juntas y en el mismo rechazo, así
# que la única forma de saber cuál saltó es la causa que el mensaje da por nodo.
CAUSAS = {
    "— not in your source": "rastreable",
    "— only used in a relation": "asimetria",
}


def esperado():
    filas = {}
    for linea in (AQUI / "esperado.tsv").read_text().splitlines():
        if not linea.strip() or linea.startswith("#"):
            continue
        corpus, caso, banco, censo, valido, espera = linea.split("\t")
        filas[(corpus, caso)] = {
            "banco": banco == "si",
            "censo": censo,
            "valido": valido == "si",
            "espera": espera,
        }
    return filas


def corrido(binario, corpus):
    """Los bloques que `check` imprime, uno por fichero: desenlace y su texto."""
    fuentes = sorted(CORPUS[corpus].glob("*.mmd"))
    salida = subprocess.run(
        [str(binario), "check", *[str(f) for f in fuentes]],
        capture_output=True,
        text=True,
        check=True,
    ).stdout

    bloques, caso = {}, None
    for linea in salida.splitlines():
        if linea.startswith("== "):
            caso = Path(linea[3:]).stem
            bloques[caso] = []
        elif caso:
            bloques[caso].append(linea)
    return {caso: (lineas[0], lineas[1:]) for caso, lineas in bloques.items()}


def reglas_que_saltaron(cuerpo):
    saltaron = Counter()
    for linea in cuerpo:
        for marca, regla in CAUSAS.items():
            if marca in linea:
                saltaron[regla] += 1
    return saltaron


def main():
    binario = Path(sys.argv[1]) if len(sys.argv) > 1 else BINARIO
    if not binario.exists():
        sys.exit(f"no está el binario en {binario} — corre `make build` en la raíz")

    tabla = esperado()
    filas = []
    for corpus in CORPUS:
        for caso, (desenlace, cuerpo) in corrido(binario, corpus).items():
            ficha = tabla.get((corpus, caso))
            if ficha is None:
                sys.exit(f"{corpus}/{caso} no está en esperado.tsv")
            hubo = "dibujo" if desenlace == "drawn" else "rechazo"
            filas.append(
                {
                    "corpus": corpus,
                    "caso": caso,
                    **ficha,
                    "desenlace": desenlace,
                    "hubo": hubo,
                    "reglas": reglas_que_saltaron(cuerpo),
                    "ids": [linea.strip() for linea in cuerpo if "—" in linea],
                }
            )

    ancho = max(len(f"{f['corpus']}/{f['caso']}") for f in filas)
    for fila in filas:
        acuerdo = (
            "  "
            if fila["espera"] == "libre" or fila["espera"] == fila["hubo"]
            else ("FP" if fila["hubo"] == "rechazo" else "FN")
        )
        reglas = "+".join(sorted(fila["reglas"])) if fila["reglas"] else ""
        nombre = f"{fila['corpus']}/{fila['caso']}"
        print(
            f"{acuerdo}  {nombre:<{ancho}}  {fila['censo']:<11} "
            f"espera {fila['espera']:<8} hubo {fila['hubo']:<8} {reglas}"
        )

    banco = [f for f in filas if f["banco"]]
    correctos = [f for f in banco if f["censo"] == "correcto"]
    inventos = [f for f in banco if f["censo"] == "invento"]
    sondas = [f for f in filas if f["corpus"] == "sondas"]

    print()
    print(f"banco                   {len(banco)} casos")
    falsos = [f for f in correctos if f["hubo"] == "rechazo"]
    print(f"falsos positivos        {len(falsos)} de {len(correctos)} casos correctos")
    for fila in falsos:
        print(f"    {fila['corpus']}/{fila['caso']}: {' / '.join(fila['ids'])}")
    cazados = [f for f in inventos if f["hubo"] == "rechazo"]
    print(f"inventos cazados        {len(cazados)} de {len(inventos)}")
    for fila in inventos:
        if fila["hubo"] != "rechazo":
            print(f"    escapa {fila['corpus']}/{fila['caso']}")
    fugas = [f for f in banco if f["censo"] == "fuga" and f["hubo"] == "rechazo"]
    print(f"fugas rechazadas        {len(fugas)} (§4 pide dibujarlas y avisar)")
    for fila in fugas:
        print(f"    {fila['corpus']}/{fila['caso']}: {' / '.join(fila['ids'])}")

    print()
    for etiqueta, grupo in (("banco", banco), ("sondas", sondas)):
        cuenta = Counter()
        for fila in grupo:
            for regla, veces in fila["reglas"].items():
                cuenta[regla] += veces
        rechazos = sum(1 for f in grupo if f["hubo"] == "rechazo")
        detalle = ", ".join(f"{regla} {veces} nodos" for regla, veces in sorted(cuenta.items()))
        print(f"{etiqueta:<8} {rechazos} rechazos — {detalle}")

    print()
    desacuerdos = [f for f in sondas if f["espera"] != f["hubo"]]
    print(f"sondas en desacuerdo    {len(desacuerdos)} de {len(sondas)}")
    for fila in desacuerdos:
        print(f"    {fila['caso']}: {' / '.join(fila['ids'])}")


if __name__ == "__main__":
    main()
