#!/bin/bash
# La mitad del experimento que NO necesita alojar nada.
#
# Se puede porque la politica de URL del catalogo y la del archive son distintas:
# `oEi` acepta http:// para el catalogo y `I$p` no le aplica `Bfo`, asi que un
# marketplace de fuente `url` se puede servir en loopback. El archive si pasa por
# `Bfo` (https + host no bloqueado), asi que su instalacion de punta a punta esta
# en run-hosted.sh.
#
# Todo ocurre bajo un CLAUDE_CONFIG_DIR propio: la instalacion real no se toca.
set -uo pipefail
cd "$(dirname "$0")"
OUT="${1:-$PWD/out}"
PORT="${PORT:-8791}"
CFG="$OUT/cfg"
SRV="$OUT/srv"
export CLAUDE_CONFIG_DIR="$CFG"

rm -rf "$CFG" "$SRV"; mkdir -p "$CFG" "$SRV"
cp serve.py "$SRV/serve.py"
echo ok > "$SRV/mode"

cleanup() { pkill -f "serve.py $PORT" 2>/dev/null; }
trap cleanup EXIT

catalogo() {  # catalogo <version-de-la-entrada>
  cat > "$SRV/doc.json" <<EOF
{
  "name": "flipchart-probe",
  "owner": { "name": "ai-render research" },
  "plugins": [
    { "name": "por-http", "description": "archive por http", "version": "$1",
      "source": { "source": "archive", "url": "http://127.0.0.1:$PORT/f.zip", "sha256": "$(printf 'a%.0s' {1..64})" } },
    { "name": "por-loopback", "description": "archive https contra 127.0.0.1", "version": "$1",
      "source": { "source": "archive", "url": "https://127.0.0.1:$PORT/f.zip", "sha256": "$(printf 'a%.0s' {1..64})" } },
    { "name": "por-localhost", "description": "archive https contra localhost", "version": "$1",
      "source": { "source": "archive", "url": "https://localhost:$PORT/f.zip", "sha256": "$(printf 'a%.0s' {1..64})" } },
    { "name": "sin-digest", "description": "archive sin sha256", "version": "$1",
      "source": { "source": "archive", "url": "https://example.invalid/f.zip" } },
    { "name": "inexistente", "description": "archive https a un host que no resuelve", "version": "$1",
      "source": { "source": "archive", "url": "https://example.invalid/f.zip", "sha256": "$(printf 'a%.0s' {1..64})" } }
  ]
}
EOF
}

catalogo 1.0.0
python3 "$SRV/serve.py" "$PORT" & sleep 1

echo "############ 1. alta del marketplace por URL (http, loopback)"
claude plugin marketplace add "http://127.0.0.1:$PORT/marketplace.json"

echo
echo "############ 2. queda git por alguna parte?"
found=$(find "$CFG" -name ".git*" 2>/dev/null | head)
echo "ficheros .git* encontrados: ${found:-NINGUNO}"
echo "el catalogo en disco:"
find "$CFG/plugins/marketplaces" | sed "s|$CFG|\$CFG|"
stat -f "  modo=%Sp bytes=%z" "$CFG/plugins/marketplaces/flipchart-probe"
echo "el config dir entero ocupa: $(du -sh "$CFG" | cut -f1)"

echo
echo "############ 3. un cambio en el JSON servido se recoge al refrescar"
catalogo 2.0.0
claude plugin marketplace update flipchart-probe
echo -n "  version en la cache tras el refresco: "
grep -o '"version": "[^"]*"' "$CFG/plugins/marketplaces/flipchart-probe" | head -1

echo
echo "############ 4. la URL sirve basura"
echo garbage > "$SRV/mode"
claude plugin marketplace update flipchart-probe
echo -n "  la cache sobrevive al fallo? version: "
grep -o '"version": "[^"]*"' "$CFG/plugins/marketplaces/flipchart-probe" | head -1

echo
echo "############ 5. la URL responde HTTP 500"
echo 500 > "$SRV/mode"
claude plugin marketplace update flipchart-probe

echo
echo "############ 6. la URL no responde"
cleanup; sleep 1
claude plugin marketplace update flipchart-probe

echo
echo "############ 7. la politica de URL del archive, entrada por entrada"
echo ok > "$SRV/mode"
python3 "$SRV/serve.py" "$PORT" & sleep 1
catalogo 1.0.0
claude plugin marketplace update flipchart-probe > /dev/null 2>&1
for p in por-http por-loopback por-localhost sin-digest inexistente; do
  echo "---- $p"
  claude plugin install "$p@flipchart-probe" 2>&1 | head -4
done

echo
echo "############ 8. el caudal real del CDN de GitHub (la division del apartado 7)"
A=$(gh api repos/oven-sh/bun/releases/latest --jq '.assets[] | select(.name=="bun-darwin-aarch64.zip") | .browser_download_url' 2>/dev/null)
if [ -n "$A" ]; then
  curl -sL -o /dev/null -w "  %{size_download} bytes en %{time_total}s -> %{speed_download} B/s, redirecciones=%{num_redirects}\n" "$A"
  echo "  cadena de redireccion:"
  curl -s -o /dev/null -D - -L --max-redirs 5 "$A" | grep -i "^location:" | sed 's/?.*$//' | sed 's/^/    /'
else
  echo "  (sin red o sin gh: saltado)"
fi
