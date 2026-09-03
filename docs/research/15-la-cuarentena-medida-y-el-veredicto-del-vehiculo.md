# La cuarentena, medida — y el veredicto del vehículo

Medición del 2026-09-03 pedida por [La cuarentena del zip, medida — decide el vehículo
de instalación][46], que es el riesgo 1 de `DECISIONS.md` §11.2 y el punto 1 de su
checklist del primer día: lo único que puede rehacer el vehículo de instalación entero
en vez de una línea de código.

**No hay cuarentena. El vehículo del §10.1 se queda como está, y el plan B no se
compra.** El binario extraído de un zip alojado de verdad por HTTPS llega **sin
`com.apple.quarantine`**, con modo **`100755`**, y **corre**: `rc=0`, tanto lanzado a
mano como lanzado por el host a través del Lanzador, y **antes** del `chmod +x`.

Lo que [research 11][r11] dejó como lectura —la cuarentena, el bit de ejecución, las
cuatro formas de rechazo del zip y el `update` que se descarga el binario para tirarlo—
está ahora **ejecutado**, y confirma las cinco. Lo que la medición añade encima son
**tres cosas que la lectura no podía ver**: un atributo extendido que sí aparece y no es
el que mata, un `spctl` que dice `rejected` sobre un binario que corre igual, y una
caché que se queda con las dos versiones tras el `update`.

## Nota sobre el método

macOS **26.5.2** (build 25F84) arm64, Claude Code **2.1.259** —no el 2.1.228 sobre el
que leyó research 11—, clang 21.0.0, python3 3.9.6 de las Command Line Tools. El banco
es el [prototipo 17][p17]; lo reproduce entero `run-hosted.sh`, contra un
`CLAUDE_CONFIG_DIR` aislado, y la corrida completa está guardada tal cual en su
[`registro-hosted.log`](./prototipos/17-el-vehiculo-del-zip/registro-hosted.log).

**No hay atajo local y por eso esto tardó en correrse:** la política de URL del archive
exige `https://` y prohíbe loopback, link-local y hosts de metadatos de nube. Hubo que
alojar de verdad, en un repo público desechable
(`javierponferradalopez/flipchart-archive-probe`), con el catálogo por
`raw.githubusercontent.com` y los zips como assets de dos releases.

Antes de medir se releyó el bundle de 2.1.259, porque research 11 leyó el de 2.1.228 y
la medición corre sobre el nuevo. Los tres mecanismos citados siguen intactos: la
descarga del archive sigue siendo `responseType:"arraybuffer"` con
`maxContentLength:vJ` (`vJ=268435456`), siguen los `5242880` del catálogo y los
`120000` del plazo, y la extracción sigue haciendo `if(U&&U&73) chmod(U&511)`. Y
`com.apple.quarantine` ya no aparece **ni una vez** en los 200 MB del ejecutable — en
2.1.228 aparecía una, dentro de un documento de skill ajeno.

El material es el mismo de research 11: un Mach-O de 42 MB con 40 MB de `/dev/urandom`
en `__DATA,__blob`, **firmado ad-hoc y sin notarizar**, que es el caso que Gatekeeper
mataría. Un `.sh` no habría medido nada.

## 1. La cuarentena: no aparece, y el binario corre

**Ejecutado.** `claude plugin install flipchart@flipchart-probe`, 17,3 s de punta a
punta para 42 MB descargados y extraídos:

| Medida sobre el fichero extraído | Valor |
|---|---|
| Ruta | `plugins/cache/flipchart-probe/flipchart/1.0.0/bin/flipchart` |
| Modo | **`-rwxr-xr-x` (`100755`)** |
| Bytes | 42 076 400 — el binario entero, intacto |
| `xattr -l` | `com.apple.provenance` **y nada más** |
| `com.apple.quarantine` | **NINGUNA** |
| Ejecutarlo | `flipchart-probe: alive, blob[0]=182`, **`rc=0`** |
| `codesign -dv` | `Format=Mach-O thin (arm64)`, `Signature=adhoc` |

Y no sólo sobre la extracción: **sobre todo lo que la caché versionada guarda**, que era
la otra mitad de lo que el ticket pedía mirar. Ninguno lleva cuarentena — ni el binario
de `1.0.0`, ni el de `2.0.0` tras el update, ni el del plugin instalado sin `sha256`, ni
el JSON del catálogo, ni los directorios:

```
-rwxr-xr-x  xattr=[com.apple.provenance ]  /plugins/cache/flipchart-probe/flipchart/1.0.0/bin/flipchart
-rwxr-xr-x  xattr=[com.apple.provenance ]  /plugins/cache/flipchart-probe/flipchart/2.0.0/bin/flipchart
-rwxr-xr-x  xattr=[com.apple.provenance ]  /plugins/cache/flipchart-probe/sin-declarar/1.0.0/bin/flipchart
-rw-r--r--  xattr=[com.apple.provenance ]  /plugins/marketplaces/flipchart-probe
```

### Lo arranca el host, no nosotros

Ejecutarlo desde el shell del banco no basta: el juez es el host. El Lanzador-sonda va
dentro del zip y anota lo que ve **antes** de tocar nada; lo arranca una sesión abierta
con `hold.py` del [prototipo 15][p15], que mantiene `claude` vivo sin llamar al API.

```json
{
  "mode_before_chmod": "-rwxr-xr-x 100755",
  "exec_bit_before_chmod": "yes",
  "xattrs": "com.apple.provenance,",
  "quarantine": "NONE",
  "run_before_chmod_rc": 0,
  "run_before_chmod_out": "flipchart-probe: alive, blob[0]=182",
  "mode_after_chmod": "-rwxr-xr-x 100755",
  "run_after_chmod_rc": 0
}
```

**El binario ya corría antes del `chmod +x`.** Y el modo no cambia después, porque no
había nada que cambiar.

### El atributo que sí aparece, y el `spctl` que asusta y no muerde

Dos cosas que sólo se ven corriéndolo, y que hay que dejar escritas juntas porque leídas
por separado dicen lo contrario de lo que pasa:

**`com.apple.provenance` está puesto**, sobre el binario y sobre todo lo demás, y está
**antes** de que nadie lo ejecute. Es el atributo que macOS pone desde Ventura para
rastrear procedencia; **no lo mira Gatekeeper** y no impide nada. El fichero corre con
él encima. Que el `xattr` del binario no salga vacío no es la señal de alarma que
parecía: la señal era `com.apple.quarantine`, y no está.

**`spctl -a -vv` responde `rejected`.** Y así responderá siempre sobre un ad-hoc sin
notarizar: es la evaluación de política de Gatekeeper, que es exactamente lo que un
binario sin notarizar no pasa. **Lo que importa es que la ejecución no pasa por ahí.**
Sin `com.apple.quarantine`, `execve` no consulta a Gatekeeper, y el `rc=0` del apartado
anterior es la prueba. Escribirlo importa porque un `spctl` en rojo dentro de un log es
justo el tipo de dato que, releído dentro de seis meses, revierte una decisión buena.

## 2. El catálogo por URL: cero git, ahora también por HTTPS real

**Ejecutado.** Research 11 midió esto en loopback por `http://`. Repetido contra
`https://raw.githubusercontent.com/…/main/marketplace.json`, que es la URL que va a
teclear el usuario:

| Medida | Valor |
|---|---|
| Ficheros `.git*` en todo el `CLAUDE_CONFIG_DIR` | **NINGUNO** |
| El catálogo en disco | `plugins/marketplaces/flipchart-probe`, fichero plano, `-rw-r--r--`, 2717 bytes |

La factura de `B × (N + 2)` del clon no se mitiga: no existe.

## 3. El digest, con el rechazo delante

**Ejecutado**, con el `sha256` cambiado a mano a 64 bes sobre un zip por lo demás bueno:

```
✘ Failed to install plugin "digest-malo@flipchart-probe": Plugin archive integrity check
  failed for https://…/flipchart-1.0.0.zip: expected sha256 bbbb…bbbb, got
  6501c7b19f40f86ad3f2592dcce71bc2ba385ba6529de6abab0b32424ab8714b. The archive was not
  installed. Verify the sha256 in the marketplace entry, or that the URL serves the
  intended file.
```

**Rechaza, es de primer plano, y el mensaje trae el esperado y el obtenido.** Es la
integridad verificada que §10.1 da por buena, ahora medida.

Y la contracara, que research 11 no pudo llegar a ver porque su entrada sin digest
moría antes en la red: **`sin-declarar`, sin `sha256` en la entrada, se instala tan
ricamente** — `✔ Successfully installed`, con su binario de 42 MB en la caché. El
`sha256` es opcional de verdad, y sin él **no hay verificación de integridad ninguna**.
Que flipchart lo declare es disciplina propia; olvidarlo en el generador del
`marketplace.json` no rompe nada visible y desarma la única defensa del vehículo.

## 4. Las cuatro formas del zip, ejecutadas

**Ejecutado.** Las cuatro se comportan como el bundle prometía:

| Zip | Resultado |
|---|---|
| Todo dentro de un único directorio envoltorio | **instala** — el envoltorio se promociona solo |
| Con `__MACOSX/` y `.DS_Store` junto al contenido | **instala** — la basura del Finder no estorba |
| Sólo `__MACOSX/` y `.DS_Store` | rechaza: *«contained no plugin files»* |
| Contenido sin forma de plugin | rechaza: *«has no plugin content at its root (expected `.claude-plugin/` or a `commands/`, `skills/`, …)»* |

## 5. El `update`: el hallazgo caro, ya no leído sino medido

**Ejecutado**, en los tres casos que separan el digest de la versión.

**(a) Misma versión, mismo digest.** `✔ flipchart is already at the latest version
(1.0.0)`. Nada que hacer, como debe ser.

**(b) Sólo cambia el digest.** Un segundo zip, con contenido distinto —lleva un
`PARCHE.md` dentro que hace de trazador— y **el mismo `1.0.0` en el `plugin.json` de
dentro**; reescritos `url` y `sha256` de la entrada, sin tocar `version`:

```
Checking for updates for plugin "flipchart" at user scope…
✔ flipchart is already at the latest version (1.0.0).

real	0m4.143s
```

**Los 4,1 s son la prueba: se bajó los 42 MB enteros antes de decidir que no había nada
que hacer.** Y el `PARCHE.md` **no aparece en ninguna parte del disco**. Un arreglo
publicado sin subir la versión se descarga, se tira y no avisa. La regla de §10.2
—generar el `marketplace.json` desde el tag— queda confirmada como **requisito de
corrección**, no comodidad.

**(c) Bump completo.** `version` a `2.0.0` dentro del `plugin.json` del zip, segundo
release, y `version` + `url` + `sha256` en la entrada:

```
✔ Plugin "flipchart" updated from 1.0.0 to 2.0.0 for scope user. Restart to apply changes.
```

**Y lo que la lectura no anticipaba: la caché se queda con las dos.**

```
plugins/cache/flipchart-probe/flipchart/1.0.0
plugins/cache/flipchart-probe/flipchart/2.0.0
```

Los dos binarios de 42 MB, enteros, con el `CLAUDE_CONFIG_DIR` en **120 M** al final de
la corrida. No es la factura del clon —aquí no hay `.git` que crezca sin podarse— y
§10.1 ya anota la recogida por `.orphaned_at` a los 14 días, pero **el pico entre una
actualización y la poda son dos binarios en disco, no uno**. Con un universal de ~84 MB
eso son ~168 MB de pico. Conviene que esté escrito antes de que alguien lo descubra
mirando su disco.

## El veredicto

**El vehículo de §10.1 sigue en pie, entero, y ninguna de las dos salidas del plan B se
compra.**

- **No se notariza.** Los 99 $/año compraban una defensa contra un ataque que no ocurre:
  el binario extraído no lleva cuarentena y corre.
- **No se vuelve al binario dentro del repo del marketplace clonado.** Su factura de
  `B × (N + 2)` permanente sigue siendo real y sigue siendo peor.

Lo que la medición **cambia** del mapa es pequeño y va todo en la misma dirección
—escribir lo que no se sabía—, no en revertir nada:

1. **`spctl` dice `rejected` y el binario corre igual.** Escribirlo para que ese
   `rejected` no revierta la decisión dentro de seis meses.
2. **`com.apple.provenance` aparece siempre.** No es la cuarentena y no impide nada.
3. **Sin `sha256` en la entrada, la instalación va igual y sin verificar.** El generador
   del `marketplace.json` tiene que declararlo o la integridad del vehículo desaparece
   en silencio.
4. **El `update` deja dos versiones en la caché hasta que la poda pase.** Pico de
   `2 × B`, no `B`.

Y una línea de §10.3 se refuerza sola: **nunca documentar «bájate el zip a mano»**. Lo
medido dice que el camino del host no cuarentena; el del navegador sí, y ése es el que
mata.

## Lo que no se ha medido

- **El universal binary.** Todo esto es `arm64` thin. El `lipo` + re-firma ad-hoc de
  §10.3 y el tamaño real siguen siendo el punto 3 de la checklist del primer día.
- **El servidor MCP del zip, funcionando.** El Lanzador-sonda arrancó, ejecutó el
  binario y escribió su `probe.json`, pero su servidor de aviso en bash pelado no
  completa el handshake y la sesión corta con `CONNECT_TIMEOUT` a los 30 s. Eso es del
  arnés, no del vehículo: lo que el ticket pedía —que el host arranque lo que salió del
  zip— ocurrió y está registrado. El servidor de verdad es de §10.5.
- **Otro macOS que no sea 26.5.2 arm64**, y cualquier Windows o Linux, que están fuera
  del MVP.
- **Un zip empaquetado por algo que no sea Info-ZIP.** El `100755` medido depende de que
  el empaquetador escriba atributos Unix; §10.2 ya lo llama parte del contrato de la CI.

[46]: https://github.com/javierponferradalopez/ai-render/issues/46
[r11]: ./11-el-zip-verificado-y-lo-que-promete-el-esquema.md
[p17]: ./prototipos/17-el-vehiculo-del-zip/
[p15]: ./prototipos/15-plugin-de-juguete/
