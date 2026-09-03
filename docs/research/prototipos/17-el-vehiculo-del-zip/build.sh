#!/bin/bash
# Construye el material del banco: un Mach-O real de ~42 MB firmado ad-hoc, el
# plugin sonda a su alrededor, y las cinco formas de zip que se quieren probar.
#
# El binario tiene que ser un Mach-O de verdad y no un script: un `.sh` no
# dispara Gatekeeper, asi que medir la cuarentena sobre un script daria un
# falso negativo. Y su relleno sale de /dev/urandom para que sea incompresible,
# que es el peor caso para el limite de 256 MB y para el plazo de 120 s.
set -euo pipefail
cd "$(dirname "$0")"
OUT="${1:-$PWD/out}"
mkdir -p "$OUT/zips"

echo "== 1. binario: Mach-O de 42 MB con una seccion incompresible, firmado ad-hoc"
dd if=/dev/urandom of="$OUT/blob.bin" bs=1m count=40 2>/dev/null
clang -O0 -o "$OUT/flipchart" probe-main.c -Wl,-sectcreate,__DATA,__blob,"$OUT/blob.bin"
codesign -s - -f "$OUT/flipchart"
codesign -dv "$OUT/flipchart" 2>&1 | grep -E "Signature|Format"
"$OUT/flipchart" || true
shasum -a 256 "$OUT/flipchart"

echo "== 2. el plugin sonda alrededor del binario"
rm -rf "$OUT/plugin"
cp -R plugin "$OUT/plugin"
mkdir -p "$OUT/plugin/bin"
cp "$OUT/flipchart" "$OUT/plugin/bin/flipchart"
chmod +x "$OUT/plugin/launcher.sh" "$OUT/plugin/bin/flipchart"

echo "== 3. zip plano, el que se publicaria de verdad"
( cd "$OUT/plugin" && zip -q -r -X "$OUT/zips/flipchart-1.0.0.zip" . )
unzip -Z "$OUT/zips/flipchart-1.0.0.zip"
echo "sha256 del zip:"; shasum -a 256 "$OUT/zips/flipchart-1.0.0.zip"

echo "== 4. las cuatro variantes pequenas de forma"
T="$OUT/tiny"; rm -rf "$T"; mkdir -p "$T/plain/.claude-plugin"
printf '%s\n' '{ "name": "tinyprobe", "description": "sonda pequena", "version": "1.0.0" }' \
  > "$T/plain/.claude-plugin/plugin.json"
echo hola > "$T/plain/README.md"

# a) todo dentro de un unico directorio envoltorio -> debe promocionarse
mkdir -p "$T/w"; cp -R "$T/plain" "$T/w/flipchart-1.0.0"
( cd "$T/w" && zip -q -r -X "$OUT/zips/t-wrapped.zip" . )

# b) basura del Finder junto al contenido -> debe ignorarse
mkdir -p "$T/f/__MACOSX"; cp -R "$T/plain/." "$T/f/"
printf 'x' > "$T/f/.DS_Store"; printf 'y' > "$T/f/__MACOSX/._plugin.json"
( cd "$T/f" && zip -q -r -X "$OUT/zips/t-finder.zip" . )

# c) solo basura del Finder -> "contained no plugin files"
mkdir -p "$T/e/__MACOSX"; printf 'y' > "$T/e/__MACOSX/._x"; printf 'x' > "$T/e/.DS_Store"
( cd "$T/e" && zip -q -r -X "$OUT/zips/t-empty.zip" . )

# d) contenido sin forma de plugin -> "no plugin content at its root"
mkdir -p "$T/n"; echo hola > "$T/n/cualquier-cosa.txt"
( cd "$T/n" && zip -q -r -X "$OUT/zips/t-noplugin.zip" . )

echo "== 5. el material del update: los dos casos que separan digest de version"
# a) mismo `version` dentro del plugin.json, bytes distintos -> otro sha256.
#    Es el arreglo publicado sin bump: el host baja el zip y deberia tirarlo.
rm -rf "$OUT/plugin-b"; cp -R "$OUT/plugin" "$OUT/plugin-b"
printf 'republicado sin subir la version\n' > "$OUT/plugin-b/PARCHE.md"
( cd "$OUT/plugin-b" && zip -q -r -X "$OUT/zips/flipchart-1.0.0b.zip" . )

# b) bump completo: el `version` que manda es el de dentro del zip.
rm -rf "$OUT/plugin2"; cp -R "$OUT/plugin" "$OUT/plugin2"
sed -i '' 's/"version": "1.0.0"/"version": "2.0.0"/' \
  "$OUT/plugin2/.claude-plugin/plugin.json"
( cd "$OUT/plugin2" && zip -q -r -X "$OUT/zips/flipchart-2.0.0.zip" . )

ls -l "$OUT/zips"
for z in flipchart-1.0.0.zip flipchart-1.0.0b.zip flipchart-2.0.0.zip; do
  shasum -a 256 "$OUT/zips/$z"
done
echo
echo "Listo. El material esta en $OUT."
echo "La mitad local se corre con ./run-local.sh; la alojada, con ./run-hosted.sh."
