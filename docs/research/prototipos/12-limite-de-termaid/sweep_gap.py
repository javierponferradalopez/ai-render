import json, subprocess, itertools
from analyze import analyze
man = json.load(open("cases/manifest.json"))
cases = [t for t in sorted(man) if t.endswith("_mem")]
print(f"{'caso':10} {'gap':>3} {'pady':>4} {'arist':>5} {'perd':>4} {'pared':>5} {'huerf':>5} {'sueltas':>7}  tamaño")
print("-"*62)
best = {}
for tag in cases:
    for gap, pady in itertools.product([1,2,4,6,8],[0,1,2]):
        out = subprocess.run(["./venv/bin/python","-m","termaid",f"cases/{tag}.mmd",
                              "--gap",str(gap),"--padding-y",str(pady)],
                             capture_output=True, text=True)
        r = analyze(out.stdout, man[tag]["edges"], set(man[tag]["nodes"]))
        bad = -r["extra_marks"] + r["corrupt_walls"] + r["orphans"] + len(r["unreachable"])
        key = (bad, r["size"][0]*r["size"][1])
        if tag not in best or key < best[tag][0]:
            best[tag] = (key, gap, pady, r)
for tag in cases:
    (bad,_), gap, pady, r = best[tag]
    print(f"{tag:10} {gap:3} {pady:4} {r['edges']:5} {-r['extra_marks']:4} "
          f"{r['corrupt_walls']:5} {r['orphans']:5} {len(r['unreachable']):7}  "
          f"{r['size'][0]}x{r['size'][1]}   (mejor de 15 combinaciones)")
json.dump({t:{"gap":b[1],"pady":b[2],"r":{k:v for k,v in b[3].items() if k!='corrupt_sample'}} for t,b in best.items()},
          open("cases/best_gap.json","w"), indent=1, default=str)
