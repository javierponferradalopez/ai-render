#!/bin/sh
# Sesion viva HOLD segundos en modo stream-json (sin llamada al API), con el
# lanzador durmiendo DELAY segundos. Mide donde corta el arranque del MCP.
LAB="$1"; DELAY="$2"; HOLD="$3"
export TOY_LOG="$LAB/logs/hold-$DELAY.log"; : > "$TOY_LOG"
export TOY_DELAY="$DELAY"
cd "$LAB/work" || exit 1
python3 -c 'import time;print("%.3f harness launch"%time.time())' >> "$TOY_LOG"
python3 "$LAB/hold.py" "$HOLD" "$LAB/logs/hold-$DELAY" claude -p --input-format stream-json \
  --output-format stream-json --verbose --model haiku --plugin-dir "$LAB/toy" \
  --debug-file "$LAB/logs/hold-$DELAY-debug.txt"
python3 -c 'import time;print("%.3f harness done"%time.time())' >> "$TOY_LOG"
