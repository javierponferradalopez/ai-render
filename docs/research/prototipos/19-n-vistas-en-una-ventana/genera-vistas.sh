#!/usr/bin/env bash
# Genera un SVG por fixture con mmdr =0.3.1, que es el motor que el mapa fijo.
# Baja el binario del release verificando el sha256 contra research 08.
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

mkdir -p vistas
for f in fixtures/*.mmd; do
  n=$(basename "$f" .mmd)
  ./mmdr -i "$f" -o "vistas/$n.svg"
  read -r w h < <(python3 -c "
import re,sys
s=open(sys.argv[1]).read()
w=re.search(r'width=\"([0-9.]+)', s); h=re.search(r'height=\"([0-9.]+)', s)
print(w.group(1) if w else '?', h.group(1) if h else '?')
" "vistas/$n.svg")
  printf '  %-14s %8s x %-8s\n' "$n" "$w" "$h"
done
