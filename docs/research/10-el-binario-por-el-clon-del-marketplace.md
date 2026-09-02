# El binario por el clon del marketplace

Investigación del 2026-09-02 para ejecutar lo que [Cómo llega el binario sin morir
en el arranque][21] decidió leyendo: que el binario de flipchart puede viajar
**comprometido dentro del repo del plugin** porque Claude Code clona superficial y
"la historia no viaja".

**El mecanismo aguanta, pero por una razón distinta de la que se creyó, y con una
factura que nadie había contado.** Lo que llega intacto es la instalación desde
cero; lo que no sale gratis es la actualización.

## Nota sobre el método

Todo lo que sigue está **ejecutado** en macOS 26.6.2 arm64 con Claude Code
**2.1.228** y git 2.50.1, salvo lo que se marca como *leído del bundle*. El banco
de pruebas es el [prototipo 16](./prototipos/16-marketplace-con-blob/): un
marketplace propio cuyo plugin lleva un blob de 40 MB de `/dev/urandom` —
incompresible, que es el peor caso—, instalado de verdad contra un
`CLAUDE_CONFIG_DIR` aislado.

No hizo falta publicar nada en GitHub: un marketplace de fuente `git` sobre un
bare repo local por `file://` recorre el mismo camino de código. Lo único que ese
atajo no mide es el tiempo de red, y el tiempo de red es una división.

## 1. El clon **no** es blobless: eso era condicional, y la condición no se da

[#21][21] leyó junto a `Cloning via SSH` las banderas `--depth`,
`--filter=blob:none`, `--no-checkout` y `sparse-checkout set`, y dedujo un clon
parcial. El código dice otra cosa (*leído del bundle*):

```js
let a = ["-c", "core.sshCommand=ssh -o BatchMode=yes -o StrictHostKeyChecking=yes",
         "clone", "--depth", "1"]
if (sparsePaths?.length) a.push("--filter=blob:none", "--no-checkout")
else                     a.push("--recurse-submodules", "--shallow-submodules")
```

`--depth 1` es incondicional; **el clon parcial sólo aparece si el marketplace
declara `sparsePaths`**, un campo opcional de las fuentes `github` y `git` cuya
propia descripción lo cierra: *"Directories to include via git sparse-checkout
(cone mode). Use for monorepos where the marketplace lives in a subdirectory. **If
omitted, the full repository is cloned**."*

Verificado en el clon en disco: `rev-list --count HEAD` = **1**, sin
`remote.origin.partialclonefilter` y sin `core.sparseCheckout`.

Esto es **mejor** de lo que #21 suponía, no peor. El miedo del ticket —que un clon
blobless dejara el binario fuera y lo bajara perezosamente en el primer acceso,
que es el arranque del servidor MCP, justo donde no queremos red— **no puede
ocurrir**: sin clon parcial no hay promisor al que ir. El riesgo queda eliminado
por construcción.

Con una condición para el futuro: si algún día flipchart declarara `sparsePaths`
—por ejemplo si el marketplace pasara a vivir en un monorepo—, el binario tendría
que estar **dentro** de las rutas listadas, o desaparecería del checkout.

## 2. El blob llega entero, ejecutable y sin cuarentena — y la copia también

Medido a los dos lados del camino, con el mismo sha256 de origen a destino:

| | Modo | sha256 | `xattr` |
|---|---|---|---|
| Clon del marketplace | `-rwxr-xr-x` | idéntico | sólo `com.apple.provenance` |
| Copia instalada en `cache/<market>/<plugin>/<versión>/` | `-rwxr-xr-x` | idéntico | sólo `com.apple.provenance` |

- **El bit de ejecución sobrevive** al clon y a la copia: git conserva el `100755`
  y la copia del host lo respeta.
- **No aparece `com.apple.quarantine`.** Confirma lo que [La experiencia de
  instalación][14] supuso: Gatekeeper dispara por ese atributo extendido, que lo
  ponen los descargadores de LaunchServices, y ni git ni la copia lo aplican. Lo
  que sí aparece es `com.apple.provenance`, que es otra cosa y no bloquea nada.
- **La copia es copia, no enlace**: inodos distintos. Esto importa para la cuenta
  de la sección 4.

## 3. El plazo son 120 s, el usuario puede ampliarlo, y el fallo se ve

`y5b = 120000` es el timeout por defecto, confirmado en el log de una sesión real
(`git clone: url=… timeout=120000ms`). Y existe una válvula que el mapa no
conocía: **`CLAUDE_CODE_PLUGIN_GIT_TIMEOUT_MS`**, que gobierna tanto el clon como
el pull.

Forzándola a 50 ms, esto es lo que ve el usuario:

```
✘ Failed to update marketplace(s): Failed to refresh marketplace 'fatblob-market':
  Failed to clone marketplace repository: Git clone timed out after 0s. The
  repository may be too large for the current timeout. Set
  CLAUDE_CODE_PLUGIN_GIT_TIMEOUT_MS to increase it (e.g., 300000 for 5 minutes).

Original error: Cloning into '…'
fatal: early EOF
```

Es lo contrario del fallo mudo que envenena el arranque de un servidor MCP: un
error en primer plano, con la causa nombrada y la solución dentro del mensaje, y
el usuario puede reintentar. Confirma la lectura de #21 sobre dónde conviene que
viva la espera.

**El umbral de ancho de banda es una división**: 40 MB en 120 s son ~2,8 Mbit/s
efectivos sostenidos. Por debajo de eso, la instalación falla — con mensaje — y se
arregla con una variable de entorno.

## 4. El `update` acumula historia: el `.git` crece un binario por release

Aquí es donde el diseño de #21 se rompe. El pull del update es (*leído del
bundle*) `git pull origin HEAD` — **sin `--depth`**. Medido:

| | `.git` del marketplace | commits |
|---|---|---|
| Tras el clon inicial | 40 MB | 1 |
| Tras un `marketplace update` a v0.0.2 | **80 MB** | **2** |

El clon nace superficial y **deja de serlo en la primera actualización**. Cada
release añade un binario entero al `.git` local, y nada lo poda nunca.

Esto rehabilita la objeción que #21 declaró falsa —*"un giga a las veinte
releases"*—, con un matiz que cambia a quién le toca pagar:

- **Quien instala de cero nunca paga la historia.** El clon es `--depth 1` y baja
  un solo binario. La lectura de #21 era correcta *para ese usuario*.
- **Quien actualiza la acumula, release a release.** El usuario fiel, el que se
  queda, es el que paga.

## 5. La factura en disco: 3× el binario, y 200 MB tras una actualización

Medido sobre un binario de 40 MB, con una sola actualización:

```
cfg/plugins/                                        200 MB
├── marketplaces/fatblob-market/                     120 MB
│   ├── .git/                                         80 MB   (v1 + v2, y creciendo)
│   └── plugins/fat/bin/blob                          40 MB   (working tree)
└── cache/fatblob-market/fat/                          80 MB
    ├── 0.0.1/bin/blob                                40 MB   (huérfana)
    └── 0.0.2/bin/blob                                40 MB   (la que se ejecuta)
```

**Un binario de 40 MB ocupa 120 MB nada más instalarlo** —el working tree del clon,
el objeto en `.git`, y la copia de la caché— y 200 MB tras el primer update.

De esos, la parte recuperable se recupera sola: la versión vieja de la caché se
marca con `.orphaned_at` y el recolector la borra a los **14 días**
(`r5b = 1209600000` ms, *leído del bundle*), siempre que ninguna sesión viva la
tenga tomada por su marcador `.in_use/<pid>` — mecanismo que se verificó existiendo
(`.in_use/74463` sobre la versión en uso).

Lo que **no** se recupera nunca es el `.git` del marketplace. Ahí la cuenta crece
monótona: con un binario de tamaño B y N actualizaciones, el suelo permanente es
`B × (N + 2)`.

## 6. SSH y HTTPS: hay ida y vuelta automática, y esto corrige a research 09

[research 09 §6](./09-la-mecanica-de-plugins-verificada.md) midió que
`marketplace add owner/repo` clona por SSH y concluyó que un repo privado exige,
por tanto, SSH configurado. El código completo es más generoso (*leído del
bundle*): hay **fallback bidireccional**.

- Primero se comprueba si SSH está configurado, ejecutando
  `ssh -T -o BatchMode=yes -o ConnectTimeout=2 git@github.com` con 3 s de límite.
- **Si lo está:** clona por SSH y, si falla, *"SSH clone failed, retrying with
  HTTPS"*.
- **Si no lo está:** *"SSH not configured, cloning via HTTPS"*, y si el HTTPS
  falla reintenta por SSH.

Así que el camino HTTPS existe, es automático y no hay que documentarle nada al
usuario. La observación de research 09 sigue siendo cierta —SSH va primero cuando
está configurado— pero no es una exigencia.

## 7. Git LFS no está descartado por el host, pero tampoco ayuda

El clon exporta `GIT_LFS_SKIP_SMUDGE=1` **sólo** si el marketplace declara
`skipLfs: true` (*"so LFS pointer files stay as pointers instead of downloading
their content"*, un opt-in para marketplaces alojados en repos con objetos LFS
grandes). No es forzado.

Dicho eso, LFS no resuelve nada de la sección 4: mueve el bulto del `.git` al
almacén LFS, sigue exigiendo `git-lfs` instalado en la máquina del usuario, y su
descarga entra en el mismo plazo.

## 8. Dos vehículos que el mapa no conocía

Buscando lo anterior aparecieron dos mecanismos de entrega de binarios que ningún
ticket había visto. Los dos son *leídos del bundle*, no ejecutados.

### 8.1 `experimental.binaries` — hace exactamente lo que flipchart quiere, y está cerrado

El manifiesto de plugin admite un campo experimental `binaries`: *"sha256-pinned
files to fetch into bin/ at install time, keyed by basename (target triple encoded
in the name)"*. Es un aprovisionamiento content-addressed en tiempo de
instalación, con verificación de digest, borrado del fichero si no casa,
colocación bajo `bin/` con modo `0755`, caché compartida entre plugins y hasta un
alias sin sufijo derivado del target triple de la máquina. Incluso hay una
validación que comprueba que el `command` de tu `.mcp.json` apunte a un `bin/<x>`
que sea *"a shipped file, a declared binaries entry, or a name derivable from the
declared entries — the server will fail to start"*. Es, literalmente, el diseño de
flipchart implementado por el host.

**Y no está disponible.** Dos candados:

```js
if (!K5b()) return                        // gate: CLAUDE_CODE_PLUGIN_BINARY_ASSETS
if (!b0(Ss(t).marketplace)) return        // sólo marketplaces oficiales
```

`b0` consulta un `Set` de nombres reservados a Anthropic —`claude-plugins-official`,
`claude-code-plugins`, `agent-skills`, `anthropic-marketplace`…—, los mismos que
un marketplace propio tiene prohibido usar. Un marketplace de terceros nunca
entra. Explica de paso por qué los doce plugins `*-lsp` del catálogo oficial
podían permitirse decir *"instálatelo tú"*: tienen una vía que nadie más tiene.

Conclusión: **no es una opción para flipchart**, y conviene anotarlo antes de que
alguien lo redescubra y crea que sí.

### 8.2 `source: "archive"` — un zip por HTTPS, y este sí está abierto

Una entrada de plugin de un marketplace puede declararse así:

```json
{"name": "flipchart", "source": "archive",
 "url": "https://…/flipchart-0.1.0.zip",
 "sha256": "…64 hex…"}
```

*"Plugin distributed as a zip archive fetched over HTTPS — for hosting on any
static file server or artifact repository (S3, GitLab, nginx) with no git or npm
on the client."* El `sha256` es opcional pero **recomendado**: *"every download is
verified against it and the install is refused on mismatch"*.

Los números, del código: hasta **256 MB** por archivo (`maxContentLength`), **120 s
de timeout** —y este **no** lo amplía ninguna variable de entorno, a diferencia del
clon—, hasta 5 redirecciones, cada salto revalidado contra una política que exige
`https://` y prohíbe loopback, link-local y hosts de metadatos de nube.

Lo que compra frente al binario comprometido en el repo: el marketplace vuelve a
ser un repo diminuto —sólo el `marketplace.json`—, el binario vive en un release y
**no entra jamás en ninguna historia de git**, así que la sección 4 desaparece y
con ella los 200 MB de la 5. Y la integridad pasa a estar verificada, que hoy no
lo está.

Lo que cuesta: hay que alojar el zip en algún sitio con HTTPS, la actualización se
señaliza por el `version` del manifiesto (*"changing only the digest while a
version is declared does not trigger an update"*), y los 120 s dejan de tener
válvula.

**No está medido.** Merece su propio ticket.

## 9. Lo que no se ha medido

- **El clon real contra GitHub**, por SSH y por HTTPS, con un binario de decenas de
  MB. Aquí se ejercitó el mismo camino de código sobre `file://`, así que lo único
  sin medir es el tiempo de red — que es la división de la sección 3.
- **`source: "archive"` corriendo.** Todo lo de §8.2 está leído.
- **El tamaño real del binario de flipchart**, que sigue dependiendo de [El stack de
  rendering][8] y estaba fuera de alcance por decisión del propio ticket.

[8]: https://github.com/javierponferradalopez/ai-render/issues/8
[14]: https://github.com/javierponferradalopez/ai-render/issues/14
[21]: https://github.com/javierponferradalopez/ai-render/issues/21
