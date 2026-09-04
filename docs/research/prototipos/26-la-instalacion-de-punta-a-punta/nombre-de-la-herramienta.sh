#!/usr/bin/env bash
# Con qué nombre presenta el host la herramienta del plugin.
#
# La respuesta la da el evento `init` del stream: `mcp_servers` trae el nombre
# interno del servidor y `tools` los nombres tal como el modelo los ve. No hay
# que llamar a nada ni abrir ninguna ventana: basta un turno que responda "ok".
#
# Uso: nombre-de-la-herramienta.sh <caja-del-plugin>
#
# <caja-del-plugin> es un directorio con el `.claude-plugin/plugin.json`, el
# `.mcp.json`, el lanzador y el binario. El del release de verdad se lo deja
# extraído el host en:
#   $CLAUDE_CONFIG_DIR/plugins/cache/<marketplace>/<plugin>/<version>/
set -uo pipefail
CAJA="${1:?falta la caja del plugin}"
SALIDA="${2:-/dev/stdout}"

claude -p 'responde solo: ok' \
  --output-format stream-json --verbose \
  --model claude-haiku-4-5-20251001 \
  --plugin-dir "$CAJA" </dev/null |
  python3 -c '
import json, sys
for line in sys.stdin:
    try:
        e = json.loads(line)
    except Exception:
        continue
    if e.get("subtype") == "init":
        print("servidores:", [m for m in e.get("mcp_servers", []) if "flip" in m["name"]])
        print("herramientas:", [t for t in e.get("tools", []) if "flip" in t])
' >"$SALIDA"
