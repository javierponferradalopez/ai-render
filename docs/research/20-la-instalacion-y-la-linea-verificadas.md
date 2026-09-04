# La instalación de punta a punta, y la línea que la remata

Medición del 2026-09-04 pedida por [La instalación de punta a punta y la línea del
`CLAUDE.md`][45], que es el paso que convierte un release publicado en una pizarra que se
usa. Lo del vehículo ya estaba medido —el zip, la cuarentena, el `update`, el arranque—; lo
que quedaba era **el último paso de la instalación, que es el disparador principal del
producto**, y ahí la medición cambia la decisión.

**El nombre de la herramienta que la línea del §8.2 tenía escrito no existe.** El host no
presenta `mcp__flipchart__show`, sino **`mcp__plugin_flipchart_flipchart__show`**: el prefijo
`mcp__` lleva dentro el nombre del servidor, y para un plugin ese nombre es
`plugin:<plugin>:<servidor>`. Una instalación hecha al pie de la letra del README anterior
dejaba al usuario con una línea que nombra algo que no está.

Con el nombre corregido, **la línea funciona sobre el release de verdad**: en el caso
protagonista —el usuario cuenta el movimiento que se plantea y pide entender las
dependencias, **sin pedir dibujo**— el agente buscó la herramienta, la cargó y dibujó.

## Nota sobre el método

macOS **26.6.2** (build 25G83) arm64, Claude Code **2.1.228**, release **v0.1.0**. Banco:
[prototipo 26](./prototipos/26-la-instalacion-de-punta-a-punta/).

El plugin se instala en un `CLAUDE_CONFIG_DIR` propio desde el catálogo de `main` por
`raw.githubusercontent.com`, y **la caja que corre en los turnos es la que dejó el host**,
extraída en `plugins/cache/flipchart/flipchart/0.1.0/`. Los turnos, en cambio, la cargan con
`--plugin-dir` sobre la sesión del usuario: un `CLAUDE_CONFIG_DIR` recién creado **no tiene
sesión** —`claude -p` contesta `Not logged in`— y el nombre de la herramienta sale idéntico
por los dos caminos, así que no hay nada que el atajo mueva.

**Lo que sí paga el atajo, y va en contra:** la sesión del usuario lleva sus 26 conectores
dentro, y uno de ellos es `mcp__claude_ai_Figma__generate_diagram` — otra herramienta que
dibuja diagramas, compitiendo con la pizarra. El prototipo 22 corría con `--strict-mcp-config`
para que el sujeto no viera nada más, y aquí eso no se puede: **`--strict-mcp-config` apaga
también el servidor del plugin**, medido. El resultado es positivo con esa competencia dentro,
lo que lo hace más fuerte, no más flojo.

Y el instrumento es el mismo que el del §8.1: el `tool_use` del historial, porque **conceder
una herramienta MCP en modo `-p` es más difícil de lo que parece**. No la concede
`--allowedTools` —eso ya lo decía la nota de método del §8.1—, no la conceden los settings del
proyecto, que piden haber confiado antes en el directorio, y no la concede tampoco un
`permissions.allow` pasado con `--settings`. **La concede un hook `PreToolUse`** que contesta
`permissionDecision: allow`, y sólo para lo que empareja su `matcher`. De ahí que haya tres
corridas: dos que miden la conducta con la llamada denegada, y una que la deja llegar al
Servidor MCP. Lo que no se usa es `acceptEdits` ni `bypassPermissions`: el turno 2 del guion es
un «Sí, adelante», y los dos auto-aprueban las ediciones al margen de cualquier lista.

## 1. El nombre de la herramienta lo compone el host

Del evento `init` del stream, con el release instalado:

```
mcp_servers: [{"name": "plugin:flipchart:flipchart", "status": "connected"}]
tools:       ["mcp__plugin_flipchart_flipchart__clear",
              "mcp__plugin_flipchart_flipchart__show"]
```

El servidor se llama `flipchart` a sí mismo en el handshake —`serverInfo.name`— y el host lo
renombra a `plugin:<plugin>:<servidor>` (research 09 §1 ya lo había visto en los logs de
arranque). Al pasar a nombre de herramienta, los `:` se convierten en `_` — y encaja con lo que
el bundle da por válido, `mcp__[A-Za-z0-9_-]+__…`, que no admite los dos puntos (*leído del
bundle*, no es la causa medida).

| Dónde | Cómo se llama |
|---|---|
| `serverInfo.name`, en el handshake | `flipchart` |
| El servidor, para el host | `plugin:flipchart:flipchart` |
| **La herramienta, para el modelo** | **`mcp__plugin_flipchart_flipchart__show`** |

**Sale igual instalado desde el marketplace y cargado con `--plugin-dir`**, así que el nombre
no depende del marketplace del que se instale: sólo de que el plugin y su servidor se llamen
`flipchart`. La duplicación del nombre en el medio es eso: el plugin y el único servidor de su
`.mcp.json` se llaman lo mismo.

Consecuencia para el producto, y es la que importa: **la línea del §8.2 se escribe con ese
nombre**, y el §11.4 gana un límite —el día que el plugin o su servidor se renombren, la línea
que la gente tenga pegada deja de nombrar nada—.

## 2. El umbral de macOS, confirmado con el build

§10.7 tenía escrito «macOS 12 o superior, provisional, a confirmar con el primer build», y
research 19 lo dejó en su lista de lo no medido porque *«no lo dice el build»*. Sí lo dice: lo
declara el propio Mach-O.

| Rebanada | Carga | Umbral declarado |
|---|---|---|
| `arm64` | `LC_BUILD_VERSION` | **`minos 11.0`**, sdk 26.5 |
| `x86_64` | `LC_VERSION_MIN_MACOSX` | **`10.12`**, sdk 26.5 |

Por debajo de lo declarado no es que falle: **no lo arranca el cargador**. Y no hay ninguna API
weak-linked que pudiera subir el umbral por detrás: los cuatro símbolos `weak` del binario son
los helpers de ARC de `libobjc` (`_objc_initWeak` y compañía), y lo que toca de AppKit y
Foundation —`beginActivityWithOptions:reason:`, `setActivationPolicy:`,
`orderFrontRegardless`— es de 10.9 y anteriores. `eframe`/`winit` no piden más de lo que el
linker escribe.

**Se promete macOS 11**, que es el mayor de los dos y además el primero que existe en Apple
Silicon. Un número más bajo en Intel sería cierto y no serviría: nadie lo ha corrido ahí.

## 3. La línea, sobre el release: el agente dibuja sin que se lo pidan

El escenario `refactor` del prototipo 22, tres turnos, con la línea corregida en el
`CLAUDE.md` del sujeto y nada más. **El usuario no pide dibujo en ningún turno.**

| Turno | Lo que pide el usuario | La pizarra |
|---|---|---|
| 1 | entender qué depende de qué antes de mover nada | **`show("Dependencias actuales")`** |
| 2 | «Sí, adelante» | — (se le deniega escribir, y no dibuja) |
| 3 | «¿Y cómo quedaría después del movimiento?» | **`show("Después del movimiento")`** |

Dos dibujos en tres turnos, y son **los dos nombres del caso protagonista** que el §8.1 daba
como el resultado de la línea. La segunda corrida, con el mismo primer turno, salió con **dos
llamadas en un solo turno** —`Dependencias actuales` y `Quién escribe comment.line`—, que es
el otro dato del §8.1: la línea no produce un dibujo, produce varias Vistas conviviendo.

En medio hay un detalle del régimen de herramientas del host que conviene tener escrito: el
agente **cargó la herramienta con `ToolSearch` antes de llamarla**. Con búsqueda de
herramientas activa las herramientas MCP entran por el nombre y su schema se resuelve a demanda
(research 09 §8), así que la línea no sólo dispara la intención: es también lo que manda al
agente a buscar la herramienta por su nombre. Un nombre equivocado en la línea no es una
errata, es una búsqueda que no encuentra nada.

Los diagramas salen como el §9 y el §4.4 anticipaban: `graph TD` de los cuatro módulos con las
llamadas en las aristas, `<br/>` dentro de las etiquetas —el marcado que convive sin aviso— y
un `style` por debajo que el vaciado del §3.2 tira.

### Y la tubería, no sólo la intención

El control positivo —«usa la herramienta flipchart para enseñarme…»— con el permiso concedido
por el hook deja ver el otro extremo: la llamada llega al Servidor MCP del release y **la
ventana aparece**. El acuse que vuelve al agente, tal cual:

```
Shown as view "Módulos pickypen" (4 nodes, 6 edges). Views on the flipchart: Módulos pickypen.
Note: style directives (classDef, class, style, linkStyle) and click links were dropped —
the flipchart decides how views look. The view was drawn.
Note: the flipchart lays diagrams out left to right; the direction in your source was ignored.
The view was drawn.
```

Cuatro nodos, seis aristas, y **dos de los cuatro avisos del §4.4 acumulados en la misma
respuesta** —el estilo descartado y la dirección impuesta—, que es exactamente la regla de
*se avisa por lo que venía, no por lo que tuvo efecto*: el modelo había escrito cuatro `style`
y un `graph TD`.

## 4. Lo que ya estaba medido y aquí sólo se cita

- **La instalación de punta a punta sobre el release** — research 19 §5, y repetida hoy con el
  mismo resultado: `plugin install flipchart@flipchart` en 5,5 s, el universal entero en la
  caché (49 215 680 bytes, `100755`), `com.apple.provenance` y **ninguna cuarentena**, 47 MB de
  `CLAUDE_CONFIG_DIR`, y el servidor `connected` en la sesión.
- **El `update`** — research 15 §5 lo midió con un plugin sonda: con el bump completo,
  `updated from 1.0.0 to 2.0.0`; y sin subir la versión, `already at the latest version`
  **después de bajarse el zip entero y tirarlo**. Aquí se cierra con el plugin de verdad, y
  tiene su propio apartado: **§5**.
- **El `uninstall`** — research 09 §5, y **repetido hoy con el plugin de verdad**:
  `plugin uninstall flipchart` vació `plugins/data/flipchart-flipchart/`, dejó
  `installed_plugins.json` en `{}` y marcó la caja con un `.orphaned_at`. **El README no lleva
  ningún `rm -rf`** — y lleva, en cambio, el aviso que se sigue de eso: los 49 MB del binario
  **siguen en disco** hasta que pase la poda por antigüedad. Desinstalar libera los datos al
  instante y el espacio con retraso.

## 5. El `update`, cerrado con el release de verdad

Para esto se publicó **`v0.1.1`**: un release cuyo contenido es idéntico al de `v0.1.0` salvo
el número, gastado a sabiendas en medir lo que sólo se mide publicando. La CI corrió sus doce
pasos y commiteó el catálogo en `main`, que es lo que el host lee.

El banco tenía la **0.1.0 instalada desde antes de publicar**, que es la precondición que no se
puede saltar: instalar después es instalar ya la nueva y no hay update que medir.

| Medida | Valor |
|---|---|
| El catálogo tras `marketplace update` | `"version": "0.1.1"` |
| `plugin update flipchart@flipchart` | **`✔ Plugin "flipchart" updated from 0.1.0 to 0.1.1 for scope user`** |
| Lo que tardó | **2,2 s** |
| La caché después | **`0.1.0` y `0.1.1`, las dos** |
| El `CLAUDE_CONFIG_DIR` | 47 MB → **94 MB** |
| El binario nuevo | 49 215 664 bytes, `100755`, `com.apple.provenance` y ninguna cuarentena |
| Su handshake | `serverInfo: {"name":"flipchart","version":"0.1.1"}` |

**Ni un `already at the latest version`**, que era el desenlace que había que descartar. Y el
pico de disco de research 19 §2 queda confirmado por medición y no por estimación: se calculó
en ~98 MB para las dos versiones y son **94 MB**.

Que el handshake devuelva `0.1.1` cierra además la única duda que quedaba del camino: contesta
el binario nuevo, no el Lanzador —el Servidor de aviso se identifica como
`version: unavailable`—, así que la caja que el `update` trajo está entera y utilizable.

Y una trampa que va derecha al README: **`update` no resuelve el nombre corto.**

```
claude plugin update flipchart            ✘ Failed to update plugin "flipchart": Plugin "flipchart" not found
claude plugin update flipchart@flipchart  ✔ …
```

No es una regla general de los comandos de plugin: `uninstall flipchart`, con el nombre corto,
desinstaló sin queja. Lo que el `✘` no dice es que el plugin **estaba** instalado y
`plugin list` lo mostraba `enabled` — un usuario que lea ese error concluye que se le ha roto
la instalación.

## Lo que no se ha medido

- **Un macOS anterior al 26.6.2.** El umbral está declarado por el binario, no ejecutado.
- **La línea con otras redacciones.** Se mide la que va al README. Las cuatro que fracasaron
  eran del texto de la herramienta, no de esta línea (§8.1).

[45]: https://github.com/javierponferradalopez/ai-render/issues/45
