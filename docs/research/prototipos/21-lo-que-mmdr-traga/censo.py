#!/usr/bin/env python3
"""El careo: las palabras del fuente contra las palabras de la pantalla.

Un constructo tragado y no dibujado no da error, así que hay que verlo en el
resultado. Este script hace la comparación que el ticket #27 propone —lo que el
`Graph` trae tras el parse contra lo que aparece en el SVG— por el lado barato:
**cada palabra del fuente tiene que salir en el SVG o estar en la lista de
palabras que son sintaxis**. Lo que no cumple ninguna de las dos es una fuga.

Uso: python3 censo.py out cases   # o `out-familias familias`
"""

import json
import re
import sys
from pathlib import Path

# Palabras que son sintaxis de Mermaid, no significado del autor: nadie espera
# verlas dibujadas. Todo lo demás que desaparece es una fuga.
SINTAXIS = {
    # cabeceras
    "flowchart", "graph", "classdiagram", "statediagram", "statediagram-v2",
    "sequencediagram", "erdiagram", "pie", "mindmap", "journey", "timeline",
    "gantt", "requirementdiagram", "gitgraph", "c4context", "sankey-beta",
    "quadrantchart", "zenuml", "block-beta", "packet-beta", "kanban",
    "architecture-beta", "radar-beta", "treemap-beta", "xychart-beta",
    # estructura
    "subgraph", "end", "direction", "class", "namespace", "state", "note",
    "for", "over", "left", "right", "participant", "alt", "else", "opt",
    "loop", "par", "box", "section", "title", "root", "commit", "branch",
    "checkout", "merge", "requirement", "element", "satisfies", "axis",
    "curve", "bar", "line", "columns", "group", "service", "in",
    # decoradores y canales de estilo/enlace
    "click", "link", "cssclass", "style", "linkstyle", "classdef",
    "acctitle", "accdescr", "shape", "label", "href", "call",
    # tokens de sintaxis de campos
    "dateformat", "x-axis", "y-axis", "quadrant-1", "quadrant-2",
    "quadrant-3", "quadrant-4", "id", "text", "risk", "verifymethod",
    "type", "high", "test", "simulation",
    # tipos y modificadores que Mermaid escribe pero que son del idioma
    "string", "int", "float", "bool",
}

PALABRA = re.compile(r"[A-Za-zÁÉÍÓÚÑáéíóúñ][A-Za-z0-9ÁÉÍÓÚÑáéíóúñ_]{2,}")
TEXTO_SVG = re.compile(r"<text\b[^>]*>(.*?)</text>", re.S)
ETIQUETA = re.compile(r"<[^>]*>")
COMENTARIO = re.compile(r"^\s*%%.*$", re.M)


def palabras(texto):
    return {m.group(0).lower() for m in PALABRA.finditer(texto)}


def palabras_svg(svg):
    # Sólo lo que hay *dentro* de un <text>, y sin los <tspan> de dentro: los
    # atributos del SVG no son palabras que nadie haya escrito.
    dentro = " ".join(ETIQUETA.sub(" ", t) for t in TEXTO_SVG.findall(svg))
    dentro = dentro.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
    dentro = dentro.replace("&quot;", '"').replace("&#39;", "'")
    return palabras(dentro)


def main():
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "out")
    casos = Path(sys.argv[2] if len(sys.argv) > 2 else "cases")
    censo = json.loads((out / "censo.json").read_text())

    filas = []
    for entrada in censo:
        nombre = entrada["caso"]
        fuente = (casos / f"{nombre}.mmd").read_text()
        # Los comentarios `%%` se descartan a propósito: no son significado.
        fuente = COMENTARIO.sub("", fuente)
        del_fuente = palabras(fuente) - SINTAXIS

        if entrada["parse"] == "Err":
            filas.append({
                "caso": nombre, "veredicto": "RECHAZO",
                "detalle": entrada["error"], "perdidas": [],
            })
            continue
        if entrada.get("render") == "PANIC":
            filas.append({
                "caso": nombre, "veredicto": "PANICO",
                "detalle": "el renderer se llevó el proceso", "perdidas": [],
            })
            continue

        # Un id con etiqueta propia no se dibuja, y eso es correcto: lo que
        # sale en pantalla es la etiqueta. Sólo cuenta como fuga el id desnudo.
        tapados = {
            n["id"].lower()
            for n in entrada["grafo"]["nodes"]
            if n["label"] and n["label"] != n["id"]
        }

        svg = (out / f"{nombre}.svg").read_text()
        en_pantalla = palabras_svg(svg)
        perdidas = sorted(del_fuente - en_pantalla - tapados)

        # Lo contrario: palabras dibujadas que el autor no escribió.
        inventadas = sorted(w for w in en_pantalla - palabras(fuente) if w not in SINTAXIS)

        if perdidas and inventadas:
            veredicto = "FUGA+INVENTO"
        elif perdidas:
            veredicto = "FUGA"
        elif inventadas:
            veredicto = "INVENTO"
        else:
            veredicto = "ok"

        filas.append({
            "caso": nombre,
            "veredicto": veredicto,
            "perdidas": perdidas,
            "inventadas": inventadas,
            "nodos": [n["id"] for n in entrada["grafo"]["nodes"]],
            "subgrafos": [s["label"] for s in entrada["grafo"]["subgraphs"]],
            "canales": sorted(entrada["grafo"]["side_channels"].keys()),
            "estilo_vaciado": entrada.get("estilo_vaciado", []),
        })

    ancho = max(len(f["caso"]) for f in filas)
    for f in filas:
        extra = ""
        if f["veredicto"] in ("FUGA", "FUGA+INVENTO"):
            extra = "  pierde: " + ", ".join(f["perdidas"])
        if f["veredicto"] in ("INVENTO", "FUGA+INVENTO"):
            extra += "  inventa: " + ", ".join(f["inventadas"])
        if f["veredicto"] == "RECHAZO":
            extra = "  " + f["detalle"]
        if f["veredicto"] == "PANICO":
            extra = "  " + f["detalle"]
        print(f"{f['caso']:<{ancho}}  {f['veredicto']:<12}{extra}")

    print()
    print(json.dumps(filas, ensure_ascii=False, indent=2), file=sys.stderr)


if __name__ == "__main__":
    main()
