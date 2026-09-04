#!/usr/bin/env bash
# ¿Dibuja el agente sin que nadie se lo pida, con la línea del §8.2 pegada y el
# plugin de verdad instalado?
#
# Es el escenario `refactor` del prototipo 22 —el caso protagonista: el usuario
# cuenta el movimiento que se plantea y pide entender las dependencias, y **no
# pide dibujo**— corrido esta vez contra el plugin del release, no contra el
# servidor falso. Con `--concede`, además, la llamada llega al Servidor MCP y la
# ventana aparece; sin él, el host la deniega y lo que queda medido es la
# conducta.
#
# Uso: corre-con-la-linea.sh <caja-del-plugin> <sujeto> <salida> [--concede]
#      corre-con-la-linea.sh <caja> <sujeto> <salida> --control-positivo
#
# <sujeto> es una copia desechable de un repo con capas de verdad, sin AGENTS.md
# ni CLAUDE.md propios: el único canal de enseñanza que se le deja es la línea,
# que se copia de `condiciones/CLAUDE.md.con-la-linea`. Nunca un repo de trabajo:
# el turno 2 del guion es un «Sí, adelante» y el agente lo lee como «hazlo».
set -uo pipefail
CAJA="${1:?falta la caja del plugin}"
SUJETO="${2:?falta el sujeto}"
SALIDA="${3:?falta el directorio de salida}"
MODO="${4:-}"

AQUI="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$SALIDA"
cp "$AQUI/condiciones/CLAUDE.md.con-la-linea" "$SUJETO/CLAUDE.md"

# El guion. El control positivo mide la tubería, no la conducta: pide el dibujo
# a la cara, así que un turno basta y sale con el modelo barato.
GUION=(
  'Estoy pensando en mover a store.lua la logica de marks.lua que recoloca un comentario cuando el fichero ha cambiado debajo, para que todo lo que sabe de lineas viva en un solo sitio. Antes de tocar una linea quiero entender bien que depende de que entre los cuatro modulos de lua/pickypen. Echale un ojo y cuentame.'
  'Si, adelante.'
  'Vale. Y como quedaria despues del movimiento?'
)
MODELO=()
if [ "$MODO" = --control-positivo ]; then
  GUION=('Usa la herramienta flipchart para ensenarme en la pizarra como se relacionan los cuatro modulos de lua/pickypen.')
  MODELO=(--model claude-haiku-4-5-20251001)
fi

# `--allowedTools` no concede herramientas MCP en modo `-p`, ni lo hace un
# `permissions.allow` en los settings; el hook sí. Ver
# `condiciones/concede-la-pizarra.sh`.
AJUSTES=()
if [ "$MODO" = --concede ] || [ "$MODO" = --control-positivo ]; then
  cat >"$SALIDA/permisos.json" <<EOF
{
  "permissions": { "deny": ["Edit", "Write", "NotebookEdit"] },
  "hooks": {
    "PreToolUse": [
      { "matcher": "mcp__plugin_flipchart_flipchart__.*",
        "hooks": [ { "type": "command", "command": "$AQUI/condiciones/concede-la-pizarra.sh" } ] }
    ]
  }
}
EOF
  AJUSTES=(--settings "$SALIDA/permisos.json")
fi

SID=""
n=0
for texto in "${GUION[@]}"; do
  n=$((n + 1))
  echo "=== turno $n" >&2
  # Sólo lectura, y además prohibido escribir explícitamente. Ni `acceptEdits`
  # ni `bypassPermissions`: los dos auto-aprueban las ediciones al margen de
  # esta lista, y el turno 2 del guion es exactamente el que las dispara.
  args=(-p "$texto" --output-format stream-json --verbose
    --plugin-dir "$CAJA"
    --disallowedTools Edit Write NotebookEdit Task
    --permission-mode default
    "${MODELO[@]+"${MODELO[@]}"}" "${AJUSTES[@]+"${AJUSTES[@]}"}")
  [ -n "$SID" ] && args+=(--resume "$SID")
  (cd "$SUJETO" && claude "${args[@]}" </dev/null) >"$SALIDA/turno-$n.jsonl" 2>"$SALIDA/turno-$n.err"
  SID=$(python3 "$AQUI/destila.py" --sesion "$SALIDA/turno-$n.jsonl")
  echo "  session=$SID" >&2
done

python3 "$AQUI/destila.py" "$SALIDA"
