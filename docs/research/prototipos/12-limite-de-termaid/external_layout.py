"""Mide el techo de la salida 2: layout externo inyectado por grid_positions.

En vez de Graphviz, busca el orden de capas que MINIMIZA cruces (exhaustivo en
capas pequeñas, baricentro iterado + muestreo en las grandes). Es un límite
superior de lo que cualquier layout engine externo podría aportar.
"""
import json, itertools, random
from collections import defaultdict
from termaid.graph.model import Graph, Node, Edge, Direction
from termaid.renderer.draw import render_graph
from analyze import analyze

man = json.load(open("cases/manifest.json"))

def build(nodes, edges):
    g = Graph(direction=Direction.TB)
    for n in nodes:
        g.add_node(Node(id=n, label=n))
    for s, d, k in edges:
        g.add_edge(Edge(source=s, target=d))
    return g

def layers_of(nodes, edges):
    succ = defaultdict(list); indeg = defaultdict(int)
    for s, d, k in edges:
        succ[s].append(d); indeg[d] += 1
    layer = {n: 0 for n in nodes}
    for _ in range(len(nodes)):
        changed = False
        for s, d, k in edges:
            if layer[d] < layer[s] + 1:
                layer[d] = layer[s] + 1; changed = True
        if not changed:
            break
    by = defaultdict(list)
    for n in nodes:
        by[layer[n]].append(n)
    return [by[i] for i in sorted(by)]

def crossings(layer_order, edges):
    pos = {}
    for li, lay in enumerate(layer_order):
        for pi, n in enumerate(lay):
            pos[n] = (li, pi)
    es = [(pos[s], pos[d]) for s, d, k in edges if s in pos and d in pos]
    c = 0
    for (a1, a2), (b1, b2) in itertools.combinations(es, 2):
        if a1[0] == b1[0] and a2[0] == b2[0] and a1[0] != a2[0]:
            if (a1[1] - b1[1]) * (a2[1] - b2[1]) < 0:
                c += 1
    return c

def best_order(layer_order, edges, tries=4000, seed=7):
    rnd = random.Random(seed)
    best = [list(l) for l in layer_order]
    bc = crossings(best, edges)
    for _ in range(tries):
        cand = [list(l) for l in best]
        li = rnd.randrange(len(cand))
        if len(cand[li]) < 2:
            continue
        i, j = rnd.sample(range(len(cand[li])), 2)
        cand[li][i], cand[li][j] = cand[li][j], cand[li][i]
        cc = crossings(cand, edges)
        if cc <= bc:
            best, bc = cand, cc
    return best, bc

print(f"{'caso':8} {'nodos':>5} {'arist':>5} | {'defecto: perd/pared/huerf/sueltas':>34} | {'externo óptimo':>34} | cruces")
print("-"*125)
for tag in ["n06_bare","n08_bare","n10_bare","n12_bare","n14_bare","n17_bare","n20_bare"]:
    nodes, edges = man[tag]["nodes"], man[tag]["edges"]
    def score(txt):
        r = analyze(txt, edges, set(nodes))
        return r, f"{-r['extra_marks']:>2}/{r['corrupt_walls']:>2}/{r['orphans']:>2}/{len(r['unreachable']):>2}   ({r['size'][0]}x{r['size'][1]})"
    g1 = build(nodes, edges)
    t1 = render_graph(g1, padding_x=1, padding_y=0, gap=1)
    r1, s1 = score(t1)

    lo = layers_of(nodes, edges)
    lo, cx0 = best_order(lo, edges)
    g2 = build(nodes, edges)
    g2.grid_positions = {n: (pi, li) for li, lay in enumerate(lo) for pi, n in enumerate(lay)}
    t2 = render_graph(g2, padding_x=1, padding_y=0, gap=1)
    r2, s2 = score(t2)
    open(f"cases/{tag}.ext.txt","w").write(t2)
    print(f"{tag:8} {len(nodes):5} {len(edges):5} | {s1:>34} | {s2:>34} | {cx0}")
