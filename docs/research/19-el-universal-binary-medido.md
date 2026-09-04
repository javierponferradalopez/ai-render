# El universal binary, medido — y la firma que sólo cubría media caja

Medición del 2026-09-04 pedida por [La caja, el zip y la CI del release][44], que es el
punto 4 —el último— de la checklist del primer día de `DECISIONS.md` §11.3: tener el
número real del universal binary **antes** de escribir el `marketplace.json`.

**Cabe de sobra, y por dos órdenes de magnitud: el universal firmado son 49,2 MB y el zip
que se publica 18,7 MB, contra un tope de archive de 256 MiB.** Y con el número delante se
publicó `v0.1.0`, que instala y arranca (§5). El margen es de **14×** en
tamaño y de **~165×** en plazo con el caudal ya medido del CDN. La referencia del ticket
—6,9 MB de mmdr suelto— se multiplica por **3,4** al enlazar `eframe` + `winit` + `resvg`
+ `rmcp` en una arquitectura, y por **7,1** en dos.

Lo que la medición **no** venía a buscar y es lo que de verdad importa: **la firma ad-hoc
del universal sólo cubría la mitad arm64**, y el único mandado que se da cuenta es
`codesign --verify`. `codesign -dv` sobre ese binario a medio firmar contesta
`Signature=adhoc` y el binario **corre** en el Mac que lo fabricó. El defecto es invisible
en la máquina que compila y sólo muerde en los Mac Intel.

## Nota sobre el método

macOS **26.6.2** (build 25G83) arm64, `rustc` 1.98.0 (el de `rust-toolchain.toml`), perfil
`release`, Info-ZIP `zip` 3.0 de Apple. Las dos mitades salen de
`cargo build --release --target {aarch64,x86_64}-apple-darwin`, con el juego real de
dependencias del §1 enlazado — que era la condición que ponía [#35][35] para que este
número significara algo.

El empaquetado es el que va a correr la CI: `publicacion/empaqueta.sh` y
`publicacion/catalogo.sh`, con el binario firmado delante.

## 1. Los números

| Pieza | Bytes | |
|---|---|---|
| `flipchart` thin arm64, tal como sale de `cargo` | 23 710 832 | 23,7 MB |
| `flipchart` thin x86_64, cruzado | 25 379 976 | 25,4 MB |
| universal tras el `lipo` | 49 106 032 | 49,1 MB |
| **universal tras la re-firma ad-hoc** | **49 215 680** | **49,2 MB** |
| **el zip de la caja entera** | **18 684 362** | **18,7 MB** |
| el `marketplace.json` generado | 701 | |

El zip comprime al **62,0 %**: un Mach-O con tres motores dentro tiene mucho que apretar,
y eso es lo que hace que el vehículo pese la mitad de lo que ocupa instalado.

## 2. Los márgenes, contra los topes que no tienen válvula

| Tope | Medido | Margen |
|---|---|---|
| Archive: **256 MiB**, sin válvula | 18,7 MB | **14,4×** |
| Archive: **120 s**, sin válvula | ~0,7 s a los 25,7–27,9 MB/s medidos del CDN | **~165×** |
| Catálogo: **5 MiB** | 701 bytes | 7 500× |
| Catálogo: **10 s** | un fichero plano de 701 bytes | — |

**Y el pico en disco baja:** §10.1 lo estimó sobre un universal de ~84 MB y anotó ~168 MB
entre la actualización y la poda. Con los 49,2 MB reales, y con el zip que nunca toca el
disco porque viaja como `arraybuffer`, el pico son **~98 MB**.

## 3. La firma: `lipo` no la rompe, es que nunca hubo dos

El §10.3 escribió el paso 3 como *«re-firmar y verificar que la firma sobrevive al
`lipo`»*. Medido, el mecanismo es otro y el remedio es el mismo.

**Rust firma ad-hoc sólo la mitad nativa.** La `x86_64` cruzada en un host arm64 sale de
`cargo` sin firmar, y se ve antes de tocar el `lipo`:

```
$ codesign -dv target/x86_64-apple-darwin/release/flipchart
target/x86_64-apple-darwin/release/flipchart: code object is not signed at all
```

**El `lipo` conserva la asimetría**, y ahí empieza la trampa: sobre el universal a medio
firmar, los dos mandados con los que uno mira una firma dicen cosas opuestas.

| Sobre el universal sin re-firmar | Qué contesta |
|---|---|
| `codesign -dv` | `Format=Mach-O universal (x86_64 arm64)`, **`Signature=adhoc`** |
| `codesign -dv --arch arm64` | `Signature=adhoc` |
| `codesign -dv --arch x86_64` | **`code object is not signed at all`** |
| `codesign --verify` | **`code object is not signed at all`** |
| ejecutarlo en este Mac (arm64) | **`rc=0`** |

`codesign -dv` a secas lee **la rebanada nativa** y da por firmado el fichero entero. Y el
binario corre, porque la rebanada que este Mac ejecuta sí está firmada. Un release
publicado así habría funcionado en todas las máquinas donde se prueba y habría estado roto
en las Intel.

Tras `codesign -s - -f`, las dos rebanadas quedan `adhoc` y `--verify` pasa:

```
flipchart: valid on disk
flipchart: satisfies its Designated Requirement
```

De ahí lo que la CI comprueba, que es lo contrario de lo intuitivo: **`--verify`, y `-dv`
una vez por arquitectura**. `-dv` sobre el fichero no vale como verificación.

## 4. La caja, empaquetada de verdad

`unzip -Z` sobre el zip que produce `publicacion/empaqueta.sh`:

```
-rwxr-xr-x  3.0 unx     4545 t- defN launcher.sh
-rw-r--r--  3.0 unx      104 t- defN .mcp.json
-rwxr-xr-x  3.0 unx 49215680 b- defN flipchart
drwxr-xr-x  3.0 unx        0 b- stor .claude-plugin/
-rw-r--r--  3.0 unx      190 t- defN .claude-plugin/plugin.json
```

**`3.0 unx`** es el `version made by == 3` que el §10.2 llama parte del contrato, y
**`-rwxr-xr-x`** sobre el binario y sobre el Lanzador es de dónde sale el `100755` que
research 15 midió en la máquina del usuario. Los cuatro ficheros y nada más.

## 5. El release de verdad, instalado

**Ejecutado el mismo día**, con `v0.1.0` publicado por la CI —los doce pasos en verde— y
verificado desde el cliente contra un `CLAUDE_CONFIG_DIR` aislado. Banco: Claude Code
2.1.228, macOS 26.6.2 arm64.

| Medida | Valor |
|---|---|
| El catálogo por `raw.githubusercontent.com` sobre `main` | 701 bytes, servido |
| El asset, y su digest careado contra el declarado | 18 683 644 bytes, **casan** |
| `plugin marketplace add` + ficheros `.git` en el cliente | **NINGUNO** |
| `plugin install flipchart@flipchart` | **✔ 5,5 s** |
| El binario extraído | `plugins/cache/flipchart/flipchart/0.1.0/flipchart` |
| Modo, y bytes | **`100755`**, 49 215 680 — el universal entero |
| `xattr -l` / cuarentena | `com.apple.provenance` y nada más / **NINGUNA** |
| Firma y arquitecturas | `Mach-O universal (x86_64 arm64)`, `Signature=adhoc` |
| La tubería entera desde el binario extraído | `flipchart check` → `drawn`, `2 nodes, 1 edge`, `rc=0` |

**Y lo que research 15 dejó explícitamente sin medir: el Servidor MCP de verdad, hablando
desde la caja extraída.** Su sonda era bash pelado y no completaba el handshake nunca. El
binario sí:

```json
{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},
 "serverInfo":{"name":"flipchart","version":"0.1.0"}}
```

`tools/list` devuelve **`show` y `clear`** —no el `unavailable` del Servidor de aviso—, así
que el Lanzador le cedió el sitio y su stderr salió vacío.

**Arrancado por el host, y aquí está el número que importa:**

```
MCP server "plugin:flipchart:flipchart": Successfully connected (transport: stdio) in 31ms
Connection established with capabilities: {"hasTools":true,…,"serverVersion":{"name":"flipchart","version":"0.1.0"}}
```

**31 ms contra el plazo de 30 000 ms** del §10.4 — margen de ~970×. Es el plazo cuyo
incumplimiento veta el servidor 15 minutos, y el que justifica que el Lanzador exista.

De propina, la cuenta del disco: **47 MB** el `CLAUDE_CONFIG_DIR` con una versión dentro,
que confirma por lo alto el pico de ~98 MB estimado en §2 para las dos.

**Un aviso para quien caree los números:** el zip que publicó la CI son 18 683 644 bytes y
el del §1 —empaquetado en local— 18 684 362. El zip **no es reproducible byte a byte**,
porque Info-ZIP guarda las fechas de modificación de lo que empaqueta. No importa: el
`sha256` lo calcula `catalogo.sh` sobre el zip exacto que se publica, en el mismo job.

## Lo que no se ha medido

- ~~**El umbral de macOS** (§10.7)~~. **Sí lo dice el build, y se midió el mismo día**:
  `minos 11.0` en la rebanada `arm64` y `10.12` en la `x86_64`, sin ninguna API weak-linked
  que pudiera subirlo por detrás
  ([research 20](./20-la-instalacion-y-la-linea-verificadas.md) §2). Lo que sigue sin medirse
  es **correrlo** en algo anterior al 26.6.2.
- **La mitad x86_64 corriendo.** Está firmada y es un Mach-O válido, pero aquí no hay Mac
  Intel: lo que se ha comprobado es la firma de su rebanada, no su ejecución.
- **El caudal**, que se hereda de research 11 (25,7–27,9 MB/s desde el CDN de GitHub) y no
  se ha vuelto a medir.

[44]: https://github.com/javierponferradalopez/ai-render/issues/44
[35]: https://github.com/javierponferradalopez/ai-render/issues/35
