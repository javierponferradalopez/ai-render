"""Las patologias del research 07, medidas sobre la geometria de mmdr."""
import json, os, subprocess, sys

MMDR = os.environ.get("MMDR", "./mmdr")
ARROWS = {"<|--": "extends", "*--": "attr", "..>": "depends"}

def source_edges(path):
    out = []
    for line in open(path):
        line = line.strip()
        for a in ARROWS:
            if f" {a} " in line:
                s, d = line.split(f" {a} ")
                out.append((s.strip(), d.strip(), a))
                break
    return out

def seg_rect_cross(p, q, r):
    """El segmento p-q atraviesa el interior del rect r (con margen)."""
    x0, y0, x1, y1 = r
    m = 1.0
    x0, y0, x1, y1 = x0+m, y0+m, x1-m, y1-m
    # Liang-Barsky
    dx, dy = q[0]-p[0], q[1]-p[1]
    t0, t1 = 0.0, 1.0
    for pp, qq in ((-dx, p[0]-x0), (dx, x1-p[0]), (-dy, p[1]-y0), (dy, y1-p[1])):
        if pp == 0:
            if qq < 0: return False
        else:
            t = qq/pp
            if pp < 0:
                if t > t1: return False
                t0 = max(t0, t)
            else:
                if t < t0: return False
                t1 = min(t1, t)
    return t0 < t1

def analyze(mmd):
    lay = mmd.replace(".mmd", ".layout.json")
    subprocess.run([MMDR, "-i", mmd, "--dumpLayout", lay, "-o", "/dev/null", "-e", "svg"],
                   capture_output=True, check=True)
    d = json.load(open(lay))
    src = source_edges(mmd)
    rects = {n["id"]: (n["x"], n["y"], n["x"]+n["width"], n["y"]+n["height"]) for n in d["nodes"]}
    drawn = {(e["from"], e["to"]) for e in d["edges"]}
    lost = sum(1 for s, t, _ in src if (s, t) not in drawn and (t, s) not in drawn)
    crossings = 0
    for e in d["edges"]:
        pts = e["points"]
        for i in range(len(pts)-1):
            for nid, r in rects.items():
                if nid in (e["from"], e["to"]):
                    continue
                if seg_rect_cross(pts[i], pts[i+1], r):
                    crossings += 1
                    break
    loose = sum(1 for nid in rects
                if any(nid in (s, t) for s, t, _ in src)
                and not any(nid in (e["from"], e["to"]) for e in d["edges"]))
    return len(rects), len(src), lost, crossings, loose, d["width"], d["height"]

print(f"{'caso':16} {'nodos':>5} {'arist':>5} {'perdidas':>8} {'cruces':>7} {'sueltas':>7}  {'tamaño px':>12}")
for mmd in sys.argv[1:]:
    n, e, lost, cr, loose, w, h = analyze(mmd)
    ok = "limpio" if (lost == 0 and cr == 0 and loose == 0) else "ROTO"
    print(f"{os.path.basename(mmd)[:16]:16} {n:5} {e:5} {lost:8} {cr:7} {loose:7}  {int(w):5}x{int(h):<6} {ok}")
