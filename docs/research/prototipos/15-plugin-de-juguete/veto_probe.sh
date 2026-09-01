#!/bin/sh
# Un paso de la prueba del veto: corre una sesion (plugin instalado) y dice si el
# proceso del servidor se lanzo, mas la entrada del cache.
LAB="$1"; TAG="$2"
export TOY_LOG="$LAB/logs/veto-$TAG.log"; : > "$TOY_LOG"
cd "$LAB/work" || exit 1
python3 "$LAB/hold.py" 12 "$LAB/logs/veto-$TAG" claude -p --input-format stream-json \
  --output-format stream-json --verbose --model haiku \
  --debug-file "$LAB/logs/veto-$TAG-debug.txt" > /dev/null
printf "%-12s lanzado=%s  " "$TAG" "$([ -s "$TOY_LOG" ] && echo SI || echo NO)"
python3 -c "
import json
d=json.load(open('/Users/ponfe/.claude/mcp-needs-auth-cache.json'))
e=d.get('plugin:toy:toy')
print('cache=', json.dumps(e))"
