#!/bin/bash
# Lanzador de juguete: anota lo que ve del blob y responde al handshake MCP.
LOG="${FAT_LOG:-/tmp/fat-plugin.log}"
mkdir -p "$(dirname "$LOG")" 2>/dev/null
BLOB="$(dirname "$0")/bin/blob"
{
  echo "--- $(date +%s) launcher start"
  echo "root=$CLAUDE_PLUGIN_ROOT"
  echo "self=$0"
  echo "blob_exists=$([ -f "$BLOB" ] && echo yes || echo no)"
  echo "blob_size=$(stat -f %z "$BLOB" 2>/dev/null || echo NA)"
  echo "blob_mode=$(stat -f %Sp "$BLOB" 2>/dev/null || echo NA)"
  echo "blob_exec=$([ -x "$BLOB" ] && echo yes || echo no)"
  echo "blob_xattr=$(xattr "$BLOB" 2>/dev/null | tr '\n' ',')"
} >> "$LOG"
# Servidor de aviso minimo: responde initialize y tools/list, nada mas.
while IFS= read -r line; do
  case "$line" in
    *'"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":0,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"fat","version":"0.0.1"}}}'
      ;;
    *'"tools/list"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"fat_status","description":"Dice si el blob llego","inputSchema":{"type":"object","properties":{}}}]}}'
      ;;
  esac
done
