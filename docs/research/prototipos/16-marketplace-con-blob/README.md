# Prototipo 16 — un marketplace cuyo plugin lleva un blob de 40 MB

Banco de pruebas de [research 10](../../10-el-binario-por-el-clon-del-marketplace.md).
Mide qué le pasa a un binario grande que viaja **dentro** del repo de un plugin:
si llega, si llega entero y ejecutable, cuánto ocupa, y qué hace `update`.

Medido en macOS 26.6.2 arm64 con Claude Code **2.1.228** y git 2.50.1.

## Por qué no hace falta GitHub

El experimento nació pidiendo un repo público de 40 MB para medir el clon de
verdad. No hizo falta: un marketplace de fuente `git` apuntando a un **bare repo
local por `file://`** recorre exactamente el mismo camino de código —el mismo
`git clone`, el mismo timeout, la misma copia a la caché— sin publicar nada. Lo
único que ese atajo no mide es el tiempo de red, que es una división.

Dos avisos de mecánica que costaron encontrar:

- **`claude plugin marketplace add` no acepta `file://`** (*"Invalid marketplace
  source format. Try: owner/repo, https://..., or ./path"*), y una ruta local se
  toma como fuente `directory`, que no clona nada. La vía es declarar el
  marketplace en `extraKnownMarketplaces` dentro de `settings.json`.
- **Los subcomandos `claude plugin …` no reconcilian `extraKnownMarketplaces`.**
  Hasta que no arranca una sesión, el marketplace "no existe" y tanto `install`
  como `marketplace update` fallan con *"not found"*. Por eso `run.sh` levanta
  una sesión con el `hold.py` del [prototipo 15](../15-plugin-de-juguete/), que
  arranca Claude Code sin llamar al API.

Todo ocurre bajo un `CLAUDE_CONFIG_DIR` propio, así que la instalación real del
usuario no se toca.

## Las piezas

| Fichero | Para qué |
|---|---|
| `market/.claude-plugin/marketplace.json` | El marketplace, con `fat` como plugin de ruta relativa |
| `plugins/fat/` | El plugin: manifiesto, `.mcp.json` y un lanzador que anota lo que ve del blob |
| `plugins/fat/launcher.sh` | Registra `blob_exists`, tamaño, modo, bit de ejecución y `xattr`, y responde al handshake MCP en bash pelado |
| `run.sh` | El experimento entero, de cero a la medición del timeout |

El blob **no está en el repo**: lo genera `run.sh` con `dd` de `/dev/urandom` —
datos incompresibles, que es el peor caso para git y para la red. Un binario real
comprime, así que los números de aquí son un techo.

## Cómo se corre

```sh
./run.sh /tmp/fatblob-lab
```
