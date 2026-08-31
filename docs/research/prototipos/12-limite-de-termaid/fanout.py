"""¿Alguna configuración dibuja bien una jerarquía de herencia simple?"""
import subprocess, itertools
base = """classDiagram
{dir}  class Base {{
    +run()
  }}
"""
def src(k, direction=""):
    s = "classDiagram\n"
    if direction:
        s += f"  direction {direction}\n"
    s += "  class Base {\n    +run()\n  }\n"
    for i in range(k):
        s += f"  class Sub{i} {{\n    +run()\n  }}\n"
    for i in range(k):
        s += f"  Base <|-- Sub{i}\n"
    return s

print(f"{'hijos':>5} {'dir':>4} {'gap':>3} {'pady':>4} | {'△ dibujados':>11} | veredicto")
print("-"*58)
for k in [2,3,4,5,6]:
    rows=[]
    for direction, gap, pady in itertools.product(["","LR","TB","RL"],[1,2,4,6],[0,1]):
        open("/tmp/f.mmd","w").write(src(k, direction))
        out = subprocess.run(["./venv/bin/python","-m","termaid","/tmp/f.mmd",
                              "--gap",str(gap),"--padding-y",str(pady)],
                             capture_output=True, text=True).stdout
        n = out.count("△")
        rows.append((n, direction or "-", gap, pady, out))
    best = max(rows, key=lambda r: r[0])
    ok = "OK" if best[0]==k else f"PIERDE {k-best[0]} de {k}"
    print(f"{k:5} {best[1]:>4} {best[2]:3} {best[3]:4} | {best[0]:>5} de {k:<3} | {ok}   (mejor de 32 combinaciones)")
    if k==4:
        open("cases/fanout4_best.txt","w").write(best[4])
print()
print("=== mejor render posible de Base + 4 subclases ===")
print(open("cases/fanout4_best.txt").read())
