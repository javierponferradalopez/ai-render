"""Puntúa los layouts del barrido contra el criterio de éxito del ticket #25.

Lee los `*.layout.json` que emite `sweep` y calcula, por combinación de
configuración y caso:

  - las tres patologías de research 07/08 (perdidas, cruces, sueltas), que son
    la salud: si una configuración miente, se descarta antes de mirar nada más;
  - `rodeos`: aristas que se salen del corredor de sus dos extremos para entrar
    por el lado contrario, que es la queja literal de research 08 §6;
  - `desvio`: el peor cociente entre la longitud de la polilínea y la distancia
    en línea recta entre los centros de sus nodos;
  - `vacio`: fracción de bandas horizontales del lienzo sin un solo nodo — la
    "banda vacía en el medio";
  - `aspecto`: ancho/alto.
"""
import json, math, os, re, sys
from collections import defaultdict

CLASS_ARROWS = ("<|--", "*--", "o--", "..>", "-->", "--")
FLOW_ARROW = re.compile(r"^\s*(\w+)\s*-[.-]*[^>]*-*>\s*(\w+)\s*$")


def source_edges(path):
    """Las relaciones tal y como las escribió el fuente. La verdad."""
    out = []
    for line in open(path):
        line = line.split("%%")[0].strip()
        if not line or line.startswith(("classDiagram", "flowchart", "graph", "subgraph", "end", "class ")):
            continue
        for a in CLASS_ARROWS:
            if f" {a} " in line:
                s, d = line.split(f" {a} ", 1)
                s, d = s.strip(), d.strip().split(":")[0].strip()
                if s and d:
                    out.append((s, d))
                break
        else:
            m = FLOW_ARROW.match(line)
            if m:
                out.append((m.group(1), m.group(2)))
    return out


def seg_rect_cross(p, q, r, margin=1.0):
    """El segmento p-q atraviesa el interior del rect r. Liang-Barsky."""
    x0, y0, x1, y1 = r[0] + margin, r[1] + margin, r[2] - margin, r[3] - margin
    dx, dy = q[0] - p[0], q[1] - p[1]
    t0, t1 = 0.0, 1.0
    for pp, qq in ((-dx, p[0] - x0), (dx, x1 - p[0]), (-dy, p[1] - y0), (dy, y1 - p[1])):
        if pp == 0:
            if qq < 0:
                return False
        else:
            t = qq / pp
            if pp < 0:
                if t > t1:
                    return False
                t0 = max(t0, t)
            else:
                if t < t0:
                    return False
                t1 = min(t1, t)
    return t0 < t1


def polyline_len(pts):
    return sum(math.dist(pts[i], pts[i + 1]) for i in range(len(pts) - 1))


def analyze(layout_path, case_path, rodeo_margin=40.0):
    d = json.load(open(layout_path))
    src = source_edges(case_path)
    nodes = {n["id"]: n for n in d["nodes"] if not n.get("hidden")}
    rects = {i: (n["x"], n["y"], n["x"] + n["width"], n["y"] + n["height"]) for i, n in nodes.items()}
    centers = {i: (n["x"] + n["width"] / 2, n["y"] + n["height"] / 2) for i, n in nodes.items()}
    drawn = {(e["from"], e["to"]) for e in d["edges"]}

    perdidas = sum(1 for s, t in src if (s, t) not in drawn and (t, s) not in drawn)
    sueltas = sum(
        1
        for nid in rects
        if any(nid in st for st in src) and not any(nid in (e["from"], e["to"]) for e in d["edges"])
    )

    cruces = rodeos = 0
    desvio = 1.0
    for e in d["edges"]:
        pts = e["points"]
        if len(pts) < 2:
            continue
        for i in range(len(pts) - 1):
            for nid, r in rects.items():
                if nid in (e["from"], e["to"]):
                    continue
                if seg_rect_cross(pts[i], pts[i + 1], r):
                    cruces += 1
                    break
            else:
                continue
            break
        # Rodeo: la polilínea se sale del corredor que forman sus dos extremos.
        a, b = rects.get(e["from"]), rects.get(e["to"])
        if a and b:
            xlo, xhi = min(a[0], b[0]) - rodeo_margin, max(a[2], b[2]) + rodeo_margin
            ylo, yhi = min(a[1], b[1]) - rodeo_margin, max(a[3], b[3]) + rodeo_margin
            if any(x < xlo or x > xhi or y < ylo or y > yhi for x, y in pts):
                rodeos += 1
        ca, cb = centers.get(e["from"]), centers.get(e["to"])
        if ca and cb:
            recta = math.dist(ca, cb)
            if recta > 1:
                desvio = max(desvio, polyline_len(pts) / recta)

    # Bandas horizontales sin ningún nodo dentro.
    w, h = d["width"], d["height"]
    banda = 16.0
    filas = max(1, int(h / banda))
    ocupadas = set()
    for n in nodes.values():
        for f in range(int(n["y"] / banda), int((n["y"] + n["height"]) / banda) + 1):
            ocupadas.add(f)
    vacio = 1 - len(ocupadas & set(range(filas))) / filas

    subs = [(s["label"], s["y"], s["x"]) for s in d.get("subgraphs", [])]
    orden = ",".join(l for l, _, _ in sorted(subs, key=lambda t: (round(t[1]), t[2])))

    return dict(
        nodos=len(nodes), aristas=len(src), perdidas=perdidas, cruces=cruces, sueltas=sueltas,
        rodeos=rodeos, desvio=round(desvio, 2), vacio=round(vacio, 3),
        w=round(w), h=round(h), aspecto=round(w / h, 2), orden=orden,
    )


def main(out_dir, cases_dir, casos):
    filas = []
    for cfg in sorted(os.listdir(out_dir)):
        cdir = os.path.join(out_dir, cfg)
        if not os.path.isdir(cdir):
            continue
        for caso in casos:
            lp = os.path.join(cdir, f"{caso}.layout.json")
            if not os.path.exists(lp):
                continue
            m = analyze(lp, os.path.join(cases_dir, f"{caso}.mmd"))
            m["config"], m["caso"] = cfg, caso
            filas.append(m)
    print(json.dumps(filas))


if __name__ == "__main__":
    out_dir, cases_dir = sys.argv[1], sys.argv[2]
    casos = sys.argv[3:] or [f[:-4] for f in sorted(os.listdir(cases_dir)) if f.endswith(".mmd")]
    main(out_dir, cases_dir, casos)
