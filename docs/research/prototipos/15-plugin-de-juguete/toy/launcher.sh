#!/bin/sh
# Lanzador de juguete: registra lo que ve, espera TOY_DELAY segundos y arranca el
# servidor MCP minimo. Todo lo interesante queda en TOY_LOG.
LOG="${TOY_LOG:-/tmp/toy-plugin.log}"
DELAY="${TOY_DELAY:-0}"
STAMP() { python3 -c 'import time;print("%.3f"%time.time())'; }
echo "$(STAMP) launcher start pid=$$ argv=[$*]" >> "$LOG"
echo "$(STAMP) launcher env CLAUDE_PLUGIN_ROOT=[${CLAUDE_PLUGIN_ROOT}] CLAUDE_PLUGIN_DATA=[${CLAUDE_PLUGIN_DATA}]" >> "$LOG"
echo "$(STAMP) launcher cwd=[$(pwd)] delay=${DELAY}" >> "$LOG"
trap 'echo "$(STAMP) launcher SIGTERM" >> "$LOG"; exit 143' TERM
trap 'echo "$(STAMP) launcher SIGINT" >> "$LOG"; exit 130' INT
if [ -n "$TOY_FAIL" ]; then
  echo "flipchart: no se pudo descargar el binario: curl: (6) Could not resolve host" >&2
  echo "$(STAMP) launcher fail-exit" >> "$LOG"
  exit 1
fi
if [ "$DELAY" != "0" ]; then
  sleep "$DELAY" &
  wait $!
  echo "$(STAMP) launcher slept ${DELAY}s" >> "$LOG"
fi
exec python3 "$(dirname "$0")/mcp_server.py"
