# Prototipo 17 — el vehículo del zip verificado

Banco de pruebas de [research 11](../../11-el-zip-verificado-y-lo-que-promete-el-esquema.md)
y de [research 15](../../15-la-cuarentena-medida-y-el-veredicto-del-vehiculo.md).
Pone a prueba lo que decidió [¿El binario en un zip verificado en vez de dentro
del repo?](https://github.com/javierponferradalopez/ai-render/issues/23):
`marketplace: source "url"` + `plugin: source "archive"`.

**Las dos mitades están corridas.** La local, en macOS 26.6.2 (build 25G83) arm64 con
Claude Code **2.1.228**; la alojada, el 2026-09-03 en macOS 26.5.2 (build 25F84) arm64
con Claude Code **2.1.259**. clang 21.0.0 y python3 3.9.6 de las Command Line Tools.

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
| El archive de punta a punta, con alojamiento real | `run-hosted.sh` | **ejecutado** — contra `javierponferradalopez/flipchart-archive-probe` |

## Las piezas

| Fichero | Para qué |
|---|---|
| `build.sh` | Construye el material: el Mach-O de 42 MB firmado ad-hoc, el plugin a su alrededor, las cinco formas de zip y los dos zips del `update` (mismo `version` con otros bytes, y el bump completo) |
| `probe-main.c` | El binario sonda. Lleva 40 MB de `/dev/urandom` en una sección `__DATA,__blob` — tiene que ser un Mach-O de verdad, porque un `.sh` no dispara Gatekeeper y mediría un falso negativo |
| `plugin/` | Manifiesto, `.mcp.json` y el lanzador |
| `plugin/launcher.sh` | Anota modo, `xattr`, cuarentena y si el binario **corre** antes y después del `chmod +x`; luego hace de Servidor de aviso en bash pelado |
| `serve.py` | Sirve el catálogo en loopback y obedece a un fichero `mode`: `ok`, `garbage`, `500` |
| `run-local.sh` | La mitad ejecutable: alta por URL, ausencia de git, refresco, tres modos de fallo, la política del archive entrada por entrada, y el caudal real del CDN |
| `run-hosted.sh` | La mitad alojada: publica el material y mide cuarentena, arranque por el host, digest, formas de zip y los tres casos del `update` |
| `registro-hosted.log` | La corrida alojada del 2026-09-03 entera, tal cual: es la evidencia de research 15 |
| `js.sh` | Extrae a `out/js.txt` la fuente JS minificada de la cola del ejecutable, y comprueba que ha salido entera |
| `peek.sh`, `look.sh` | Lectura del bundle. `peek.sh` saca la tabla de cadenas del bytecode; `look.sh` busca sobre el `js.txt` de `js.sh` |

## Cómo se lee el bundle

El ejecutable **no** es JS: el código de Claude Code está compilado a bytecode, y lo
único legible ahí es su tabla de cadenas (`peek.sh`). Pero la **fuente JS minificada sí
está**, en la cola del fichero — a partir de unos 250 MB en los 289 MB del 2.1.228, y de
unos 158 MB en los 200 MB del 2.1.259. Eso lo saca `./js.sh`, y sobre su `out/js.txt`
trabaja `look.sh`, que imprime una ventana de caracteres alrededor de cada coincidencia.
De ahí salen todas las citas de research 11 y 15.

```sh
./js.sh          # o ./js.sh 240 para cortar más abajo
./look.sh 'responseType:"arraybuffer"'
```

**Y una trampa que costó una tarde:** la receta vieja era
`dd … | strings -n 1 > js.txt`, y **eso no vale**. El `strings` de macOS parsea Mach-O,
y sobre este ejecutable devuelve 11 MB de los 43 que hay, **sin las cadenas que se
buscan** — así que `look.sh` no encuentra nada y parece que el host ha cambiado el
código, cuando lo que ha pasado es que no se ha leído. `js.sh` usa `tr`, que no
interpreta el formato, y verifica que la fuente está ahí antes de darse por bueno.
Ni `js.sh` ni `peek.sh` clavan ya la ruta del ejecutable: resuelven el `claude` vivo,
que en esta máquina ya no es el del Caskroom sino
`~/.local/share/claude/versions/2.1.259`.

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

## Cómo se corre la mitad alojada

```sh
./build.sh
./run-hosted.sh <owner>/flipchart-archive-probe
```

Crea el repo público, publica dos releases (~126 MB de assets), instala contra un
`CLAUDE_CONFIG_DIR` propio y mide los ocho apartados. Tarda unos 10 minutos, casi todos
esperando al CDN. Deja el repo vivo a propósito — borrarlo pide el scope `delete_repo`,
que el token de `gh` de esta máquina no trae (`gh auth refresh -h github.com -s delete_repo`).

## Avisos de la mitad alojada, que costaron una corrida entera

- **`gh release create` sobre un repo recién creado falla con `422 Repository is empty`.**
  Un repo sin un solo commit no puede tener releases, y entonces las siete entradas del
  catálogo dan 404 y la corrida entera sale «medida» sin haber medido nada. Por eso el
  script publica el `marketplace.json` **antes** que el release: ese push es lo que crea
  `main`.
- **`raw.githubusercontent.com` tarda de verdad.** Medido: **180 s** y **275 s** en
  servir un catálogo recién empujado. Refrescar antes mide la corrida anterior sin
  decirlo, así que `publicar` no vuelve hasta ver su propia marca, con seis minutos de
  paciencia.
- **El servidor de aviso del Lanzador no completa el handshake** y la sesión corta con
  `CONNECT_TIMEOUT` a los 30 s. Es del arnés y no del vehículo: lo que interesa —que el
  host arranque el Lanzador y éste ejecute el binario— ya está en el `probe.json` antes
  de eso.
