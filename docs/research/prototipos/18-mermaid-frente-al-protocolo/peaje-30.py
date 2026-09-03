#!/usr/bin/env python3
"""Peaje de la descripcion que decide el issue #30, contra la de #26.

Mismo metodo que peaje-26.py: cl100k_base via tiktoken, comparable con
research 04 y los issues #10, #15, #18 y #26.
"""
import json, tiktoken
enc = tiktoken.get_encoding("cl100k_base")
n = lambda s: len(enc.encode(s))

def report(label, tools):
    print(f"{label:<44} {n(json.dumps({'tools': tools})):>4} tokens")

# #30: fuera el liston del cuando y la norma de ofrecer; dentro el view_id con
# ejemplo, la mecanica de reemplazo, la muerte con la sesion y la asimetria.
DECIDIDO_30 = [
  {"name":"flipchart_show",
   "description":(
     "Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\n"
     "Any id used in a relationship must carry a label or a body when another id in the same "
     "diagram does; a bare id alongside a labelled one is rejected.\n\n"
     "Showing an existing view id replaces it and brings it to the front; several named views "
     "coexist. The flipchart dies with the session."),
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":(
         "Short human-readable name, shown to the user above the diagram - e.g. "
         "\"Current dependencies\", not \"v1\". Reusing a name replaces that view.")},
      "diagram":{"type":"string","description":"Mermaid source."}},
      "required":["view_id","diagram"]}},
  {"name":"flipchart_clear",
   "description":"Remove one view from the flipchart, or all of them. Does not close the window.",
   "inputSchema":{"type":"object","properties":{
      "view_id":{"type":"string","description":"View to remove. Omit to clear the whole flipchart."}},
      "required":[]}},
]

SIN_ASIMETRIA = json.loads(json.dumps(DECIDIDO_30))
SIN_ASIMETRIA[0]["description"] = (
     "Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\n"
     "Showing an existing view id replaces it and brings it to the front; several named views "
     "coexist. The flipchart dies with the session.")

report("Lo decidido en #30", DECIDIDO_30)
report("  ...sin la clausula de la asimetria", SIN_ASIMETRIA)
