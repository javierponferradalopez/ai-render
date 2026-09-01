#!/bin/sh
# Un punto del barrido: arranca una sesion headless con el lanzador durmiendo
# DELAY segundos y registra tiempos, salida y debug.
LAB="$1"; DELAY="$2"
export TOY_LOG="$LAB/logs/sweep-$DELAY.log"; : > "$TOY_LOG"
export TOY_DELAY="$DELAY"
cd "$LAB/work" || exit 1
python3 -c 'import time;print("%.3f harness launch"%time.time())' >> "$TOY_LOG"
python3 "$LAB/run.py" 180 "$LAB/logs/sweep-$DELAY" claude -p "di ok" --model haiku \
  --plugin-dir "$LAB/toy" --debug-file "$LAB/logs/sweep-$DELAY-debug.txt" --output-format json
python3 -c 'import time;print("%.3f harness done"%time.time())' >> "$TOY_LOG"
