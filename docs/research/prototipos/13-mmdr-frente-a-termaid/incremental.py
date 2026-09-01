"""Determinismo y estabilidad del layout ante un cambio incremental.

Una pizarra en vivo se actualiza: si al añadir un nodo salta todo el diagrama,
el usuario pierde el hilo. Mide cuantos nodos preexistentes cambian de posicion.
"""
import hashlib, json, subprocess, sys

MMDR = sys.argv[1] if len(sys.argv) > 1 else "mmdr"
BASE = sys.argv[2] if len(sys.argv) > 2 else "cases/n06_mem.mmd"

def render(mmd, svg):
    subprocess.run([MMDR, "-i", mmd, "-o", svg, "-e", "svg"], capture_output=True, check=True)
    return hashlib.sha256(open(svg, "rb").read()).hexdigest()

def layout(mmd):
    out = mmd.replace(".mmd", ".layout.json")
    subprocess.run([MMDR, "-i", mmd, "--dumpLayout", out, "-o", "/dev/null", "-e", "svg"],
                   capture_output=True, check=True)
    d = json.load(open(out))
    return {n["id"]: (round(n["x"], 1), round(n["y"], 1)) for n in d["nodes"]}, d["width"], d["height"]

hashes = {render(BASE, f"/tmp/det{i}.svg") for i in range(5)}
print(f"determinismo: {len(hashes)} hash distinto(s) en 5 renders del mismo fuente")

plus = open(BASE).read().rstrip() + "\n  class Renderer {\n      +draw(g)\n  }\n  Renderer ..> Graph\n"
open("/tmp/inc_plus.mmd", "w").write(plus)
a, aw, ah = layout(BASE)
b, bw, bh = layout("/tmp/inc_plus.mmd")
movidos = [k for k in a if k in b and a[k] != b[k]]
print(f"lienzo: {int(aw)}x{int(ah)} -> {int(bw)}x{int(bh)}")
print(f"nodos preexistentes: {len(a)} | movidos al anadir uno: {len(movidos)}")
for k in movidos:
    print(f"   {k:20} {a[k]} -> {b[k]}")
