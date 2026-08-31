"""Detecta las patologías de ruteo de termaid sobre un render Unicode."""
import json, re, sys, subprocess
from collections import defaultdict

MARKS = set("►◄▲▼△◆◇")
LINES = set("─│┄┆┈┊")
CORNERS = set("┌┐└┘")
TEES = set("├┤┬┴┼")

def grid(text):
    rows = text.split("\n")
    w = max((len(r) for r in rows), default=0)
    return [r.ljust(w) for r in rows], w

def find_boxes(rows, w):
    """Rectángulos delimitados por ┌ ... ┐ / └ ... ┘."""
    boxes = []
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != "┌":
                continue
            # borde superior hasta ┐
            x2 = None
            for xx in range(x + 1, w):
                c = row[xx]
                if c == "┐":
                    x2 = xx; break
                if c not in ("─", "┬", "┼"):
                    break
            if x2 is None:
                continue
            # bajar por la columna izquierda hasta └
            y2 = None
            for yy in range(y + 1, len(rows)):
                c = rows[yy][x] if x < len(rows[yy]) else " "
                if c == "└":
                    y2 = yy; break
                if c not in ("│", "├", "┼", "┤"):
                    break
            if y2 is None:
                continue
            boxes.append({"x1": x, "y1": y, "x2": x2, "y2": y2})
    return boxes

def label_of(rows, b):
    """Primera línea de texto dentro de la caja = nombre de la clase."""
    if b["y1"] + 1 > b["y2"] - 1:
        return None
    inner = rows[b["y1"] + 1][b["x1"] + 1:b["x2"]].strip()
    return inner or None

def wall_cells(b):
    """Celdas del perímetro: (x, y, orientación)."""
    out = []
    for x in range(b["x1"] + 1, b["x2"]):
        out.append((x, b["y1"], "h")); out.append((x, b["y2"], "h"))
    for y in range(b["y1"] + 1, b["y2"]):
        out.append((b["x1"], y, "v")); out.append((b["x2"], y, "v"))
    return out

def analyze(text, expected_edges, node_names):
    rows, w = grid(text)
    boxes = find_boxes(rows, w)
    named = {}
    for b in boxes:
        lab = label_of(rows, b)
        if lab in node_names:
            named[lab] = b
    at = lambda x, y: rows[y][x] if 0 <= y < len(rows) and 0 <= x < w else " "

    # --- 1. marcadores totales vs aristas
    total_marks = sum(1 for r in rows for c in r if c in MARKS)
    extra = total_marks - len(expected_edges)

    # --- 2. paredes corrompidas
    corrupt = []
    for b in boxes:
        for (x, y, o) in wall_cells(b):
            c = at(x, y)
            if o == "h" and c != "─":
                corrupt.append((x, y, c, "h"))
            elif o == "v" and c not in ("│", "├", "┤"):
                corrupt.append((x, y, c, "v"))

    # --- 3. marcadores tocando cada caja vs grado real
    deg = defaultdict(int)
    for s, d, k in expected_edges:
        deg[s] += 1; deg[d] += 1
    touching = defaultdict(int)
    for name, b in named.items():
        cells = set(wall_cells(b))
        # marcador sobre la pared, o pegado justo fuera de ella
        for (x, y, o) in cells:
            if at(x, y) in MARKS:
                touching[name] += 1
            else:
                nx, ny = (x, y - 1) if (o == "h" and y == b["y1"]) else \
                         (x, y + 1) if o == "h" else \
                         (x - 1, y) if x == b["x1"] else (x + 1, y)
                if at(nx, ny) in MARKS:
                    touching[name] += 1
    mismatch = {n: (touching[n], deg[n]) for n in named if touching[n] != deg[n]}
    unreachable = [n for n in named if deg[n] > 0 and touching[n] == 0]
    false_in = {n: touching[n] - deg[n] for n in named if touching[n] > deg[n]}

    # --- 4. fragmentos huérfanos: marcador sin línea que lo alimente
    orphans = []
    feed = {"▼": (0, -1), "▲": (0, 1), "►": (-1, 0), "◄": (1, 0)}
    for y, row in enumerate(rows):
        for x, c in enumerate(row):
            if c in feed:
                dx, dy = feed[c]
                back = at(x + dx, y + dy)
                if back == " ":
                    orphans.append((x, y, c))

    return {
        "boxes": len(boxes), "named": len(named),
        "edges": len(expected_edges), "marks": total_marks, "extra_marks": extra,
        "corrupt_walls": len(corrupt), "corrupt_sample": corrupt[:5],
        "false_incoming": false_in, "mismatch": mismatch, "unreachable": unreachable,
        "missing_boxes": sorted(node_names - set(named)),
        "orphans": len(orphans), "orphan_sample": orphans[:5],
        "size": (w, len(rows)),
    }

if __name__ == "__main__":
    man = json.load(open("cases/manifest.json"))
    results = {}
    for tag in sorted(man):
        p = f"cases/{tag}.mmd"
        out = subprocess.run(["./venv/bin/python", "-m", "termaid", p,
                              "--gap", "1", "--padding-y", "0"],
                             capture_output=True, text=True)
        text = out.stdout
        open(f"cases/{tag}.txt", "w").write(text)
        r = analyze(text, man[tag]["edges"], set(man[tag]["nodes"]))
        results[tag] = r
    json.dump(results, open("cases/results.json", "w"), indent=1)
    hdr = f"{'caso':12} {'nodos':>5} {'arist':>5} {'dibuj':>5} {'perd':>5} {'pared':>5} {'huerf':>5} {'mal-grado':>9} {'sueltas':>7}  tamaño"
    print(hdr); print("-" * len(hdr))
    for tag in sorted(man, key=lambda t: (t.split('_')[1], t)):
        r = results[tag]
        print(f"{tag:12} {len(man[tag]['nodes']):5} {r['edges']:5} {r['marks']:5} "
              f"{-r['extra_marks']:5} {r['corrupt_walls']:5} {r['orphans']:5} "
              f"{len(r['mismatch']):9} {len(r['unreachable']):7}  {r['size'][0]}x{r['size'][1]}")
