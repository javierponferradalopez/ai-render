#!/bin/bash
# Empaqueta la caja: los cuatro ficheros del ADR-0013 y nada más, en un zip de
# Info-ZIP, y escribe en stdout la ruta del zip.
#
#   empaqueta.sh <version-o-tag> <binario> <destino>
#
# Info-ZIP (`zip`) es parte del contrato, no comodidad: es el empaquetador que
# escribe `version made by == 3` con los modos Unix intactos, y de ahí sale el
# `100755` con el que el binario llega a la máquina del usuario —el host lee los
# atributos externos y hace `chmod(mode & 0o777)` cuando hay bit de ejecución—.
# Nunca desde el Finder, que mete `__MACOSX/` y `.DS_Store`.
set -euo pipefail

RAIZ=$(cd "$(dirname "$0")/.." && pwd)
readonly RAIZ
readonly MANIFIESTO=publicacion/caja/.claude-plugin/plugin.json

# shellcheck source=json.sh
. "$RAIZ/publicacion/json.sh"

muere() {
  printf 'empaqueta: %s\n' "$1" >&2
  exit 1
}

la_version_del_cargo_toml() {
  local linea
  linea=$(sed -n '/^\[package\]/,/^\[/p' "$RAIZ/Cargo.toml" | grep -m1 '^version') \
    || muere 'Cargo.toml no declara version en su [package]'
  linea=${linea#*\"}
  printf '%s' "${linea%%\"*}"
}

[ $# -eq 3 ] || muere 'uso: empaqueta.sh <version-o-tag> <binario> <destino>'
readonly VERSION=${1#v}
readonly BINARIO=$2
readonly DESTINO=$3

[ -f "$BINARIO" ] || muere "no hay binario en $BINARIO"

# La versión que manda es la del `plugin.json` de dentro del zip —la UI de
# `/plugin` hace `manifest.version ?? "unknown"`—, y la del tag es la que va a
# ver el catálogo. Que coincidan no se comprueba por pulcritud: publicarlas
# desalineadas le deja al usuario una versión distinta de la que instaló.
declarada=$(campo version "$(grep -m1 '"version"' "$RAIZ/$MANIFIESTO")") \
  || muere "$MANIFIESTO no declara version"
[ "$declarada" = "$VERSION" ] || muere "$MANIFIESTO declara $declarada y el tag dice $VERSION"
del_cargo=$(la_version_del_cargo_toml)
[ "$del_cargo" = "$VERSION" ] || muere "Cargo.toml declara $del_cargo y el tag dice $VERSION"

# Los cuatro ficheros se copian uno a uno y no hay un `cp -R` de un directorio
# entero, que es lo que dejaría entrar un `.DS_Store` del árbol de trabajo o un
# `skills/` que alguien añadiese de paso. La caja del ADR-0013 es cerrada, así que
# no hace falta comprobar que lo es: no hay por dónde meter un quinto fichero.
readonly CAJA="$DESTINO/caja"
rm -rf "$CAJA"
mkdir -p "$CAJA/.claude-plugin"
cp "$RAIZ/publicacion/caja/.claude-plugin/plugin.json" "$CAJA/.claude-plugin/plugin.json"
cp "$RAIZ/publicacion/caja/.mcp.json" "$CAJA/.mcp.json"
cp "$RAIZ/launcher.sh" "$CAJA/launcher.sh"
cp "$BINARIO" "$CAJA/flipchart"
chmod 755 "$CAJA/launcher.sh" "$CAJA/flipchart"
chmod 644 "$CAJA/.claude-plugin/plugin.json" "$CAJA/.mcp.json"

readonly ZIP="$DESTINO/flipchart-$VERSION.zip"
rm -f "$ZIP"
( cd "$CAJA" && zip -q -r -X "$ZIP" . )

# Que el zip lleve atributos Unix no lo promete el esquema del host: lo promete
# el empaquetador, y esto es lo que lo comprueba en vez de confiarlo. Sin ellos
# el binario llegaría sin bit de ejecución, y el `chmod +x` del Lanzador dejaría
# de ser respaldo para ser mecanismo.
mirado=$(unzip -Z "$ZIP")
for ejecutable in flipchart launcher.sh; do
  grep -qE "^-rwxr-xr-x +[0-9.]+ unx .* $ejecutable\$" <<<"$mirado" \
    || muere "$ejecutable no viaja como -rwxr-xr-x de Unix:
$mirado"
done

# El tope del archive es de 256 MiB y **no tiene válvula**: no hay variable de
# entorno que lo amplíe, así que pasarse de aquí no se degrada, deja el plugin
# sin forma de instalarse. El margen es enorme hoy y lo que lo come son las
# dependencias, que es justo lo que nadie mira al añadir una.
readonly TOPE_DEL_ARCHIVE=$((256 * 1024 * 1024))
bytes=$(stat -f %z "$ZIP")
[ "$bytes" -le "$TOPE_DEL_ARCHIVE" ] \
  || muere "el zip son $bytes bytes y el tope del archive son $TOPE_DEL_ARCHIVE, sin válvula"

printf 'empaqueta: %s bytes, tope %s\n' "$bytes" "$TOPE_DEL_ARCHIVE" >&2
printf 'empaqueta: %s\n' "$(cd "$DESTINO" && shasum -a 256 "$(basename "$ZIP")")" >&2
printf '%s\n' "$ZIP"
