"""Genera classDiagram Mermaid a partir de subgrafos conexos del grafo real."""
import json, sys
from collections import defaultdict, deque

g = json.load(open("graph.json"))
CLS, EDGES = g["classes"], g["edges"]

adj = defaultdict(set)
for s, d, k in EDGES:
    adj[s].add(d); adj[d].add(s)

def subgraph(seed, n):
    """BFS desde seed, prefiriendo vecinos de mayor grado (subgrafo denso)."""
    seen, order = {seed}, [seed]
    q = deque([seed])
    while q and len(order) < n:
        cur = q.popleft()
        for nb in sorted(adj[cur], key=lambda x: (-len(adj[x]), x)):
            if nb not in seen and len(order) < n:
                seen.add(nb); order.append(nb); q.append(nb)
    return order

ARROW = {"extends": "<|--", "attr": "*--", "depends": "..>"}

def to_mermaid(names, members=True, maxm=3):
    ns = set(names)
    lines = ["classDiagram"]
    if members:
        for n in names:
            c = CLS[n]
            body = []
            for a in c["attrs"][:2]:
                body.append(f"    -{a}")
            for vis, m, args in c["methods"][:maxm]:
                sig = ", ".join(args[:2])
                body.append(f"    {vis}{m}({sig})")
            if body:
                lines.append(f"  class {n} {{")
                lines += ["  " + b for b in body]
                lines.append("  }")
            else:
                lines.append(f"  class {n}")
    else:
        for n in names:
            lines.append(f"  class {n}")
    used = []
    for s, d, k in EDGES:
        if s in ns and d in ns:
            if k == "extends":
                lines.append(f"  {d} <|-- {s}")
            elif k == "attr":
                lines.append(f"  {s} *-- {d}")
            else:
                lines.append(f"  {s} ..> {d}")
            used.append((s, d, k))
    return "\n".join(lines), used

if __name__ == "__main__":
    # semilla: la clase de mayor grado
    seed = max(sorted(adj), key=lambda x: len(adj[x]))
    print("seed:", seed, "grado:", len(adj[seed]))
    import os
    os.makedirs("cases", exist_ok=True)
    manifest = {}
    for n in [3, 4, 5, 6, 7, 8, 10, 12, 14, 17, 20]:
        names = subgraph(seed, n)
        for withm in (True, False):
            src, used = to_mermaid(names, members=withm)
            tag = f"n{n:02d}_{'mem' if withm else 'bare'}"
            open(f"cases/{tag}.mmd", "w").write(src + "\n")
            manifest[tag] = {"nodes": names, "edges": used}
    json.dump(manifest, open("cases/manifest.json", "w"), indent=1)
    for k, v in manifest.items():
        print(k, len(v["nodes"]), "nodos", len(v["edges"]), "aristas")
