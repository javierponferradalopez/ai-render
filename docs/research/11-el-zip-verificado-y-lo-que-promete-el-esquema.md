# El zip verificado y lo que promete el esquema

Investigación del 2026-09-02 para ejecutar lo que [¿El binario en un zip verificado
en vez de dentro del repo?][23] decidió leyendo: que el binario de flipchart puede
viajar en un **zip por HTTPS con `sha256` verificado** (`source: "archive"`) servido
desde un catálogo que es **un JSON por URL** (`source: "url"`), y que con eso **en el
cliente no queda git por ninguna parte**.

**La cuarentena no muerde: no puede.** El zip descargado **nunca toca el disco** —viaja
como `arraybuffer` en memoria—, la extracción escribe ficheros nuevos con
`fs.writeFile`, y la única mención de `com.apple.quarantine` en los 289 MB del bundle
está dentro de un documento de skill ajeno. No hay fichero cuarentenado del que
heredar el atributo ni código que lo ponga. El vehículo se sostiene.

> **Cerrado el 2026-09-03 por [research 15][r15]:** la mitad que faltaba está corrida
> contra un release alojado de verdad. No hay cuarentena, el binario llega `100755` y
> **corre**. Las cinco cosas que aquí quedaron leídas están ahora ejecutadas, y el
> vehículo se sostiene.

**Pero eso es lectura, no medición, y esta investigación se ha quedado a medias.** La
mitad del catálogo está ejecutada de punta a punta; la del archive **no**, porque su
política de URL prohíbe el atajo local que salvó a [research 10][r10] y exige alojar
de verdad. Lo que sí ha aparecido, leyendo, son **tres cosas que el mapa no sabía**, y
una de ellas es cara: **`/plugin update` descarga el zip entero antes de comprobar si
hay algo que actualizar.**

## Nota sobre el método

macOS 26.6.2 (build 25G83) arm64, Claude Code **2.1.228**, clang 21.0.0, python3 3.9.6
de las Command Line Tools. El banco de pruebas es el
[prototipo 17](./prototipos/17-el-vehiculo-del-zip/), y `run-local.sh` reproduce entero
lo que aquí se marca como ejecutado, contra un `CLAUDE_CONFIG_DIR` aislado.

Tres niveles, y conviene no confundirlos:

- **Ejecutado** — corrido de verdad: el catálogo por URL completo (alta, refresco,
  tres modos de fallo), la política de URL del archive entrada por entrada, el
  empaquetado del zip y el caudal del CDN.
- **Leído del bundle** — con la cita del código: la cuarentena, la extracción, el bit
  de ejecución, el `update` y las cuatro formas de rechazo del zip.
- **Sin correr** — el archive de punta a punta. Sigue escrito y listo en
  `run-hosted.sh`.

Una nota sobre cómo se lee el bundle, porque cambia respecto a research 09 y 10: el
ejecutable está compilado a bytecode y ahí sólo se ve su tabla de cadenas, pero **la
fuente JS minificada está entera en la cola del fichero**, a partir de unos 250 MB.
Todas las citas de este documento son de esa fuente, no reconstrucciones.

## 1. Por qué la mitad del archive no se ha corrido

La política de URL del archive no admite rodeos:

```js
function Bfo(e){ try{ let t=new URL(e); return t.protocol==="https:" && !Gin(t.hostname) }catch{ return !1 } }
```

`https:` obligatorio, y `Gin` bloquea `localhost`, `ip6-localhost`, `ip6-loopback`,
`.localhost`, literales IPv6, IPv4 no canónica, la dirección no especificada, loopback,
link-local y los hosts de metadatos de nube (`metadata.google.internal`,
`instance-data.ec2.internal`, `100.100.100.200`, `168.63.129.16`, `192.0.0.192`). Se
revalida en **cada** salto de redirección, hasta 5.

El truco de [research 10][r10] —un bare repo local por `file://` recorriendo el mismo
camino de código— aquí no existe. Hace falta un host real.

**El catálogo, en cambio, sí se puede servir en casa**, y ése es el hallazgo que
rescató la mitad del experimento. Su parser acepta los dos protocolos:

```js
if (t.startsWith("http://") || t.startsWith("https://")) { … return {source:"url", url:a} }
```

y su descarga (`I$p`) **no llama a `Bfo`**: sólo instala un guardián de redirecciones
que exime el salto que se queda en el mismo origen. Así que la mitad que *«mata el git
entero»* —la que a #23 le importaba— se ha medido sin publicar nada.

## 2. El catálogo por URL: el git desaparece, y con él la factura de #22

**Ejecutado.** `claude plugin marketplace add http://127.0.0.1:8791/marketplace.json`:

```
Downloading marketplace from http://127.0.0.1:8791/marketplace.json
Validating marketplace data
Saving marketplace to cache
✔ Successfully added marketplace: flipchart-probe (declared in user settings)
```

Y lo que queda en disco:

| Medida | Valor |
|---|---|
| Ficheros `.git*` en todo el `CLAUDE_CONFIG_DIR` | **ninguno** |
| El catálogo en disco | `plugins/marketplaces/flipchart-probe`, **un fichero plano**, `-rw-r--r--`, 1548 bytes |
| El `CLAUDE_CONFIG_DIR` entero | **20 K** |

No es un directorio con un clon dentro: es el JSON tal cual, sin extensión. El código
lo confirma —`P$p` hace `mkdir` del padre y escribe el fichero— y no hay ninguna rama
que invoque git.

Contra esto, la factura que midió [¿Pasa un binario de decenas de MB por el clon de un
marketplace?][22]: `B × (N + 2)` permanente, 200 MB en disco para un binario de 40 MB,
y nada que pode el `.git` jamás. **No se mitiga: deja de existir.** Queda `B`.

**El refresco recoge los cambios.** Cambiada la `version` del JSON servido de `1.0.0` a
`2.0.0`, `claude plugin marketplace update` vuelve a descargar, revalida y sobreescribe;
la caché queda en `2.0.0`.

**Y los tres modos de fallo son de primer plano, con el mensaje diciendo qué pasa** —y,
al contrario que el baile de `.bak` y re-clon del camino de git, **la caché anterior
sobrevive intacta al fallo**:

| Lo que sirve la URL | Lo que se ve |
|---|---|
| Basura, no JSON | `Failed to refresh marketplace 'flipchart-probe': Invalid marketplace schema from URL: : Invalid input: expected object, received string` |
| HTTP 500 | `HTTP 500 error while downloading marketplace from … The marketplace file may not exist at this URL.` |
| Nada (servidor caído) | `Could not connect to … Please check your internet connection and verify the URL is correct.` + `connect ECONNREFUSED` |

### Y un número que corrige a #23

[#23][23] anotó *«256 MB y 120 s sin válvula»* y lo dio por bueno para todo el camino.
Lo es para el **archive**:

```js
var jEt = 268435456, c$o = 5242880, VF_ = 120000, z9s = "Claude-Code-Plugin-Manager";
```

`maxContentLength: jEt` = **256 MiB**, `timeout: VF_` = **120 000 ms**, y confirmado
que no hay variable de entorno que lo mueva. Pero **el catálogo tiene otros números**:

```js
i = await NS.get(e, { timeout: 1e4, maxContentLength: c$o, headers: o, beforeRedirect: fkd(e, W9s(t)) })
```

**10 segundos y 5 MiB.** Para un `marketplace.json` de 1,5 KB sobra, y la espera de
#21 —*«en primer plano, 120 s, reintentable»*— sigue siendo cierta en su parte que
importa (primer plano, reintentable), pero el plazo real del alta es **doce veces más
corto** de lo que el mapa cree. Y el tope de 5 MiB es un techo que nadie había escrito:
un catálogo enorme no cabe.

## 3. La política de URL, medida entrada por entrada

**Ejecutado.** Cinco entradas `archive` en el mismo catálogo, instaladas una a una:

| Entrada | `url` | Resultado |
|---|---|---|
| `por-http` | `http://127.0.0.1:8791/f.zip` | rechazada |
| `por-loopback` | `https://127.0.0.1:8791/f.zip` | rechazada |
| `por-localhost` | `https://localhost:8791/f.zip` | rechazada |
| `sin-digest` | `https://example.invalid/f.zip`, **sin `sha256`** | pasa la política, falla la red |
| `inexistente` | `https://example.invalid/f.zip` con `sha256` | pasa la política, falla la red |

Dos cosas que sólo se ven corriéndolo:

**La política se aplica al validar el esquema, no al descargar.** El mensaje no es un
error de red, es de entrada inválida:

```
✘ Failed to install plugin "por-http@flipchart-probe": This plugin's marketplace entry
  is invalid: source.url: Archive URLs must use https:// and must not point at a
  loopback, link-local, or cloud-metadata host
```

La entrada se convierte en `source: "unsupported"` con ese texto dentro, y **el resto
del catálogo carga bien**: una entrada mal formada no envenena a las demás. Para
flipchart importa poco —su URL será buena— pero significa que un error en el
`marketplace.json` generado se ve al instalar y sólo en el plugin afectado.

**`sha256` es opcional en el esquema.** `sin-digest` parseó y llegó a intentar la
descarga. Que flipchart lo declare es disciplina propia, no obligación del host — y sin
él la verificación de integridad simplemente no ocurre (`if (t.sha256 && …)`).

Y el fallo de descarga es de primer plano y accionable:

```
✘ Failed to install plugin "sin-digest@flipchart-probe": Could not connect to
  https://example.invalid/f.zip. Check your network connection and that the archive
  URL is correct.

Technical details: getaddrinfo ENOTFOUND example.invalid
```

## 4. La cuarentena: no es que no aparezca, es que no puede

**Leído del bundle.** Era la condición de viabilidad del ticket, y la respuesta está en
una sola línea de la descarga:

```js
let c = await i(e, { timeout: VF_, responseType: "arraybuffer", maxRedirects: 5,
                     maxContentLength: jEt, headers: o, beforeRedirect: KF_(e, W9s(n)) });
… a = Buffer.from(c.data)
```

**`responseType: "arraybuffer"`. El zip nunca se escribe en disco.** Y la extracción
escribe ficheros nuevos, uno a uno, con `fs.writeFile`:

```js
async function J_a(e,t,r){
  let n = await MSt(e), o = JSr(e);
  await fr().mkdir(t);
  for (let [s,a] of Object.entries(n)) {
    …
    let l = Uve.join(t,s);
    await fr().mkdir(Uve.dirname(l)), await uce.writeFile(l, a);
    …
  }
}
```

`com.apple.quarantine` lo pone la aplicación que descarga, a través de LaunchServices,
sobre el fichero que deja en disco; se hereda al desempaquetar sólo si hay un fichero
cuarentenado del que heredarlo y un desempaquetador que lo propague. Aquí no hay ni lo
uno ni lo otro. Y buscando el atributo en los 289 MB del ejecutable aparece **una sola
vez**, dentro de un documento de skill sobre el CLI `ant` de Anthropic
(`xattr -d com.apple.quarantine "$(brew --prefix)/bin/ant"`): **el código de instalación
de plugins no lo menciona en ninguna parte**, ni para ponerlo ni para quitarlo.

Esto es más fuerte que lo que midió [#22][22] para el clon —allí era una observación,
aquí es una imposibilidad estructural— pero **sigue siendo lectura**. El ticket pedía
mirar el `xattr` sobre el fichero extraído, en la copia versionada de la caché, y
**correrlo de verdad**; eso está en `run-hosted.sh` y no se ha corrido. Con un binario
firmado ad-hoc y sin notarizar, el ejecutarlo es el único juez definitivo.

## 5. El bit de ejecución **sí** se preserva: la sospecha de fflate es medio falsa

El ticket sospechaba que el descompresor es **fflate**, que no transporta el modo Unix,
y que por eso el binario llegaría `644`. **La primera mitad es cierta; la conclusión, no.**

fflate está confirmado por su lista literal de errores en el bundle —`unexpected EOF`,
`invalid block type`, `invalid length/literal`, `invalid distance`, `stream finished`,
`no stream handler`, `invalid UTF-8 data`, `extra field too long`,
`date not in range 1980-2099`, `filename too long`, `invalid zip data`— y por su uso:

```js
async function MSt(e){ let {unzipSync:t} = await …; return t(new Uint8Array(e), {filter: …}) }
```

Pero junto a él hay un **segundo parser, propio, que sí lee los modos**: `JSr` recorre
el directorio central del zip (EOCD `0x06054b50`, cabeceras `0x02014b50`) y saca el modo
de los atributos externos, **sólo si el zip lo hizo un empaquetador Unix**:

```js
let l = t.readUInt16LE(s+4), … p = t.readUInt32LE(s+38), f = t.toString("utf8", s+46, s+46+c);
if (l>>8 === 3) { let m = p>>>16 & 65535; if (m) r[f] = m }
```

y `J_a` lo aplica cuando hay algún bit de ejecución puesto:

```js
let c = o[s];
if (c && c & 73) await uce.chmod(l, c & 511).catch(()=>{})
```

`73` es `0o111`. **Ejecutado** en el lado del zip: Info-ZIP 3.0 de macOS produce
`version made by = 3` (Unix) y guarda los modos tal cual —

```
-rwxr-xr-x  3.0 unx     1987 t- defN  launcher.sh
-rwxr-xr-x  3.0 unx 42076400 b- defN  bin/flipchart
-rw-r--r--  3.0 unx      188 t- defN  .mcp.json
```

— así que lo esperado es que el binario aterrice **`0755`**, no `644`.

Consecuencia para [#23][23]: su `chmod +x` en el Lanzador **deja de ser lo que sostiene
la ejecución y pasa a ser un cinturón sobre los tirantes**. Conviene conservarlo, y por
una razón concreta: la preservación depende de **quién empaqueta**. Un zip hecho por una
herramienta que no escribe atributos Unix —o desde Windows— no lleva modo, `JSr` no
devuelve nada y el fichero cae al `writeFile` pelado. Si el `marketplace.json` se genera
en CI, el `zip` de esa CI es parte del contrato.

## 6. `/plugin update` descarga el zip entero antes de saber si hace falta

**Leído del bundle.** Es el hallazgo caro, y ninguna decisión del mapa lo contemplaba.
La identidad de versión de un plugin la calcula `IIe`, con esta precedencia:

```js
async function IIe(e,t,r,n,o,i,s){
  if (r?.version) return r.version;                 // 1. el manifiesto del plugin
  if (o) return o;                                  // 2. el `version` de la entrada
  if (i) { … return i.substring(0,12) }              // 3. el SHA de git
  if (typeof t==="object" && t.source==="archive") { // 4. para archive:
    let l = t.sha256 ?? s;                           //    el digest, pinneado o descargado
    if (l) return l.toLowerCase().substring(0,12)
  }
  … return "unknown"
}
```

Dos cosas de aquí. Una: **`plugin.json` manda sobre la entrada del marketplace**, no al
revés — lo que hay que subir para que exista una versión nueva es el `version` de
*dentro del zip*. Dos: con `sha256` declarado un archive **nunca** cae en `"unknown"`,
así que el miedo de #23 a que el digest hiciera de identidad era en realidad la red de
seguridad; su razón para declarar `version` sigue en pie, pero es la que dijo —que la UI
pinta `manifest.version ?? "unknown"` en la pantalla donde el usuario decide si se fía—
y no la de la identidad.

Y la comprobación de si hay algo que hacer llega **después** de la descarga:

```js
let K = await PMr(F, { … });                      // <-- descarga y extrae el zip
…
let z = await IIe(s, p.source, K.manifest, K.path, p.version, …, K.contentSha256);
D = …
let q = mce(s,D), F = D==="unknown", B = mkt(s,D);
if (!F && (y.version===D || y.installPath===q || y.installPath===B))
  return { outcome:"up_to_date", message:`${n} is already at the latest version (${D}).` … }
```

`PMr` es la descarga completa —tiene que serlo, porque de ahí sale el manifiesto y el
`contentSha256`—. Sólo entonces se compara. De modo que:

- **Cada `/plugin update` de flipchart paga el binario entero**, unos 42 MB (~84 MB si
  es universal), aunque no haya nada nuevo. No es catastrófico a 25 MB/s, pero es un
  coste recurrente que el diseño no había contado, y es peor con red mala.
- **Cambiar sólo el digest no aterriza.** Si `plugin.json` sigue en `1.0.0`, la
  identidad no cambia: el host baja el zip nuevo, calcula que ya está *«at the latest
  version»* y lo tira. Un arreglo publicado sin subir la versión **no llega al usuario**
  y no avisa de nada. Es exactamente lo que el apartado 5 del ticket sospechaba, y hace
  de la disciplina de #23 —generar el `marketplace.json` desde el tag— no una comodidad
  sino un requisito de corrección.

## 7. Las cuatro formas en que el zip puede ser rechazado

**Leído del bundle**, con los mensajes literales. El orden importa: primero se filtra la
basura del Finder, después se promociona el envoltorio, y sólo al final se exige forma de
plugin.

```js
if (!(await J_a(s,l)).some(p => !p.startsWith("__MACOSX/") && Ta.basename(p) !== ".DS_Store"))
  throw new Bt(`Plugin archive from … contained no plugin files. The archive was not
    installed. Verify the URL serves a zip of the plugin contents.`, "plugin archive was empty")
let d = await aDn(l);
if (d !== l) w(`Plugin archive had a wrapper directory; using … as the plugin root`)
```

- **`__MACOSX/` y `.DS_Store` no estorban** — se ignoran al decidir si el zip tiene
  contenido. Un zip hecho desde el Finder no se rompe por eso. Pero **un zip que sólo
  los lleve** cuenta como vacío.
- **Un único directorio envoltorio se promociona solo** (`aDn`), y se dice en el log.
  Así que `flipchart-1.0.0/.claude-plugin/…` vale igual que `.claude-plugin/…`.
- Si el zip no tiene forma de plugin, el mensaje enumera qué se esperaba: *«has no plugin
  content at its root (expected `.claude-plugin/` or a `commands/`, `skills/`, `agents/`,
  `hooks/`, `themes/`, `output-styles/`, `monitors/`, `workflows/`, `SKILL.md`,
  `.mcp.json`, or `.lsp.json` at the top level, optionally inside a single wrapper
  directory)»*.
- Y hay un cuarto rechazo, para cuando la entrada declara rutas de componentes que el zip
  no trae: *«does not contain the component paths its marketplace entry declares … Repackage
  the zip so the declared paths sit at the plugin root»*.

Además, límites de bomba zip que nadie había anotado:

```js
HSt = { MAX_FILE_SIZE: 536870912, MAX_TOTAL_SIZE: 1073741824, MAX_FILE_COUNT: 1e5, MAX_COMPRESSION_RATIO: 50 }
```

512 MiB por fichero, 1 GiB descomprimido, 100 000 entradas y ratio 50. Ninguno aprieta
para flipchart —el tope de descarga de 256 MiB ata antes—, pero el ratio 50 es una nota
para cualquiera que piense en comprimir mejor: un binario nativo no llega ni cerca.

## 8. Los números, con el caudal real delante

**Ejecutado**, dos veces contra un asset de release público de 25,6 MB:

```
25568657 bytes en 0.996613s -> 25655552 B/s (redirects=1)
25568657 bytes en 0.934956s -> 27347444 B/s (redirects=1)
```

**25,7–27,9 MB/s, o sea 205–223 Mbit/s.** Con eso, la división:

| Magnitud | Valor |
|---|---|
| Tope del host | 256 MiB, 120 s, **sin válvula** |
| Caudal mínimo para llenar el tope en plazo | 2,24 MB/s ≈ **17,9 Mbit/s** |
| El zip de flipchart, arm64 | 42,05 MB → **2,8 Mbit/s** de mínimo (el mismo umbral que el clon de #22, porque es el mismo tamaño y el mismo plazo) |
| El mismo zip al caudal medido | **~1,6 s** — margen de ~75× |
| Universal binary (≈84 MB) | ~3,3 s — margen de ~36× |

El plazo no es el riesgo. Y hay una cosa que la medición trajo de propina:

**El asset de release redirige fuera de origen.** Un salto 302 de
`github.com/…/releases/download/…` a
`https://release-assets.githubusercontent.com/…` con firma en el query string;
`raw.githubusercontent.com`, en cambio, responde **200 sin ninguna redirección**. Los
dos hosts satisfacen `Bfo`, así que el camino elegido por #23 funciona tal cual. Pero el
salto cruza origen, y el código deja caer ahí las cabeceras heredadas del marketplace:

```js
if (r && Object.keys(r.headers).length>0)
  if (u$o(r.url, e.url)) i = r.headers;
  else w("Not forwarding marketplace headers to plugin archive on a different origin")
```

Consecuencia: **un asset de release privado o autenticado es imposible por esta vía** —
las cabeceras no sobreviven el redirect de GitHub. Para flipchart, que publica en abierto,
es gratis; queda escrito para que nadie lo intente.

## Qué le hace esto al mapa

**Confirma** el núcleo de [#23][23]: el catálogo por URL es un fichero JSON de 1,5 KB en
disco, **el cliente no ejecuta git ni una vez**, y la factura de [#22][22] —`B × (N + 2)`,
200 MB para un binario de 40 MB— **deja de existir**. Confirma también los 256 MiB y los
120 s sin válvula del archive, y que el plazo sobra por dos órdenes de magnitud.

**Corrige tres cosas:**

1. **El plazo del alta del catálogo son 10 s, no 120 s**, con un tope de 5 MiB. Los 120 s
   son del archive.
2. **El bit de ejecución sí se preserva**, si el zip lo hace un empaquetador Unix. El
   `chmod +x` del Lanzador pasa de ser el mecanismo a ser el respaldo — y el `zip` de la
   CI pasa a ser parte del contrato.
3. **`/plugin update` descarga el binario entero antes de comprobar si hay algo que
   actualizar**, y **si sólo cambia el digest, el zip nuevo se baja y se tira**. Lo
   segundo convierte *«generar el `marketplace.json` desde el tag»* de comodidad en
   requisito, y añade una regla nueva: **la versión que manda es la de `plugin.json`,
   dentro del zip**, no la de la entrada del catálogo.

**Y deja abierto lo que era la condición de viabilidad** — abierto durante un día. La
cuarentena no puede aparecer por construcción, y eso es un argumento sólido; pero el
ticket pedía verlo, y con un binario firmado ad-hoc y sin notarizar el único juez
definitivo es ejecutarlo. Eso, más el rechazo por digest cambiado, el modo real del
fichero extraído, las cuatro formas de zip y el `update` de verdad, es lo que
**[research 15][r15] corrió el 2026-09-03** contra un repo público desechable: las cinco
salieron como este documento predijo, y encima trajeron tres cosas que la lectura no
podía ver —`com.apple.provenance`, un `spctl -a` que dice `rejected` sobre un binario
que corre igual, y una caché que se queda con las dos versiones tras el `update`.

[22]: https://github.com/javierponferradalopez/ai-render/issues/22
[23]: https://github.com/javierponferradalopez/ai-render/issues/23
[r10]: ./10-el-binario-por-el-clon-del-marketplace.md
[r15]: ./15-la-cuarentena-medida-y-el-veredicto-del-vehiculo.md
