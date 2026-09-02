#!/usr/bin/env python3
"""Peaje de las variantes de descripcion, para el issue #26.

Mide lo que cuesta cada trozo de ensenanza en el texto de las dos
herramientas: la linea base escueta que midio el issue #11 (204), lo que
decidio el #26, y tres controles que aislan cada pieza.

Mismo metodo que peaje.py: cl100k_base via tiktoken, para que las cifras
sean comparables con research 04 y con los issues #10, #15 y #18.

Uso:  pip install tiktoken && ./peaje-26.py
"""
import json, tiktoken
enc = tiktoken.get_encoding("cl100k_base")
n = lambda s: len(enc.encode(s))

def report(label, tools):
    print(f"\n{label}: {n(json.dumps({'tools': tools}))} tokens")
    for t in tools:
        print(f"   {t['name']:<18} total {n(json.dumps(t)):>4}"
              f"   desc {n(t['description']):>4}   schema {n(json.dumps(t['inputSchema'])):>4}")

ESCUETO = [
  {"name":"flipchart_show",
   "description":("Show a diagram on the ephemeral flipchart window, as a named view. "
     "Takes Mermaid source. Showing an existing view id replaces it; several "
     "named views coexist on screen. The flipchart dies with the session."),
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":"Name of the view. Reusing a name replaces that view."},
      "diagram":{"type":"string","description":"Mermaid source."}},
      "required":["view_id","diagram"]}},
  {"name":"flipchart_clear",
   "description":"Remove one view from the flipchart, or all of them.",
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":"View to remove. Omit to clear the whole flipchart."}},
      "required":[]}},
]

# Lo decidido en #26: listón del cuándo sí + la norma de pedir/avisar + la
# invariante de la asimetría; nada de estilo, nada de mmdr-vs-Mermaid, nada de
# cuándo no. Y el registro del view_id con un ejemplo, en el campo.
DECIDIDO = [
  {"name":"flipchart_show",
   "description":(
     "Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\n"
     "Use it when the user needs to understand a structure, or a change to one, before deciding "
     "about it. If the window is not open yet, offer it and wait for the user to accept; once it "
     "is open, just say what you are drawing.\n\n"
     "Any id used in a relationship must carry a label or a body when another id in the same "
     "diagram does; a bare id alongside a labelled one is rejected.\n\n"
     "Showing an existing view id replaces it and brings it to the front; several named views "
     "coexist. The flipchart dies with the session."),
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":"Short human-readable name, shown to the user above the diagram - e.g. \"Current dependencies\", not \"v1\". Reusing a name replaces that view."},
      "diagram":{"type":"string","description":"Mermaid source."}},
      "required":["view_id","diagram"]}},
  {"name":"flipchart_clear",
   "description":"Remove one view from the flipchart, or all of them. Does not close the window.",
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":"View to remove. Omit to clear the whole flipchart."}},
      "required":[]}},
]

report("A. Escueto (linea base de #11)", ESCUETO)
report("B. Lo decidido en #26", DECIDIDO)

# Variantes de control, para saber que compra cada trozo.
import copy
solo_cuando = copy.deepcopy(DECIDIDO)
d = solo_cuando[0]["description"]
solo_cuando[0]["description"] = d.replace(
  "Any id used in a relationship must carry a label or a body when another id in the same "
  "diagram does; a bare id alongside a labelled one is rejected.\n\n", "")
report("C. B sin la invariante de la asimetria", solo_cuando)

sin_ejemplo = copy.deepcopy(DECIDIDO)
sin_ejemplo[0]["inputSchema"]["properties"]["view_id"]["description"] = \
  "Short human-readable name, shown to the user above the diagram. Reusing a name replaces that view."
report("D. B con la clausula del view_id sin ejemplo", sin_ejemplo)

todo = copy.deepcopy(DECIDIDO)
todo[0]["description"] = todo[0]["description"].replace(
  "coexist. The flipchart dies",
  "coexist. Style (style, classDef, linkStyle) is discarded. Mermaid support is close to "
  "complete but not identical to mermaid-js. Do not use it to illustrate every answer. "
  "The flipchart dies")
report("E. B mas lo que #26 decidio NO contar (estilo, mmdr, cuando no)", todo)

a = n(json.dumps({"tools": ESCUETO})); b = n(json.dumps({"tools": DECIDIDO}))
print(f"\nB - A = +{b-a} tokens sobre la linea base ({a} -> {b})")
print(f"E - B = +{n(json.dumps({'tools': todo}))-b} tokens que se ahorran por no contarlo")
