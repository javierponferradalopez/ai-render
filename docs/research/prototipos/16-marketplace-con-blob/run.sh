#!/bin/bash
# Banco de pruebas de research 10: un marketplace propio cuyo plugin lleva un
# blob de 40 MB dentro, instalado de verdad contra un CLAUDE_CONFIG_DIR aislado.
# No toca la instalacion real de Claude Code y no necesita red ni GitHub.
set -e
LAB="${1:?uso: run.sh <directorio-de-trabajo>}"
mkdir -p "$LAB"
SRC="$(cd "$(dirname "$0")" && pwd)"

# 1. El repo del marketplace, con el blob sintetico (incompresible: peor caso)
rm -rf "$LAB/repo" "$LAB/repo.git" "$LAB/cfg"
mkdir -p "$LAB/repo"
cp -R "$SRC/market/.claude-plugin" "$LAB/repo/.claude-plugin"
mkdir -p "$LAB/repo/plugins"
cp -R "$SRC/plugins/fat" "$LAB/repo/plugins/fat"
mkdir -p "$LAB/repo/plugins/fat/bin"
dd if=/dev/urandom of="$LAB/repo/plugins/fat/bin/blob" bs=1m count=40 2>/dev/null
chmod 755 "$LAB/repo/plugins/fat/bin/blob" "$LAB/repo/plugins/fat/launcher.sh"
git -C "$LAB/repo" init -q -b main
git -C "$LAB/repo" add -A
git -C "$LAB/repo" -c user.name=lab -c user.email=lab@example.com commit -qm "v0.0.1"
git clone --bare -q "$LAB/repo" "$LAB/repo.git"
git -C "$LAB/repo" remote add origin "$LAB/repo.git"

# 2. Un CLAUDE_CONFIG_DIR aislado que declara el marketplace como fuente git.
#    `claude plugin marketplace add` NO acepta file://; extraKnownMarketplaces si.
mkdir -p "$LAB/cfg"
cat > "$LAB/cfg/settings.json" <<JSON
{
  "extraKnownMarketplaces": {
    "fatblob-market": {
      "source": {"source": "git", "url": "file://$LAB/repo.git"}
    }
  }
}
JSON
export CLAUDE_CONFIG_DIR="$LAB/cfg"

# 3. El reconciliador solo corre al arrancar una sesion, no desde `plugin ...`.
#    hold.py (prototipo 15) arranca una sin llamar al API.
python3 "$SRC/../15-plugin-de-juguete/hold.py" 25 "$LAB/hold.out" \
  claude -p --input-format stream-json --output-format stream-json --verbose \
  --debug-file "$LAB/dbg.txt"
grep 'git clone' "$LAB/dbg.txt"

M="$LAB/cfg/plugins/marketplaces/fatblob-market"
echo "== el clon: profundidad, filtro y sparse"
git -C "$M" rev-list --count HEAD
git -C "$M" config --get remote.origin.partialclonefilter || echo "(sin filtro parcial)"
git -C "$M" config --get core.sparseCheckout || echo "(sin sparse-checkout)"
echo "== el blob en el clon"; ls -l "$M/plugins/fat/bin/blob"; xattr -l "$M/plugins/fat/bin/blob"

claude plugin install fat@fatblob-market
C="$LAB/cfg/plugins/cache/fatblob-market/fat/0.0.1"
echo "== el blob en la copia instalada"; ls -l "$C/bin/blob"; xattr -l "$C/bin/blob"
stat -f '%i %Sp %z' "$C/bin/blob" "$M/plugins/fat/bin/blob"   # inodos distintos = copia real

echo "== v0.0.2, para medir que hace el update"
dd if=/dev/urandom of="$LAB/repo/plugins/fat/bin/blob" bs=1m count=40 2>/dev/null
sed -i '' 's/"version": "0.0.1"/"version": "0.0.2"/' \
  "$LAB/repo/plugins/fat/.claude-plugin/plugin.json" "$LAB/repo/.claude-plugin/marketplace.json"
git -C "$LAB/repo" add -A
git -C "$LAB/repo" -c user.name=lab -c user.email=lab@example.com commit -qm "v0.0.2"
git -C "$LAB/repo" push -q origin main

du -sh "$M/.git"
claude plugin marketplace update fatblob-market
du -sh "$M/.git"                      # crecio un binario entero
git -C "$M" rev-list --count HEAD     # el pull no es superficial
claude plugin update fat@fatblob-market
du -sh "$LAB/cfg/plugins"             # el coste total en disco

echo "== que se ve cuando el clon no cabe en el plazo"
mv "$M" "$LAB/discard-market"
CLAUDE_CODE_PLUGIN_GIT_TIMEOUT_MS=50 claude plugin marketplace update fatblob-market || true
