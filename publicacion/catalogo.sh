#!/bin/bash
# Genera el marketplace.json desde el tag del release, en stdout. Nunca se
# edita a mano.
#
#   catalogo.sh <tag> <zip> [repo]
#
# Es requisito de corrección, no comodidad: medido, `/plugin update` descarga el
# zip entero *antes* de comparar identidades, así que un arreglo publicado sin
# subir la versión se baja, se tira y no avisa —4,1 s para decir «already at the
# latest version»—. Sacar `version`, `url` y `sha256` del mismo sitio, y del
# mismo zip, es lo que hace imposible olvidar el bump.
set -euo pipefail

RAIZ=$(cd "$(dirname "$0")/.." && pwd)
readonly RAIZ

# shellcheck source=json.sh
. "$RAIZ/publicacion/json.sh"

muere() {
  printf 'catalogo: %s\n' "$1" >&2
  exit 1
}

[ $# -ge 2 ] || muere 'uso: catalogo.sh <tag> <zip> [repo]'
readonly TAG=$1
readonly ZIP=$2
readonly REPO=${3:-${GITHUB_REPOSITORY:-javierponferradalopez/ai-render}}
readonly VERSION=${TAG#v}

[ -f "$ZIP" ] || muere "no hay zip en $ZIP"

# El manifiesto se lee de dentro del zip que se va a publicar, no del árbol de
# trabajo: así el catálogo no puede describir otra cosa que la caja que sirve.
readonly MANIFIESTO=.claude-plugin/plugin.json
manifiesto=$(unzip -p "$ZIP" "$MANIFIESTO") || muere "el zip no lleva $MANIFIESTO"
nombre=$(campo name "$manifiesto") || muere "$MANIFIESTO no declara name"
descripcion=$(campo description "$manifiesto") || muere "$MANIFIESTO no declara description"
declarada=$(campo version "$manifiesto") || muere "$MANIFIESTO no declara version"

# La que manda es la del manifiesto de dentro del zip, y la del tag es la que va
# a leer el usuario. Un catálogo que las mezcle publica una mentira.
[ "$declarada" = "$VERSION" ] \
  || muere "el zip declara $declarada y el tag dice $VERSION"

# El JSON se compone con `printf`, así que un `"` o un `\` en un valor lo
# rompería en silencio. La premisa se comprueba en vez de confiarse.
for valor in "$nombre" "$descripcion"; do
  case $valor in
    *'"'* | *'\'*) muere "un valor del manifiesto lleva comillas o barras: $valor" ;;
  esac
done

# Medido: el `sha256` es opcional en el esquema del host, y una entrada sin él
# se instala igual y sin comprobar nada, sin aviso. Publicarlo vacío desarmaría
# en silencio la única defensa de integridad del vehículo.
digesto=$(shasum -a 256 "$ZIP" | cut -d' ' -f1)
[ ${#digesto} -eq 64 ] || muere "el sha256 del zip no salió: '$digesto'"

# Inmutable a propósito: un digest pinneado apunta a un byte exacto, así que si
# la URL pudiera cambiar de contenido el pin no valdría nada.
readonly URL="https://github.com/$REPO/releases/download/$TAG/$(basename "$ZIP")"

cat <<JSON
{
  "name": "$nombre",
  "description": "$descripcion",
  "owner": { "name": "${REPO%%/*}" },
  "plugins": [
    {
      "name": "$nombre",
      "description": "$descripcion",
      "version": "$VERSION",
      "source": {
        "source": "archive",
        "url": "$URL",
        "sha256": "$digesto"
      }
    }
  ]
}
JSON
