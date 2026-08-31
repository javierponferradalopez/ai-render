# Mapa — Pizarra efímera para agentes (`ephemeral-visual-mcp`)

## Destination

Un **spec del MVP listo para implementar**: el documento del que salen tickets de
construcción sin que quede nada por decidir. Se llega cuando están resueltas las
incógnitas que hoy hacen que empezar a picar código sea una apuesta — cómo llega
el canvas al usuario, con qué se pinta y quién calcula el layout, cuál es la
forma exacta del protocolo semántico y de sus updates incrementales, y qué
significa "efímero" operativamente.

El MVP construido **no** es el destino de este mapa.

## Notes

**Dominio.** Un agente de IA (Claude Code) necesita un canal visual temporal para
explicarse: una pizarra. El agente expresa *semántica* (nodos, relaciones,
contención); nunca píxeles. El caso de uso protagonista es **entender un refactor
o un movimiento de estructura antes de hacerlo** — diagramas de clases junto con
flujos. Material de partida: [handoff.md](./handoff.md).

**Idioma.** Los artefactos de este esfuerzo se escriben en español; los
identificadores, nombres de tipos y código, en inglés.

**Skills a consultar en cada sesión.** `/grill-me` y `/domain-modeling` por
defecto. Los tickets `research` se resuelven con subagente.

**Decisiones de partida** — tomadas al cartografiar, valen como restricciones
para todo el esfuerzo:

1. **El destino es un spec, no el MVP construido.**
2. **El navegador en localhost es el suelo aceptable de entrega.** Cualquier
   integración embebida en Claude Code es una mejora encima, no un requisito. El
   proyecto no se ata a una capacidad de Claude Code que no controlamos.
3. **"Renderer" se parte en dos piezas de primera clase:** `VisualDocument →
   Layout Engine → PositionedScene → Drawing Surface`. tldraw bajó de decisión
   firme (§22.9 del handoff) a candidato — y **el candidato se cayó**: su
   licencia es propietaria y prohíbe el uso en producción. **§22.9 y el paquete
   `renderer-tldraw` de §19 quedan revocados.**
4. **Una sola familia de diagrama en el MVP: el grafo dirigido** (`architecture`,
   `flow`, `dependency-graph` y `class-diagram` son el mismo motor). El
   protocolo se valida **en papel** contra `sequence` antes de cerrarlo, sin
   implementarlo.
5. **Las primitivas suben de listón:** `Node` con secciones opcionales de texto
   (atributos y métodos de una clase), `Edge` con un `kind` semántico de
   vocabulario cerrado (`depends`, `extends`, `implements`, `contains`, `calls`),
   y `Group` **anidable** como ciudadano de primera — es lo que representa
   carpetas y capas.
6. **Un canvas contiene N vistas con nombre**, no un único diagrama: `show` sobre
   un `id` existente lo reemplaza, y vistas distintas conviven (actual vs
   propuesto; clases + flujo). **El servidor MCP es dueño del estado**; el visor
   es tonto y se puede recargar sin perder nada. **El canvas muere con la sesión
   MCP**, no con la disciplina del agente.

## Decisions so far

<!-- una línea por ticket cerrado: gist + enlace -->

- [Qué puede mostrar Claude Code hoy](./issues/01-que-puede-mostrar-claude-code.md)
  — Claude Code **no renderiza UI de servidores MCP** en ninguna superficie: el
  suelo del navegador contra `localhost` queda confirmado como única entrega. Pero
  **MCP Apps** (`ui://` en iframe sandboxeado) sí existe y lo soportan Claude
  web/Desktop, ChatGPT, Cursor y VS Code Copilot — así que el visor se construye
  como **página autocontenida**, y el mismo artefacto sirve a los dos mundos.
- [El MCP de tldraw: arquitectura y coste en tokens](./issues/04-mcp-de-tldraw.md)
  — existe, pero expone un **intérprete de JavaScript** (`exec`), no herramientas
  de dibujo, y vuelca el lienzo entero al contexto tras cada operación: un retoque
  de un nodo cuesta ~780 tokens, de los que 51 son la llamada. **No construir
  encima; robar el formato *focused*, `fromId`/`toId`, y su peaje fijo de ~900
  tokens como techo.**
- [Motores de layout para grafos dirigidos](./issues/02-motores-de-layout.md)
  — medido ejecutando los tres: **Graphviz WASM gana** (lienzo 6,7× más compacto
  que dagre y casi cuadrado, 7-20× más rápido, contención a tres niveles y
  aristas contra cluster), **elkjs es el plan B** y **dagre queda descartado**
  por lanzar excepción al conectar una arista a un grupo. Punto ciego de
  Graphviz: no puede hacer layout incremental estable, así que el motor va
  detrás de una interfaz sustituible.
- [Superficies de dibujo candidatas](./issues/03-superficies-de-dibujo.md)
  — **tldraw descartado por licencia**: es propietaria, prohíbe el uso en
  producción, obliga a que *cada usuario aguas abajo* licencie, y pinta un botón
  "Get a license for production" sobre el canvas que no se puede quitar ni en
  `localhost`. Recomendado **React Flow** (MIT, ~60 KB), con SVG propio como plan
  B barato. Texto de nodos en **HTML**, no `<tspan>`.

## Not yet specified

- **Cómo se le enseña al agente *cuándo* usar la pizarra.** Una herramienta que
  existe pero no se invoca no vale nada; y una que se invoca de más es ruido.
  Vive en las descripciones de las tools y quizá en instrucciones de proyecto.
  Se podrá concretar cuando la API de las tres herramientas esté cerrada.
- **Estilo visual por defecto** — tema, paleta, tipografía, densidad. Depende de
  qué Drawing Surface gane.
- **Estructura de packages del repo.** El handoff propone cinco (§19); el corte
  real depende del stack de rendering y de si `core` tiene entidad propia.
- **Cómo se testea un renderer visual** sin volverse loco con snapshots.
- **Qué pasa con diagramas grandes** (50+ nodos): legibilidad, viewport,
  colapsado de grupos, o simplemente un límite y un mensaje honesto.
- **Qué devuelve cada herramienta al agente.** Hay principio — resumen, nunca
  estado — pero no forma. Se concretará con la API de las tres herramientas.

## Out of scope

- Todo lo que el handoff lista en §23: cuentas, cloud, colaboración,
  persistencia, base de datos, historial, exportación, marketplace, múltiples
  renderers simultáneos, IA propia para generar layouts, sincronización entre
  máquinas.
- **Edición del diagrama por el usuario** — el usuario observa (§18).
- **Integración embebida en Claude Code**, si resulta que hoy no existe el
  mecanismo. Sería un esfuerzo posterior, no una parada de este mapa
  (decisión de partida 2).
- **Cualquier segunda familia de diagrama implementada** — `sequence`,
  `timeline`, `wireframe`, `mindmap`, `state-machine`, `ER` (decisión de
  partida 4).
