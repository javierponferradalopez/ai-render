# termaid y la terminal como superficie de entrega

Investigación disparada a mitad de la sesión de [El presupuesto de tokens][10],
al proponerse termaid como alternativa al visor web. Cambia la superficie de
entrega del MVP, así que revoca parte de [La superficie de entrega del visor][7].

## Nota sobre el método

Todo lo que sigue está **ejecutado**, no estimado: `termaid` 0.8.0 instalado desde
PyPI en un venv con Python 3.9, renderizando ficheros `.mmd` escritos a mano. Los
tokens se cuentan con `tiktoken` / `cl100k_base`, el mismo tokenizador que
[el informe de tldraw][r04], para que las cifras sean comparables. Los renders
citados se reprodujeron tal cual salen por stdout.

La muestra es **6 diagramas sintéticos** de entre 4 y 17 nodos, escritos por el
agente, no por un humano sobre un refactor real. Es suficiente para detectar los
fallos, no para acotar dónde empiezan. Ese hueco es el trabajo de [¿Se lee bien
un refactor real?][12].

## 1. Qué es

Paquete Python, **MIT**, cero dependencias, 428 estrellas, 126 commits. Renderiza
sintaxis Mermaid como arte Unicode para fuente monoespaciada. Soporta 18 tipos de
diagrama. `pip install termaid` o `uvx termaid`.

CLI: `--ascii`, `--width`, `--gap`, `--padding-x/y`, `--theme` (11 temas ANSI, vía
`rich`), `--tui` (visor interactivo con Textual), `--json TYPE`, `-o`.

## 2. El hecho que reabre la terminal

[La superficie de entrega del visor][7] descartó la terminal con tres motivos:
sin soporte universal, sin interacción, y *"el layout de un grafo en celdas de
texto es otro problema entero"*.

**El tercero está refutado**: termaid lo resuelve. El segundo no aplica — la
interacción está fuera de alcance por §18, el usuario observa. Y el primero se
disuelve si el visor es una **ventana de terminal propia** lanzada por el
servidor, no la terminal de la conversación.

### La distinción que decide el coste en tokens

La primera lectura de la propuesta fue "pintar el diagrama en la conversación de
Claude Code", y esa lectura **sí es fatal**: no existe canal del servidor MCP a la
pantalla. `stdout` es el canal JSON-RPC; escribir ahí rompe el protocolo. `/dev/tty`
pelea con el TUI de Claude Code, que redibuja por encima. La única vía es devolver
el ASCII como texto del tool result, y entonces entra en el contexto del modelo:
**393 tokens por render** para un diagrama de 6 clases. Es exactamente la
enfermedad que [el informe de tldraw][r04] diagnosticó como fatal — el 94 % del
coste de un retoque era re-volcar el lienzo.

Una **ventana de terminal aparte** no tiene ese problema: es un canal lateral que
esquiva el contexto del modelo, igual que el navegador. El servidor devuelve una
confirmación de ~22 tokens y el dibujo va por otro lado.

| Escenario: 1 `show` + 3 `update` | Tokens |
|---|---:|
| Ventana propia (terminal **o** navegador) | ~1.212 |
| ASCII devuelto como tool result (inline) | ~2.298 |

**El coste en tokens no distingue entre terminal y navegador.** La decisión se
juega en otros ejes.

## 3. Lo que termaid hace bien

- **Vocabulario UML completo** en `classDiagram`: `△` herencia, `△` punteado
  implementación, `◆` composición, `◇` agregación, `▼` asociación, `┆▼`
  dependencia. Las seis se distinguen. *(El enum `ArrowType` de 3 valores —
  `ARROW`/`CIRCLE`/`CROSS` — es del modelo de flowchart, no de `classDiagram`.)*
- **Secciones de clase** con separador, que es la decisión de partida 5: nombre,
  línea, atributos, línea, métodos.
- **Escala razonable**: 14 clases con miembros caben en **89 columnas × 35
  líneas** — la mitad del ancho y el 80 % de la altura de una terminal a pantalla
  completa en un portátil. No hay problema de tamaño al tamaño de un refactor.
- **`subgraph` anidado** en `flowchart`, con aristas cruzando fronteras de grupo.
- **El modelo interno mapea casi 1:1 con el `VisualDocument`.**
  `termaid.graph.model` expone `Graph`, `Node`, `Edge`, `Subgraph` como dataclases
  públicas (`from .graph.model import Graph` en `__init__.py`). `Subgraph` tiene
  `children` y `parent` — anidable de primera clase. `Edge` tiene
  `source_is_subgraph` / `target_is_subgraph` — aristas contra grupo, que es lo
  que hizo explotar a dagre en [Motores de layout][3].
- **`Graph.grid_positions`** — *"precomputed (col, row) for architecture
  diagrams"*. Permite **inyectar posiciones calculadas fuera**, así que la
  decisión de partida 3 (Layout Engine separable de la Drawing Surface) sobrevive:
  se podría seguir usando Graphviz para el layout.
- **`render_graph(graph: Graph, ...)`** en `termaid.renderer.draw` acepta un
  `Graph` directamente. El `render()` público solo acepta Mermaid en texto, pero
  no hay que pasar por Mermaid.
- **Resuelve el punto flojo de [La superficie de entrega del visor][7]**: su
  apartado 4 tuvo que inventar un "estado terminal honesto" porque una ventana de
  navegador **no puede cerrarse sola**. Un proceso en su propia terminal sí puede.

## 4. Carencia 1 — `namespace` se descarta en silencio

`classDiagram` con tres `namespace` (`application`, `domain`, `infrastructure`)
renderiza **cero** apariciones de los tres nombres. No es un fallo de layout: está
en una regex de líneas ignoradas a propósito, `parser/classdiagram.py:57`:

```python
# Lines to skip silently
_SKIP_RE = re.compile(
    r"^\s*(?:namespace\s|style\s|classDef\s|cssClass\s|click\s|callback\s|link\s)",
    re.IGNORECASE,
)
```

Consecuencia: **no hay capas ni carpetas en un diagrama de clases**, que es la
decisión de partida 5 (`Group` anidable como ciudadano de primera, "es lo que
representa carpetas y capas").

### Y no se puede compensar con flowchart

`flowchart` sí anida grupos, pero **no admite saltos de línea en un label**.
Probado con las dos sintaxis:

- Salto de línea real dentro de `"..."`: el parser trata cada línea como un
  **nodo distinto**, y las comillas se filtran a los nombres. Salen nodos
  llamados `OrderService["OrderService`, `o`, `+cancel(id)"]`, `-id`, `-lines`.
- `<br/>`, que es la sintaxis correcta de Mermaid: **se pinta literal**. El nodo
  dice `OrderService<br/>+place(o)<br/>+cancel(id)` en una sola línea.

`Node.label_segments` no ayuda: `LabelSegment` es negrita/cursiva en línea, y
`renderer/draw.py:203` suma anchos de display, no gestiona filas.

**Resultado: `flowchart` da contención sin secciones; `classDiagram` da secciones
sin contención. Son dos renderers distintos y no se combinan.**

### Coste de arreglarlo

`renderer/classdiagram.py` son **727 líneas con su propio layout y sin noción de
grupo**. La maquinaria de `subgraph` vive en `layout/subgraphs.py` (180 líneas) y
en `renderer/draw.py`, construida para el `GridLayout` de flowchart, y no es
reutilizable tal cual. `namespace` ya se parsea, solo se descarta — pero
renderizar cajas anidadas en `classDiagram` es una feature, no un parche.

**Es aditiva**: aportable upstream, y mientras tanto se vive sin capas.

## 5. Carencia 2 — el ruteo de aristas produce lecturas falsas

Esta es la grave, y sale en **todos** los renders de la muestra, desde 4 nodos.
El README lo admite en abstracto — *"layout engine is approximate. Node
positioning uses a grid-based barycenter heuristic"*, *"Manhattan-only edge
routing. Edges use A\* pathfinding on a grid"* — pero el efecto concreto es peor
que "algún cruce".

**Aristas que atraviesan cajas ajenas con punta de flecha en cada borde.**
14 clases, la arista es `OrderService → OrderRepository`:

```
│ OrderService │ ┌───────────────────┐ ┌──────────────┐ │ OrderRepository │
├──────────────┤ │     Payments      │ │  Shipments   │ ├─────────────────┤
│ +place(cmd)  │─┼───────────────────┼►┼──────────────┼►│ +save(o)        │
│ +cancel(id)  │ │ +charge(a: Money) │ │ +dispatch(o) │ │ +byId(id)       │
└──────────────┘ └───────────────────┘ └──────────────┘ └─────────────────┘
```

Se lee como una cadena `OrderService → Payments → Shipments → OrderRepository`.
**Esa cadena no existe en el fuente.**

**Aristas que parten el texto de un miembro.** 6 clases:

```
│ +save(o)        │────┼─+charge(a)───┼───►│ -lines       │
```

El miembro `+charge(a)` de `Payments` está cortado por una arista que no es suya.

**Fragmentos huérfanos.** `──▼` sin línea que lo alimente, `┌┐` vacíos,
`────────◆` desconectado, tees `├` arrancando del blanco.

**Bordes de caja corrompidos.** 17 nodos, flowchart LR:

```
│ │  PaymentGateway   ┼ │
```

Un `┼` incrustado en la pared de la caja, donde debería ir `│`.

**No es del renderer de `classDiagram`, es de termaid.** El flowchart de 17 nodos
y 4 capas se degrada igual: los bordes de `application` e `infrastructure` se
convierten en `┼` al ser cruzados, y trazar una sola arista de punta a punta deja
de ser posible a simple vista.

A 6 nodos los dos renderers salen limpios. A 14-17 los dos se rompen. **El punto
de ruptura está sin medir, y medirlo es el trabajo de [¿Se lee bien un refactor
real?][12].**

### Por qué esta no es aditiva

Es el A\* de termaid, en los dos renderers. "Mejorarlo después" significa adoptar
el motor de ruteo de otro proyecto o escribir el nuestro — que es exactamente el
trabajo que el camino del navegador ahorraba. **La ventaja de la terminal es
íntegramente prestada de termaid; si nos debemos su ruteo, la ventaja
desaparece.**

Para un proyecto cuya promesa es *entender un refactor antes de hacerlo*, un
dibujo en el que no puedes fiarte de las relaciones ataca la promesa central. No
es fealdad: es información falsa.

## 6. Carencias menores, pero reales

- **Python en un proyecto TypeScript.** Es literalmente la objeción con la que
  [La superficie de entrega del visor][7] descartó Tauri ("entra una toolchain de
  Rust en un proyecto TypeScript"). Y rompe el `npx ephemeral-visual-mcp` de §16,
  que es parte de la promesa de instalación.
- **Abrir la ventana de terminal es específico de plataforma.** `osascript` /
  Terminal.app / iTerm en macOS, otra cosa en Linux, otra en Windows. El navegador
  tiene `open` / `xdg-open` / `start` y es una línea.
- **Sin zoom.** La celda es el átomo: un nodo mide lo que mide su texto. No es un
  problema al tamaño medido (14 clases), pero fija un techo duro donde el
  navegador solo reduce escala.

## 7. Contexto competitivo

**El CLI de Cursor renderiza bloques Mermaid como ASCII inline desde el 18 de
febrero de 2026**, con `Ctrl+O` para alternar entre dibujo y fuente:

> "Mermaid code blocks now render inline as ASCII diagrams in your CLI
> conversation. Flowcharts, sequence diagrams, state machines, class diagrams,
> and ER diagrams can all be displayed directly in the terminal."

Es una función **del host**, no de un servidor MCP. Si Claude Code la copia, buena
parte de este proyecto queda absorbida por la plataforma. Eso pesa directamente
sobre [¿Qué añade esto sobre Mermaid?][15] y refuerza su salida (4), *parar*.

## Fuentes

- `termaid` 0.8.0 — <https://github.com/fasouto/termaid> · <https://pypi.org/project/termaid/>
- Código leído: `graph/model.py`, `parser/classdiagram.py`, `renderer/draw.py`,
  `renderer/classdiagram.py`, `layout/subgraphs.py`, `ingest.py`, `__init__.py`
- Changelog de Cursor CLI, 18-02-2026 — <https://cursor.com/changelog/cli-feb-18-2026>

[3]: https://github.com/javierponferradalopez/ai-render/issues/3
[7]: https://github.com/javierponferradalopez/ai-render/issues/7
[10]: https://github.com/javierponferradalopez/ai-render/issues/10
[12]: https://github.com/javierponferradalopez/ai-render/issues/12
[15]: https://github.com/javierponferradalopez/ai-render/issues/15
[r04]: ./04-mcp-de-tldraw.md
