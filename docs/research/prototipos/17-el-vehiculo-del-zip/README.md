# Prototipo 17 — el vehículo del zip verificado

Banco de pruebas de [research 11](../../11-el-zip-verificado-y-lo-que-promete-el-esquema.md).
Pone a prueba lo que decidió [¿El binario en un zip verificado en vez de dentro
del repo?](https://github.com/javierponferradalopez/ai-render/issues/23) leyendo el
bundle y sin ejecutar nada: `marketplace: source "url"` + `plugin: source "archive"`.

Medido en macOS 26.6.2 (build 25G83) arm64 con Claude Code **2.1.228**, clang 21.0.0
y python3 3.9.6 de las Command Line Tools.

## Por qué el banco está partido en dos

A diferencia del [prototipo 16](../16-marketplace-con-blob/), aquí **no hay atajo
por `file://`**. La política de URL del archive es explícita:

```js
function Bfo(e){ try{ let t=new URL(e); return t.protocol==="https:" && !Gin(t.hostname) }catch{ return !1 } }
```

`https:` obligatorio y host no bloqueado — y `Gin` bloquea `localhost`,
`ip6-localhost`, `.localhost`, literales IPv6, IPv4 no canónica, la dirección no
especificada, loopback, link-local y los hosts de metadatos de nube
(`metadata.google.internal`, `instance-data.ec2.internal`, `100.100.100.200`,
`168.63.129.16`, `192.0.0.192`). Se revalida en cada uno de los hasta 5 saltos de
redirección.

**El catálogo, en cambio, no pasa por esa política.** Su parser acepta `http://`
igual que `https://`, y su descarga (`I$p`) no llama a `Bfo` — sólo instala un
guardián de redirecciones que exime el salto que se queda en el mismo origen. De
ahí el corte:

| Mitad | Script | Estado |
|---|---|---|
| El catálogo por URL, de punta a punta, en loopback | `run-local.sh` | **ejecutado** |
| El archive de punta a punta, con alojamiento real | `run-hosted.sh` | **sin ejecutar** — pide un repo público |

## Las piezas

| Fichero | Para qué |
|---|---|
| `build.sh` | Construye el material: el Mach-O de 42 MB firmado ad-hoc, el plugin a su alrededor y las cinco formas de zip |
| `probe-main.c` | El binario sonda. Lleva 40 MB de `/dev/urandom` en una sección `__DATA,__blob` — tiene que ser un Mach-O de verdad, porque un `.sh` no dispara Gatekeeper y mediría un falso negativo |
| `plugin/` | Manifiesto, `.mcp.json` y el lanzador |
| `plugin/launcher.sh` | Anota modo, `xattr`, cuarentena y si el binario **corre** antes y después del `chmod +x`; luego hace de Servidor de aviso en bash pelado |
| `serve.py` | Sirve el catálogo en loopback y obedece a un fichero `mode`: `ok`, `garbage`, `500` |
| `run-local.sh` | La mitad ejecutable: alta por URL, ausencia de git, refresco, tres modos de fallo, la política del archive entrada por entrada, y el caudal real del CDN |
| `run-hosted.sh` | La mitad que falta, ya escrita: publica el material y mide cuarentena, digest, formas de zip y `update` |
| `peek.sh`, `look.sh` | Lectura del bundle. `peek.sh` saca la tabla de cadenas del bytecode; `look.sh` busca sobre la fuente JS minificada, que vive en la cola del ejecutable |

## Cómo se lee el bundle

El ejecutable son 289 MB y **no** es JS: el código de Claude Code está compilado a
bytecode, y lo único legible ahí es su tabla de cadenas (`peek.sh`). Pero la
**fuente JS minificada sí está**, en la cola del fichero a partir de unos 250 MB:

```sh
dd if=/opt/homebrew/Caskroom/claude-code/2.1.228/claude bs=1M skip=240 | strings -n 1 > js.txt
```

Sobre ese `js.txt` trabaja `look.sh`, que imprime una ventana de caracteres
alrededor de cada coincidencia. Es de donde salen todas las citas de research 11.

## Avisos de mecánica que costaron encontrar

- **El puerto se queda pegado.** Un `serve.py` de una corrida anterior sigue vivo y
  la siguiente lo hereda sirviendo el catálogo viejo: los pasos 3–5 salen
  «correctos» sin haber medido nada. `run-local.sh` mata el suyo con un `trap`,
  pero conviene un `pkill -f serve.py` antes de empezar.
- **`claude plugin marketplace add` sí acepta una URL** (a diferencia del `file://`
  del prototipo 16), así que aquí no hace falta el rodeo por
  `extraKnownMarketplaces` ni el `hold.py` del [prototipo 15](../15-plugin-de-juguete/):
  el alta escribe `known_marketplaces.json` y el `install` la encuentra sin que
  arranque ninguna sesión.
- **La política de URL del archive se aplica al validar el esquema**, no al
  descargar. Una entrada con `url` prohibida no da un error de red: se convierte en
  `source: "unsupported"` y el fallo aparece al instalar, como *«This plugin's
  marketplace entry is invalid: source.url: …»*. El resto del catálogo carga bien.

## Lo que queda por correr

`run-hosted.sh` necesita un repo público desechable y permiso para publicarlo:

```sh
./build.sh
./run-hosted.sh <owner>/flipchart-archive-probe
```

Deja el repo vivo a propósito — borrarlo pide el scope `delete_repo`, que el token
de `gh` de esta máquina no trae (`gh auth refresh -h github.com -s delete_repo`).

Sus apartados 5b y 5c piden dos pasos manuales que el script no puede inventar:

- **5b — sólo cambia el digest.** Subir un segundo zip como asset y reescribir
  `url` + `sha256` de la entrada **sin tocar** `version` ni el `version` de
  `plugin.json`. Lo esperado, por lectura: el host **descarga el zip nuevo entero**
  y luego dice *«already at the latest version»*, tirándolo.
- **5c — bump completo.** Subir `version` en `plugin.json`, reempaquetar, segundo
  release, y `version` + `url` + `sha256` en la entrada. Lo esperado: *«updated from
  1.0.0 to 2.0.0»*.
