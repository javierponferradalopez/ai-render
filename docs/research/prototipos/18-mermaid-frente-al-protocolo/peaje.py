#!/usr/bin/env python3
"""Peaje fijo en tokens: claude-mermaid real, contra un flipchart que trague Mermaid.

Metodo: cl100k_base via tiktoken, el mismo tokenizador que usaron
docs/research/04-mcp-de-tldraw.md y el comentario de medicion del issue #10,
para que las cifras sean comparables entre si.

Uso:  pip install tiktoken && ./peaje.py
      (claude-mermaid-tools.json es la respuesta real de tools/list,
       capturada con probe.mjs)
"""
import json, pathlib, tiktoken

enc = tiktoken.get_encoding("cl100k_base")
n = lambda s: len(enc.encode(s))
here = pathlib.Path(__file__).parent


def report(label, tools):
    print(f"\n{label}: {n(json.dumps({'tools': tools}))} tokens")
    for t in tools:
        schema = json.dumps(t.get("inputSchema"))
        print(f"   {t['name']:<20} total {n(json.dumps(t)):>4}"
              f"   desc {n(t['description']):>4}   schema {n(schema):>4}")


report("claude-mermaid 1.6.5 (medido)", json.loads((here / "claude-mermaid-tools.json").read_text()))

# Flipchart tragando Mermaid: dos herramientas, sin schema de nodos ni aristas.
# Es la variante que el issue #15 eligio, escrita con descripciones escuetas
# equivalentes a las de la fila "Descripciones escuetas" del issue #10.
flipchart = [
    {
        "name": "flipchart_show",
        "description": (
            "Show a diagram on the ephemeral flipchart window, as a named view. "
            "Takes Mermaid source. Showing an existing view id replaces it; several "
            "named views coexist on screen. The flipchart dies with the session."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "view_id": {"type": "string", "description": "Name of the view. Reusing a name replaces that view."},
                "diagram": {"type": "string", "description": "Mermaid source."},
            },
            "required": ["view_id", "diagram"],
        },
    },
    {
        "name": "flipchart_clear",
        "description": "Remove one view from the flipchart, or all of them.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "view_id": {"type": "string", "description": "View to remove. Omit to clear the whole flipchart."}
            },
            "required": [],
        },
    },
]
report("flipchart con Mermaid (2 tools)", flipchart)

print("\nReferencias del mapa, para comparar:")
print("   protocolo propio, 3 tools, descripciones escuetas ....  738  (issue #10)")
print("   protocolo propio, con guia de uso y ejemplo ..........  1047  (issue #10)")
print("   tldraw, 2 tools .....................................  ~900  (research 04)")

print("\nBreak-even contra el protocolo propio, con los payloads del issue #10")
print("(show propio 260 / retoque 43; show Mermaid 151 / retoque 151):")
fijo_flip = n(json.dumps({"tools": flipchart}))
for guia, fijo_propio in (("escueto", 738), ("con guia", 1047)):
    print(f"   {guia:<9} delta = {fijo_propio + 260 - fijo_flip - 151} - 108k"
          f"   ->  break-even k = {(fijo_propio + 260 - fijo_flip - 151) / 108:.1f} retoques")
