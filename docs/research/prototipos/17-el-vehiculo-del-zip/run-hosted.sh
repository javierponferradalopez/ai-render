#!/bin/bash
# La mitad del experimento que SI necesita alojar de verdad, y que quedo sin
# ejecutar: `source: "archive"` exige https:// y prohibe loopback, link-local y
# hosts de metadatos de nube, revalidado en cada uno de los hasta 5 saltos, asi
# que no hay atajo local como el `file://` de research 10.
#
# Requiere un repo publico desechable y permiso para publicarlo. Uso:
#
#   ./run-hosted.sh javierponferradalopez/flipchart-archive-probe
#
# Deja el repo vivo a proposito: borrarlo pide el scope delete_repo, que el
# token de `gh` no trae (gh auth refresh -h github.com -s delete_repo).
set -uo pipefail
cd "$(dirname "$0")"
REPO="${1:?uso: run-hosted.sh <owner>/<repo>}"
OUT="${2:-$PWD/out}"
CFG="$OUT/cfg-hosted"
export CLAUDE_CONFIG_DIR="$CFG"
TAG=v1.0.0

[ -d "$OUT/zips" ] || { echo "falta $OUT/zips: corre ./build.sh primero"; exit 1; }

echo "############ 0. publicar el material"
gh repo create "$REPO" --public \
  --description "TEMPORAL / DESECHABLE - banco de pruebas del vehiculo archive (ai-render research 11)" || true
W="$OUT/repo"; rm -rf "$W"; git clone "https://github.com/$REPO.git" "$W" 2>/dev/null || \
  { mkdir -p "$W" && git -C "$W" init -q && git -C "$W" remote add origin "git@github.com:$REPO.git"; }

gh release create "$TAG" --repo "$REPO" --title "$TAG" --notes "sonda" \
  "$OUT/zips/flipchart-1.0.0.zip" "$OUT/zips/t-wrapped.zip" "$OUT/zips/t-finder.zip" \
  "$OUT/zips/t-empty.zip" "$OUT/zips/t-noplugin.zip" || true

base="https://github.com/$REPO/releases/download/$TAG"
sha() { shasum -a 256 "$OUT/zips/$1" | cut -d' ' -f1; }
S_FLAT=$(sha flipchart-1.0.0.zip)

# El digest equivocado, para el apartado 3: mismo zip, sha256 de otra cosa.
S_MALO=$(printf 'b%.0s' {1..64})

mkdir -p "$W"
cat > "$W/marketplace.json" <<EOF
{
  "name": "flipchart-probe",
  "owner": { "name": "ai-render research" },
  "plugins": [
    { "name": "flipchart", "description": "el zip plano, el caso real", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/flipchart-1.0.0.zip", "sha256": "$S_FLAT" } },
    { "name": "digest-malo", "description": "mismo zip, digest que no casa", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/flipchart-1.0.0.zip", "sha256": "$S_MALO" } },
    { "name": "sin-declarar", "description": "el zip plano sin sha256 declarado", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/flipchart-1.0.0.zip" } },
    { "name": "envuelto", "description": "dentro de un unico directorio envoltorio", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/t-wrapped.zip", "sha256": "$(sha t-wrapped.zip)" } },
    { "name": "con-finder", "description": "con __MACOSX/ y .DS_Store", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/t-finder.zip", "sha256": "$(sha t-finder.zip)" } },
    { "name": "solo-basura", "description": "solo __MACOSX/ y .DS_Store", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/t-empty.zip", "sha256": "$(sha t-empty.zip)" } },
    { "name": "sin-forma", "description": "contenido sin forma de plugin", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/t-noplugin.zip", "sha256": "$(sha t-noplugin.zip)" } }
  ]
}
EOF
git -C "$W" add marketplace.json
git -C "$W" -c user.email=probe@local -c user.name=probe commit -qm "catalogo de la sonda"
git -C "$W" push -q origin HEAD:main --force

CAT="https://raw.githubusercontent.com/$REPO/main/marketplace.json"
echo "catalogo en $CAT"; sleep 5   # raw tiene cache corta

rm -rf "$CFG"; mkdir -p "$CFG"
claude plugin marketplace add "$CAT"

echo
echo "############ 1. LA CUARENTENA - la condicion de viabilidad"
time claude plugin install flipchart@flipchart-probe
BIN=$(find "$CFG/plugins" -name flipchart -type f -path '*/bin/*' | head -1)
echo "binario extraido: ${BIN:-NO ENCONTRADO}"
if [ -n "${BIN:-}" ]; then
  echo "  modo:        $(stat -f '%Sp %p' "$BIN")"
  echo "  bytes:       $(stat -f '%z' "$BIN")"
  echo "  sha256:      $(shasum -a 256 "$BIN" | cut -d' ' -f1)  (origen: $(shasum -a 256 "$OUT/flipchart" | cut -d' ' -f1))"
  echo "  xattrs:      $(xattr "$BIN" | tr '\n' ' ')  <- vacio = sin cuarentena"
  echo "  cuarentena:  $(xattr -p com.apple.quarantine "$BIN" 2>/dev/null || echo NINGUNA)"
  echo "  ejecuta?     "; "$BIN"; echo "  rc=$?"
  echo "  gatekeeper:  "; spctl -a -vv "$BIN" 2>&1 | head -3
fi
echo "y en la copia versionada de la cache, no solo en la extraccion:"
find "$CFG/plugins" -name flipchart -type f | while read -r f; do
  echo "  $(stat -f '%Sp' "$f")  xattr=[$(xattr "$f" | tr '\n' ' ')]  ${f#$CFG}"
done
echo "lo que vio el lanzador:"
find "$CFG" -name probe.json -exec cat {} \;

echo
echo "############ 3. el digest que no casa"
claude plugin install digest-malo@flipchart-probe 2>&1 | head -6
echo "y sin sha256 declarado:"
claude plugin install sin-declarar@flipchart-probe 2>&1 | head -4

echo
echo "############ 6. las formas del zip"
for p in envuelto con-finder solo-basura sin-forma; do
  echo "---- $p"; claude plugin install "$p@flipchart-probe" 2>&1 | head -5
done

echo
echo "############ 5. /plugin update con la disciplina de #23"
echo "-- a) mismo version, mismo digest: no deberia haber nada que hacer"
claude plugin update flipchart 2>&1 | head -4
echo "-- b) SOLO el digest cambia (misma version declarada): dispara?"
#   Se sube otro zip y se cambia url+sha256 sin tocar version ni plugin.json.
echo "   (sube un segundo asset y reescribe la entrada; ver README)"
echo "-- c) version + url + sha256 a la vez: debe disparar"
echo "   (bump de plugin.json y de la entrada, segundo release; ver README)"

echo
echo "############ 7. los numeros"
echo "  zip: $(stat -f '%z' "$OUT/zips/flipchart-1.0.0.zip") bytes"
echo "  tope del host: 268435456 bytes (256 MiB), plazo 120000 ms sin valvula"
