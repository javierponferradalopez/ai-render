# El contrato del VisualDocument

Type: grilling
Status: open
Blocked by: 14

## Question

¿Cuál es la forma exacta del modelo semántico que el agente escribe y que el
Layout Engine consume?

Es la pieza central del proyecto: todo lo demás la rodea. Deliberadamente **no**
depende de qué renderer gane, así que puede resolverse ya.

A decidir:

- **Las primitivas.** La decisión de partida 5 fija el rumbo — `Node` con
  secciones de texto, `Edge` con `kind` semántico, `Group` anidable — pero falta
  la forma concreta: campos, cuáles son obligatorios, cómo se identifican, qué
  pasa con `Text` y `Note` (§8 del handoff los lista: ¿entran o sobran?).
- **El vocabulario de `kind` de las aristas.** `depends`, `extends`,
  `implements`, `contains`, `calls` es la propuesta. ¿Es el conjunto correcto
  para el caso de uso de refactor? ¿Cerrado o extensible? ¿Qué hace el renderer
  con un `kind` que no conoce?
- **Vistas.** La decisión de partida 6 dice que un canvas tiene N vistas con
  nombre. ¿Cómo se declara una vista? ¿Tiene título visible? ¿Cómo se ordenan
  entre sí?
- **La capa de azúcar semántico.** §8 quiere que el agente escriba
  `type: "architecture"` con listas simples en vez de primitivas. ¿Es una capa
  aparte que compila a primitivas, o el documento tiene un `type` y ya? Esto
  decide cuántos tokens escribe el agente en el caso común.
- **Identidad y referencias.** ¿Quién genera los ids — el agente o el servidor?
  De esto depende que `visual.update` pueda referirse a algo.
- **Validación contra `sequence`, en papel.** Antes de cerrar: escribir cómo
  sería un `visual.show({type:"sequence", ...})` y comprobar que el contrato lo
  admite sin romperse. Si no lo admite, el contrato está mal.

Deja escrito el contrato en un artefacto propio del esfuerzo y enlázalo. También
es el momento de arrancar `CONTEXT.md` con los términos del dominio.

## Context

[El MCP de tldraw](./04-mcp-de-tldraw.md) dejó vocabulario prestado que conviene
copiar en lugar de reinventar:

- El **formato *focused*** de tldraw (`src/widget/focused/`): ids string cortos,
  `_type` plano, paleta cerrada, nombres de forma legibles. Diseñado para que un
  modelo lo escriba sin equivocarse.
- **`fromId` / `toId` en las aristas** — el agente conecta por id, el servidor
  calcula la geometría. De ahí sale la mitad del ahorro en tokens.
- **Qué devolver al agente**: un resumen, nunca el estado. tldraw gasta ~730
  tokens empujando el lienzo entero tras cada operación; nosotros debemos gastar
  ~10-30 en una confirmación y cero en estado, salvo petición explícita.
- Referencia de coste a batir: el mismo diagrama de 5 nodos son **88 tokens en
  Mermaid** y **178** en un `visual.show` con `nodes[]`/`edges[]`.
