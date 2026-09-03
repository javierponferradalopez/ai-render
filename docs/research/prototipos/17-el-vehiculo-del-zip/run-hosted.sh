#!/bin/bash
# La mitad del experimento que SI necesita alojar de verdad: `source: "archive"`
# exige https:// y prohibe loopback, link-local y hosts de metadatos de nube,
# revalidado en cada uno de los hasta 5 saltos, asi que no hay atajo local como
# el `file://` de research 10.
#
# Requiere un repo publico desechable y permiso para publicarlo. Uso:
#
#   ./build.sh && ./run-hosted.sh javierponferradalopez/flipchart-archive-probe
#
# Deja el repo vivo a proposito: borrarlo pide el scope delete_repo, que el
# token de `gh` no trae (gh auth refresh -h github.com -s delete_repo).
#
# Todo ocurre bajo un CLAUDE_CONFIG_DIR propio: la instalacion real no se toca.
set -uo pipefail
cd "$(dirname "$0")"
REPO="${1:?uso: run-hosted.sh <owner>/<repo>}"
OUT="${2:-$PWD/out}"
CFG="$OUT/cfg-hosted"
export CLAUDE_CONFIG_DIR="$CFG"
TAG=v1.0.0
TAG2=v2.0.0

[ -d "$OUT/zips" ] || { echo "falta $OUT/zips: corre ./build.sh primero"; exit 1; }

sha() { shasum -a 256 "$OUT/zips/$1" | cut -d' ' -f1; }
base="https://github.com/$REPO/releases/download"
CAT="https://raw.githubusercontent.com/$REPO/main/marketplace.json"
W="$OUT/repo"

# El catalogo entero. La entrada `flipchart` es la unica que se mueve durante la
# corrida: las demas son las formas de zip y los dos casos del digest.
#   catalogo <version> <zip> <base-url> <sha256>
catalogo() {
  cat > "$W/marketplace.json" <<EOF
{
  "name": "flipchart-probe",
  "owner": { "name": "ai-render research" },
  "plugins": [
    { "name": "flipchart", "description": "el zip plano, el caso real", "version": "$1",
      "source": { "source": "archive", "url": "$3/$2", "sha256": "$4" } },
    { "name": "digest-malo", "description": "mismo zip, digest que no casa", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/flipchart-1.0.0.zip", "sha256": "$S_MALO" } },
    { "name": "sin-declarar", "description": "el zip plano sin sha256 declarado", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/flipchart-1.0.0.zip" } },
    { "name": "envuelto", "description": "dentro de un unico directorio envoltorio", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/t-wrapped.zip", "sha256": "$(sha t-wrapped.zip)" } },
    { "name": "con-finder", "description": "con __MACOSX/ y .DS_Store", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/t-finder.zip", "sha256": "$(sha t-finder.zip)" } },
    { "name": "solo-basura", "description": "solo __MACOSX/ y .DS_Store", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/t-empty.zip", "sha256": "$(sha t-empty.zip)" } },
    { "name": "sin-forma", "description": "contenido sin forma de plugin", "version": "1.0.0",
      "source": { "source": "archive", "url": "$base/$TAG/t-noplugin.zip", "sha256": "$(sha t-noplugin.zip)" } }
  ]
}
EOF
}

# Publica el catalogo y no vuelve hasta que raw sirve ESE, no el anterior: su
# CDN cachea, y refrescar contra el viejo mediria la corrida de antes.
#   publicar <marca-que-tiene-que-aparecer> <mensaje>
publicar() {
  local marca="$1" msg="$2" i=0
  git -C "$W" add marketplace.json
  git -C "$W" -c user.email=probe@local -c user.name=probe commit -qm "$msg"
  git -C "$W" push -q origin HEAD:main --force
  while [ $i -lt 360 ]; do
    curl -fsS "$CAT" 2>/dev/null | grep -q "$marca" && { echo "  raw sirve '$msg' tras ${i}s"; return 0; }
    sleep 5; i=$((i+5))
  done
  echo "  AVISO: raw sigue sin servir '$msg' tras ${i}s"
}

# El asset recien subido tarda un instante en ser servible. Sin esta espera un
# 404 de carrera se lee como si la entrada del catalogo fuera mala.
esperar_asset() {
  local url="$1" i=0
  while [ $i -lt 60 ]; do
    curl -fsIL -o /dev/null "$url" && { echo "  asset servible tras ${i}s: ${url##*/}"; return 0; }
    sleep 5; i=$((i+5))
  done
  echo "  AVISO: $url sigue sin responder tras ${i}s"
}

# Lo que quedo en disco de la instalacion, que es de lo que va el ticket.
mirar_binario() {
  local BIN
  BIN=$(find "$CFG/plugins" -name flipchart -type f -path '*/bin/*' | head -1)
  echo "binario extraido: ${BIN:-NO ENCONTRADO}"
  [ -n "${BIN:-}" ] || return 0
  echo "  modo:        $(stat -f '%Sp %p' "$BIN")"
  echo "  bytes:       $(stat -f '%z' "$BIN")"
  echo "  sha256:      $(shasum -a 256 "$BIN" | cut -d' ' -f1)"
  echo "  xattr -l:    [$(xattr -l "$BIN" | tr '\n' ' ')]  <- vacio = sin cuarentena"
  echo "  cuarentena:  $(xattr -p com.apple.quarantine "$BIN" 2>/dev/null || echo NINGUNA)"
  echo -n "  ejecuta?     "; "$BIN"; echo "  rc=$?"
  echo "  gatekeeper:  "; spctl -a -vv "$BIN" 2>&1 | head -3
  echo "  codesign:    "; codesign -dv "$BIN" 2>&1 | grep -E "Signature|Format"
  echo "y sobre todo lo que la cache versionada guarda, no solo la extraccion:"
  find "$CFG/plugins" -name 'flipchart*' | while read -r f; do
    echo "  $(stat -f '%Sp' "$f")  xattr=[$(xattr "$f" | tr '\n' ' ')]  ${f#$CFG}"
  done
}

echo "############ 0. publicar el material"
gh repo create "$REPO" --public \
  --description "TEMPORAL / DESECHABLE - banco de pruebas del vehiculo archive (ai-render research 11)" || true
rm -rf "$W"; git clone "https://github.com/$REPO.git" "$W" 2>/dev/null || \
  { mkdir -p "$W" && git -C "$W" init -q && git -C "$W" remote add origin "https://github.com/$REPO.git"; }

# El digest equivocado, para el apartado 3: mismo zip, sha256 de otra cosa.
S_MALO=$(printf 'b%.0s' {1..64})
S_FLAT=$(sha flipchart-1.0.0.zip)

# El catalogo va ANTES que el release: `gh release create` sobre un repo sin un
# solo commit falla con 422 "Repository is empty", y entonces cada entrada del
# catalogo da 404 y no se mide nada. El push del catalogo es lo que crea `main`.
catalogo 1.0.0 flipchart-1.0.0.zip "$base/$TAG" "$S_FLAT"
publicar "$S_FLAT" "catalogo de la sonda"
echo "catalogo en $CAT"

ZIPS=("$OUT/zips/flipchart-1.0.0.zip" "$OUT/zips/t-wrapped.zip" "$OUT/zips/t-finder.zip"
      "$OUT/zips/t-empty.zip" "$OUT/zips/t-noplugin.zip")
gh release create "$TAG" --repo "$REPO" --title "$TAG" --notes "sonda" "${ZIPS[@]}" || \
  gh release upload "$TAG" --repo "$REPO" --clobber "${ZIPS[@]}"
esperar_asset "$base/$TAG/flipchart-1.0.0.zip"

rm -rf "$CFG"; mkdir -p "$CFG"
claude plugin marketplace add "$CAT"

echo
echo "############ 1. LA CUARENTENA - la condicion de viabilidad"
time claude plugin install flipchart@flipchart-probe
mirar_binario

echo
echo "############ 2. el binario, arrancado POR EL HOST y no por nosotros"
# El lanzador anota modo, xattr y si el binario corre, antes y despues del
# chmod +x, y luego hace de servidor MCP. Solo lo arranca una sesion, no
# `claude plugin ...`; hold.py (prototipo 15) abre una sin llamar al API.
find "$CFG" -name probe.json -delete 2>/dev/null
python3 ../15-plugin-de-juguete/hold.py 30 "$OUT/hold" \
  claude -p --input-format stream-json --output-format stream-json --verbose \
  --debug-file "$OUT/dbg.txt"
grep -i "flipchart" "$OUT/dbg.txt" | head -8
echo "lo que vio el lanzador:"
find "$CFG" -name probe.json -exec cat {} \;

echo
echo "############ 3. el digest que no casa"
claude plugin install digest-malo@flipchart-probe 2>&1 | head -8
echo "y sin sha256 declarado:"
claude plugin install sin-declarar@flipchart-probe 2>&1 | head -4

echo
echo "############ 4. las formas del zip"
for p in envuelto con-finder solo-basura sin-forma; do
  echo "---- $p"; claude plugin install "$p@flipchart-probe" 2>&1 | head -5
done

echo
echo "############ 5. queda git por alguna parte?"
found=$(find "$CFG" -name ".git*" 2>/dev/null | head)
echo "ficheros .git* en el CLAUDE_CONFIG_DIR: ${found:-NINGUNO}"
echo "el catalogo en disco:"
find "$CFG/plugins/marketplaces" | sed "s|$CFG|\$CFG|"
stat -f "  modo=%Sp bytes=%z" "$CFG/plugins/marketplaces/flipchart-probe"
echo "el config dir entero ocupa: $(du -sh "$CFG" | cut -f1)  (con el binario de 42 MB dentro)"

echo
echo "############ 6. /plugin update con la disciplina de #23"
echo "-- a) misma version, mismo digest: no deberia haber nada que hacer"
claude plugin update flipchart 2>&1 | head -4

echo
echo "-- b) SOLO el digest cambia (misma version declarada dentro del zip)"
gh release upload "$TAG" --repo "$REPO" "$OUT/zips/flipchart-1.0.0b.zip" --clobber || true
esperar_asset "$base/$TAG/flipchart-1.0.0b.zip"
S_B=$(sha flipchart-1.0.0b.zip)
catalogo 1.0.0 flipchart-1.0.0b.zip "$base/$TAG" "$S_B"
publicar "$S_B" "el parche sin bump"
claude plugin marketplace update flipchart-probe 2>&1 | tail -2
time claude plugin update flipchart 2>&1 | head -4
echo "   el zip nuevo trae PARCHE.md; llego al disco?"
find "$CFG/plugins" -name PARCHE.md | sed "s|$CFG|\$CFG|" | head -3
echo "   -> vacio = el host lo bajo entero y lo tiro"

echo
echo "-- c) bump completo: version dentro del zip, y version + url + sha256 en la entrada"
gh release create "$TAG2" --repo "$REPO" --title "$TAG2" --notes "sonda v2" \
  "$OUT/zips/flipchart-2.0.0.zip" || \
  gh release upload "$TAG2" --repo "$REPO" --clobber "$OUT/zips/flipchart-2.0.0.zip"
esperar_asset "$base/$TAG2/flipchart-2.0.0.zip"
S_2=$(sha flipchart-2.0.0.zip)
catalogo 2.0.0 flipchart-2.0.0.zip "$base/$TAG2" "$S_2"
publicar "$S_2" "el bump completo"
claude plugin marketplace update flipchart-probe 2>&1 | tail -2
claude plugin update flipchart 2>&1 | head -4
echo "   la cache versionada tras el update:"
ls -1 "$CFG/plugins/cache/flipchart-probe/flipchart" 2>/dev/null | sed 's/^/     /'
mirar_binario

echo
echo "############ 7. los numeros"
for z in flipchart-1.0.0.zip flipchart-1.0.0b.zip flipchart-2.0.0.zip; do
  echo "  $z: $(stat -f '%z' "$OUT/zips/$z") bytes  sha256=$(sha "$z")"
done
echo "  tope del host: 268435456 bytes (256 MiB), plazo 120000 ms sin valvula"
echo "  el CLAUDE_CONFIG_DIR entero: $(du -sh "$CFG" | cut -f1)"
