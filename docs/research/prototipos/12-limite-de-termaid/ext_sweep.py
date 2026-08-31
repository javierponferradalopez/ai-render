import json, itertools
from external_layout import build, layers_of, best_order, man
from termaid.renderer.draw import render_graph
from analyze import analyze

print(f"{'caso':9} {'n':>3} {'e':>3} | {'gap':>3} {'padx':>4} {'pady':>4} | {'perd':>4} {'pared':>5} {'huerf':>5} {'suelt':>5} | {'total':>5} | tamaño")
print("-"*88)
summary={}
for tag in ["n06_bare","n08_bare","n10_bare","n12_bare","n14_bare","n17_bare","n20_bare"]:
    nodes, edges = man[tag]["nodes"], man[tag]["edges"]
    lo, _ = best_order(layers_of(nodes, edges), edges)
    pos = {n: (pi, li) for li, lay in enumerate(lo) for pi, n in enumerate(lay)}
    best=None
    for gap, padx, pady in itertools.product([1,2,3,4,6,8],[1,2,4],[0,1]):
        g = build(nodes, edges); g.grid_positions = dict(pos)
        txt = render_graph(g, padding_x=padx, padding_y=pady, gap=gap)
        r = analyze(txt, edges, set(nodes))
        bad = -r["extra_marks"] + r["corrupt_walls"] + r["orphans"] + len(r["unreachable"])
        key=(bad, r["size"][0]*r["size"][1])
        if best is None or key < best[0]:
            best=(key,gap,padx,pady,r,txt)
    (bad,_),gap,padx,pady,r,txt = best
    open(f"cases/{tag}.extbest.txt","w").write(txt)
    summary[tag]={"gap":gap,"padx":padx,"pady":pady,"bad":bad,
                  "perd":-r["extra_marks"],"pared":r["corrupt_walls"],
                  "huerf":r["orphans"],"suelt":len(r["unreachable"]),"size":r["size"]}
    print(f"{tag:9} {len(nodes):3} {len(edges):3} | {gap:3} {padx:4} {pady:4} | "
          f"{-r['extra_marks']:4} {r['corrupt_walls']:5} {r['orphans']:5} {len(r['unreachable']):5} | {bad:5} | {r['size'][0]}x{r['size'][1]}")
json.dump(summary, open("cases/ext_best.json","w"), indent=1)
