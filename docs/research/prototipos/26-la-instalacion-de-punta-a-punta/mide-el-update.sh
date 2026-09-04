#!/usr/bin/env bash
# ¿Trae `/plugin update` la versión nueva, o dice `already at the latest version`?
#
# Con el plugin de verdad, y por eso el guion tiene una precondición que no se
# puede saltar: **el config del banco tiene que traer instalada la versión
# vieja** desde antes de que se publique la nueva. Si se instala después, se
# instala ya la nueva y no hay update que medir.
#
# Uso: mide-el-update.sh <config-del-banco> <version-vieja> <version-nueva>
#
# No hace falta sesión: los comandos de plugin funcionan en un
# CLAUDE_CONFIG_DIR sin login.
set -uo pipefail
CONFIG="${1:?falta el CLAUDE_CONFIG_DIR del banco}"
VIEJA="${2:?falta la versión vieja}"
NUEVA="${3:?falta la versión nueva}"
export CLAUDE_CONFIG_DIR="$CONFIG"

CACHE="$CONFIG/plugins/cache/flipchart/flipchart"

echo "############ 0. de qué versión se parte"
claude plugin list 2>&1 | grep -E 'flipchart|Version'
[ -d "$CACHE/$VIEJA" ] || { echo "el banco no tiene la $VIEJA instalada: nada que medir"; exit 1; }
du -sh "$CONFIG" | cut -f1

echo
echo "############ 1. el catálogo, refrescado"
claude plugin marketplace update flipchart 2>&1 | tail -2
grep -o '"version": "[^"]*"' "$CONFIG/plugins/marketplaces/flipchart" | head -1

echo
echo "############ 2. el update"
# Con el nombre corto contesta `Plugin "flipchart" not found` aunque esté
# instalado: `update` no lo resuelve, y `uninstall` sí. Medido.
time claude plugin update flipchart@flipchart 2>&1 | tail -3

echo
echo "############ 3. lo que queda en la caché"
ls -1 "$CACHE" | sed 's/^/  /'
du -sh "$CONFIG" | cut -f1

echo
echo "############ 4. el binario nuevo, y si habla"
BIN="$CACHE/$NUEVA/flipchart"
if [ ! -x "$BIN" ]; then
  echo "  no hay binario de la $NUEVA en la caché"
  exit 1
fi
stat -f '  modo=%Sp bytes=%z' "$BIN"
xattr -l "$BIN" | sed 's/^/  xattr: /'
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"banco","version":"0"}}}' |
  "$CACHE/$NUEVA/launcher.sh" 2>/dev/null | head -1 | sed 's/^/  handshake: /'
