"""Barrido final: por cada caso, la MEJOR de 32 configuraciones."""
import json, subprocess, itertools, sys
from analyze import analyze

def run(casedir, tags):
    man = json.load(open(f"{casedir}/manifest.json"))
    print(f"{'caso':10} {'nodos':>5} {'arist':>5} | {'cfg':>10} | {'perd':>4} {'pared':>5} {'huerf':>5} {'suelt':>5} | {'total':>5} | tamaño")
    print("-"*82)
    out_summary={}
    for tag in tags:
        if tag not in man: continue
        nodes, edges = man[tag]["nodes"], man[tag]["edges"]
        best=None
        for direction, gap, pady in itertools.product(["","LR","TB"],[1,2,4,6],[0,1]):
            s = open(f"{casedir}/{tag}.mmd").read()
            if direction:
                s = s.replace("classDiagram\n", f"classDiagram\n  direction {direction}\n",1)
            open("/tmp/s.mmd","w").write(s)
            txt = subprocess.run(["./venv/bin/python","-m","termaid","/tmp/s.mmd",
                                  "--gap",str(gap),"--padding-y",str(pady)],
                                 capture_output=True,text=True).stdout
            r = analyze(txt, edges, set(nodes))
            bad = -r["extra_marks"] + r["corrupt_walls"] + r["orphans"] + len(r["unreachable"])
            key=(bad, r["size"][0]*r["size"][1])
            if best is None or key<best[0]:
                best=(key,(direction or "def",gap,pady),r,txt)
        (bad,_),cfg,r,txt = best
        open(f"{casedir}/{tag}.best.txt","w").write(txt)
        out_summary[tag]={"cfg":cfg,"perd":-r["extra_marks"],"pared":r["corrupt_walls"],
                          "huerf":r["orphans"],"suelt":len(r["unreachable"]),
                          "unreachable":r["unreachable"],"bad":bad,"size":r["size"],
                          "n":len(nodes),"e":len(edges)}
        print(f"{tag:10} {len(nodes):5} {len(edges):5} | {cfg[0]:>3},g{cfg[1]},p{cfg[2]} | "
              f"{-r['extra_marks']:4} {r['corrupt_walls']:5} {r['orphans']:5} {len(r['unreachable']):5} | {bad:5} | {r['size'][0]}x{r['size'][1]}")
    json.dump(out_summary, open(f"{casedir}/best_summary.json","w"), indent=1)
    return out_summary

tags = [f"n{n:02d}_mem" for n in [3,4,5,6,7,8,10,12,14,17,20]]
print("### FUENTE 1 — termaid (composición + dependencia + herencia)")
run("cases", tags)
print()
print("### FUENTE 2 — asyncio stdlib (herencia pura)")
run("cases_asyncio", tags)
