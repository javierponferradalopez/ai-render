#!/usr/bin/env bash
# Que hace mmdr con lo que Mermaid permite y la pizarra no deberia aceptar.
#
# Baja el binario del release v0.3.1 (sha256 verificado contra research 08),
# y corre los tres casos de fixtures/. Ninguno necesita red mas alla de la
# descarga, y ninguno publica nada.
set -euo pipefail
cd "$(dirname "$0")"

SHA_ESPERADO=562d0250cb8588adefe398a23e4bbdf67f242849ea8d888c38268bcc3edf3223
URL=https://github.com/1jehuang/mermaid-rs-renderer/releases/download/v0.3.1/mmdr-aarch64-apple-darwin.tar.gz

if [ ! -x ./mmdr ]; then
  curl -sL "$URL" -o mmdr.tar.gz
  SHA_REAL=$(shasum -a 256 mmdr.tar.gz | cut -d' ' -f1)
  [ "$SHA_REAL" = "$SHA_ESPERADO" ] || { echo "sha256 no casa: $SHA_REAL"; exit 1; }
  tar xzf mmdr.tar.gz && rm mmdr.tar.gz
fi
echo "mmdr $(./mmdr --version)"

textos() { python3 -c "
import re,sys
s=open(sys.argv[1]).read()
print('   textos:', [x for x in re.findall(r'<text[^>]*>(?:<tspan[^>]*>)?([^<]*)', s) if x.strip()])
" "$1"; }

echo
echo "== 1. Constructos de pixel: classDef / style / linkStyle =="
echo "   Esperado si la decision de partida 5 estuviera defendida: rechazo o descarte."
./mmdr -i fixtures/pixel.mmd -o /tmp/pixel.svg
echo "   exit=$? ; colores en el SVG:"
grep -o 'fill="#[0-9a-f]\{3,6\}"\|stroke="#[0-9a-f]\{3,6\}"' /tmp/pixel.svg | sort | uniq -c | sed 's/^/     /'

echo
echo "== 2. Basura sintactica =="
echo "   Esperado si fallara cerrado: error y ningun dibujo."
./mmdr -i fixtures/basura.mmd -o /tmp/basura.svg
echo "   exit=$?"
textos /tmp/basura.svg

echo
echo "== 3. Un id con typo, que es la mentira plausible =="
echo "   'Ordr --> Money' con 'Order' declarada."
./mmdr -i fixtures/typo.mmd -o /tmp/typo.svg
echo "   exit=$?"
textos /tmp/typo.svg
