#!/bin/bash
# Lanzador-sonda. No lanza nada: anota lo que ve del binario y responde el handshake.
set -u
BIN="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "$0")" && pwd)}/bin/flipchart"
OUT="${PROBE_OUT:-/tmp/flipchart-probe.json}"
mkdir -p "$(dirname "$OUT")" 2>/dev/null

mode=$(stat -f "%Sp %p" "$BIN" 2>/dev/null || echo "ABSENT")
size=$(stat -f "%z" "$BIN" 2>/dev/null || echo 0)
xattrs=$(xattr "$BIN" 2>/dev/null | tr '\n' ',' )
quar=$(xattr -p com.apple.quarantine "$BIN" 2>/dev/null || echo "NONE")
if [ -x "$BIN" ]; then execbit=yes; else execbit=no; fi

# El test de Gatekeeper: correrlo de verdad, sin chmod previo.
raw_out=$("$BIN" 2>&1); raw_rc=$?
# Y despues con el chmod +x que el Lanzador hace por diseno (#23).
chmod +x "$BIN" 2>/dev/null
post_out=$("$BIN" 2>&1); post_rc=$?
post_mode=$(stat -f "%Sp %p" "$BIN" 2>/dev/null || echo ABSENT)

cat > "$OUT" <<EOJ
{
  "plugin_root": "${CLAUDE_PLUGIN_ROOT:-unset}",
  "plugin_data": "${CLAUDE_PLUGIN_DATA:-unset}",
  "bin": "$BIN",
  "size": $size,
  "mode_before_chmod": "$mode",
  "exec_bit_before_chmod": "$execbit",
  "xattrs": "$xattrs",
  "quarantine": "$quar",
  "run_before_chmod_rc": $raw_rc,
  "run_before_chmod_out": "$(printf '%s' "$raw_out" | tr -d '"' | tr '\n' ' ')",
  "mode_after_chmod": "$post_mode",
  "run_after_chmod_rc": $post_rc,
  "run_after_chmod_out": "$(printf '%s' "$post_out" | tr -d '"' | tr '\n' ' ')"
}
EOJ

# Servidor de aviso: JSON-RPC minimo por stdio, una herramienta sin argumentos.
while IFS= read -r line; do
  case "$line" in
    *'"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"flipchart-probe","version":"1.0.0"}}}' ;;
    *'"tools/list"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"probe_status","description":"Lo que la sonda vio del binario","inputSchema":{"type":"object","properties":{}}}]}}' ;;
    *'"notifications/initialized"'*) : ;;
  esac
done
