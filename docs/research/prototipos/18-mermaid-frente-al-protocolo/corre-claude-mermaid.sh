#!/usr/bin/env bash
# Instala y corre claude-mermaid 1.6.5 — el competidor del issue #15 —
# midiendo su peaje fijo y su comportamiento observable.
#
# ABRE PESTANAS DE NAVEGADOR (eso es parte de lo que se mide) y deja un
# servidor HTTP en localhost:3737 hasta que mates el proceso.
# XDG_CONFIG_HOME se apunta aqui para no ensuciar ~/.config del usuario.
set -euo pipefail
cd "$(dirname "$0")"
export XDG_CONFIG_HOME="$PWD/xdg"

[ -d node_modules ] || npm install claude-mermaid@1.6.5 --silent
echo "node_modules: $(du -sh node_modules | cut -f1)"

echo
echo "== tools/list, tal cual lo ve el agente =="
node probe.mjs > tools-raw.jsonl
python3 -c "
import json
for l in open('tools-raw.jsonl'):
    m=json.loads(l)
    if m.get('id')==2: json.dump(m['result']['tools'], open('claude-mermaid-tools.json','w'), indent=1)
print('capturado en claude-mermaid-tools.json')"
python3 -c "import tiktoken" 2>/dev/null && ./peaje.py || echo "(pip install tiktoken para el peaje)"

echo
echo "== render y live reload, con dos vistas con nombre =="
python3 -c "
import json
json.dump([
 {'name':'actual','args':{'diagram':open('fixtures/vista-actual.mmd').read(),'preview_id':'actual','format':'svg'}},
 {'name':'propuesto','args':{'diagram':open('fixtures/vista-propuesto.mmd').read(),'preview_id':'propuesto','format':'svg'}},
 {'name':'typo (la mentira plausible, con Mermaid.js)','args':{'diagram':open('fixtures/typo.mmd').read(),'preview_id':'typo','format':'png'}},
], open('casos.json','w'))"
node drive.mjs casos.json

echo
echo "== la galeria, que es lo contrario de efimero =="
echo "   ficheros que sobreviven en \$XDG_CONFIG_HOME/claude-mermaid/live/:"
find xdg/claude-mermaid/live -type f | sed 's/^/     /'
echo "   closeLiveServer() esta exportada. Quien la llama:"
grep -rn "closeLiveServer" node_modules/claude-mermaid/build/*.js | sed 's/^/     /'
