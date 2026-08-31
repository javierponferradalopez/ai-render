"""Extrae un grafo de clases real del paquete termaid usando ast."""
import ast, os, json, sys

ROOT = sys.argv[1]

classes = {}   # name -> {module, methods[], attrs[], bases[]}
edges = []     # (src, dst, kind)

def ann_names(node):
    """Nombres de tipo que aparecen en una anotación."""
    out = []
    for n in ast.walk(node):
        if isinstance(n, ast.Name):
            out.append(n.id)
        elif isinstance(n, ast.Attribute):
            out.append(n.attr)
        elif isinstance(n, ast.Constant) and isinstance(n.value, str):
            out.append(n.value.strip("'\"[] "))
    return out

for dirpath, _, files in os.walk(ROOT):
    for f in files:
        if not f.endswith(".py"):
            continue
        path = os.path.join(dirpath, f)
        mod = os.path.relpath(path, ROOT).replace(".py", "").replace("/", ".")
        try:
            tree = ast.parse(open(path, encoding="utf-8").read())
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.ClassDef):
                continue
            methods, attrs = [], []
            for b in node.body:
                if isinstance(b, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    if b.name.startswith("__"):
                        continue
                    args = [a.arg for a in b.args.args if a.arg != "self"]
                    vis = "-" if b.name.startswith("_") else "+"
                    methods.append((vis, b.name, args, b))
                elif isinstance(b, ast.AnnAssign) and isinstance(b.target, ast.Name):
                    attrs.append((b.target.id, b.annotation))
            classes[node.name] = {
                "module": mod,
                "layer": mod.split(".")[0] if "." in mod else mod,
                "methods": methods,
                "attrs": attrs,
                "bases": [b.id for b in node.bases if isinstance(b, ast.Name)],
                "node": node,
            }

# aristas
for name, c in classes.items():
    for b in c["bases"]:
        if b in classes:
            edges.append((name, b, "extends"))
    # atributos tipados -> contención / asociación
    for attr, ann in c["attrs"]:
        for t in ann_names(ann):
            if t in classes and t != name:
                edges.append((name, t, "attr"))
    # tipos usados en firmas de métodos -> dependencia
    for vis, mname, args, fn in c["methods"]:
        for a in fn.args.args:
            if a.annotation is not None:
                for t in ann_names(a.annotation):
                    if t in classes and t != name:
                        edges.append((name, t, "depends"))
        if fn.returns is not None:
            for t in ann_names(fn.returns):
                if t in classes and t != name:
                    edges.append((name, t, "depends"))

# dedup, prioridad extends > attr > depends
prio = {"extends": 0, "attr": 1, "depends": 2}
best = {}
for s, d, k in edges:
    key = (s, d)
    if key not in best or prio[k] < prio[best[key]]:
        best[key] = k
edges = [(s, d, k) for (s, d), k in best.items()]

out = {
    "classes": {n: {"module": c["module"], "layer": c["layer"],
                    "methods": [(v, m, a) for v, m, a, _ in c["methods"]],
                    "attrs": [a for a, _ in c["attrs"]]}
                for n, c in classes.items()},
    "edges": edges,
}
print(json.dumps(out, indent=1))
