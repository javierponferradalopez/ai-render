#!/bin/bash
# Extrae la fuente JS minificada de la cola del ejecutable de Claude Code a
# js.txt, que es sobre lo que trabaja look.sh.
#
# NO uses `strings` aqui, aunque la receta vieja del README lo hiciera: el
# `strings` de macOS parsea Mach-O y sobre este ejecutable devuelve un js.txt
# incompleto -- 11 MB de los 43 que hay, y sin las cadenas que se buscan, asi
# que `look.sh` no encuentra nada y parece que el codigo ha cambiado. `tr` no
# interpreta el formato y saca todo lo imprimible.
#
#   ./js.sh [MB_desde_los_que_cortar]     # por defecto 150
set -euo pipefail
cd "$(dirname "$0")"
B="${CLAUDE_BIN:-$(readlink -f "$(command -v claude)")}"
SKIP="${1:-150}"
mkdir -p out
dd if="$B" bs=1M skip="$SKIP" 2>/dev/null | LC_ALL=C tr -c '\11\12\40-\176' '\n' > out/js.txt
echo "$B ($(stat -f '%z' "$B") bytes) -> out/js.txt ($(stat -f '%z' out/js.txt) bytes)"

# Si el corte cae por debajo de donde empieza el JS, js.txt sale vacio de lo que
# importa y todo lo demas parece haber desaparecido del bundle. Se avisa aqui.
if ! grep -q 'must not point at a loopback' out/js.txt; then
  echo "AVISO: no aparece la politica de URL del archive. O el corte de ${SKIP} MB" >&2
  echo "       se ha comido el JS, o esta version ya no la trae. Prueba con menos MB." >&2
  rm -f out/js.txt   # un js.txt a medias es peor que ninguno: look.sh lo leeria
  exit 1
fi
echo "ok: la fuente JS esta ahi (la politica de URL del archive aparece)"
