"""Aísla la variable: ¿es el número de nodos o la anchura de las etiquetas?"""
import subprocess, itertools

def src(names, members):
    s = "classDiagram\n  class Base {\n    +run()\n  }\n"
    for n in names:
        s += f"  class {n} {{\n" + "".join(f"    +{m}\n" for m in members) + "  }\n"
    for n in names:
        s += f"  Base <|-- {n}\n"
    return s

SETS = {
 "cortos, uniformes":      (["Sub0","Sub1","Sub2","Sub3"], ["run()"]),
 "cortos, desiguales":     (["A","Bbbbbbbbbbbbbbbb","Cc","Dddddddd"], ["run()"]),
 "realistas, uniformes":   (["PidfdChildWatcher","BaseChildWatcher","FastChildWatcher","SafeChildWatcher"], ["is_active()"]),
 "realistas, desiguales":  (["PidfdChildWatcher","BaseChildWatcher","MultiLoopChildWatcher","ThreadedChildWatcher"],
                            ["add_child_handler(pid, cb)"]),
}
print(f"{'caso':24} {'nodos':>5} {'arist':>5} | {'△ dibujados (mejor de 32)':>25} | anchura máx caja")
print("-"*90)
for label,(names,members) in SETS.items():
    best=(-1,None,None)
    for direction, gap, pady in itertools.product(["","LR","TB","RL"],[1,2,4,6],[0,1]):
        s = src(names, members)
        if direction: s = s.replace("classDiagram\n", f"classDiagram\n  direction {direction}\n",1)
        open("/tmp/w.mmd","w").write(s)
        out = subprocess.run(["./venv/bin/python","-m","termaid","/tmp/w.mmd",
                              "--gap",str(gap),"--padding-y",str(pady)],
                             capture_output=True,text=True).stdout
        if out.count("△") > best[0]:
            best=(out.count("△"), out, (direction,gap,pady))
    w = max(len(n) for n in names+members)
    print(f"{label:24} {len(names)+1:5} {len(names):5} | {best[0]:>10} de {len(names):<12} | {w}")
    open(f"cases/width_{label.split(',')[0]}_{len(names)}.txt","w").write(best[1])
print()
print("=== realistas, desiguales — mejor render posible ===")
print(open("cases/width_realistas_4.txt").read())
