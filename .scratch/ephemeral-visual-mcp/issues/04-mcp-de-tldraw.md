> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/5 — copia congelada, no la edites.

# El MCP de tldraw: arquitectura y coste en tokens

Type: research
Status: resolved

## Question

¿Qué hace exactamente el MCP oficial de tldraw, cuánto cuesta en tokens usarlo, y
merece la pena construir sobre él en vez de hacer un MCP propio?

Cubre §26.1, §26.5, §26.10 y §26.11 del handoff.

- **Arquitectura**: ¿cómo está montado? ¿Qué herramientas expone y con qué
  esquemas? ¿Habla de shapes y coordenadas, o de semántica? ¿Cómo conecta con su
  visor?
- **Coste real en tokens**: esto es lo más valioso del ticket. Medir, con
  números:
  - cuánto ocupan las descripciones de sus herramientas en el contexto, solo por
    estar el servidor conectado (ese peaje se paga en *cada* conversación, se use
    o no);
  - cuánto ocupa una llamada típica para pintar un diagrama de 5 nodos;
  - cuánto ocupa una modificación pequeña.
  Si no se puede medir directamente, estimar a partir de los esquemas publicados
  y decir explícitamente que es estimación.
- **Veredicto**: ¿construir encima, tomar prestadas ideas, o ignorarlo? Con el
  porqué.

Los números que salgan de aquí son la línea base contra la que se fija
[El presupuesto de tokens](./09-presupuesto-de-tokens.md).

## Context

Hallazgos: [research/04-mcp-de-tldraw.md](../research/04-mcp-de-tldraw.md)

## Answer

**Existe, es oficial, y no es lo que suponía el handoff. Veredicto: robar ideas,
no construir encima.**

El MCP de tldraw no expone herramientas de dibujo. Expone **dos herramientas —
`search` y `exec` — y `exec` es un intérprete de JavaScript** contra la instancia
viva del `Editor` de tldraw. El blog de tldraw que describe "una herramienta para
crear shapes, otra para editarlas y otra para borrarlas" está desactualizado
respecto al `tools/list` real. El visor es un **widget de MCP Apps** renderizado
en el iframe del host — lo que significa, cruzándolo con
[Qué puede mostrar Claude Code hoy](./01-que-puede-mostrar-claude-code.md), que
**el MCP de tldraw no dibuja nada en Claude Code**. Verificado: llamar a `exec`
sin widget conectado devuelve una respuesta degradada y el modelo nunca ve el
resultado.

### Las cifras (línea base para el presupuesto)

| Concepto | Tokens | |
|---|---:|---|
| Peaje fijo por tener el servidor conectado | **~900** | verificado |
| ...en un host que no filtra `_meta.ui.visibility` | ~1.350 | verificado |
| Descubrimiento vía `search` | 89 – **14.576** | verificado, llamadas reales |
| Diagrama de 5 nodos, camino típico | ~1.750 | llamada estimada |
| Modificación de un solo nodo | ~780 | estimado |
| ...de los cuales son la llamada en sí | 51 | |
| Modificación con el lienzo a 20 nodos | 3.163 | estimado |

**El hallazgo que decide el veredicto:** tras *cada* operación — y también tras
cada edición manual del usuario con el ratón — el widget vuelca el **lienzo
entero** al contexto del modelo (`getCurrentPageShapes()`, sin diff ni ventana).
El 94 % del coste de un retoque mínimo es ese re-volcado, y crece linealmente sin
techo con el tamaño del dibujo. Es una decisión arquitectónica del widget, no un
parámetro: no se apaga sin forkear.

Un escenario de "pintar 5 nodos y hacer 3 retoques" sale a **~5.000 tokens con
tldraw frente a ~590 con un MCP semántico efímero** — unas 8x, y la ratio empeora
cuanto más grande es el diagrama.

### Por qué no construir encima

1. El modelo de contexto es incompatible con tener presupuesto de tokens.
2. `exec` traslada al modelo la carga de conocer 331 métodos del Editor, y luego
   le cobra el descubrimiento. Las tres herramientas semánticas van justo en la
   dirección contraria, que es la correcta.
3. Ejecutar JS arbitrario generado por el modelo es superficie que no necesitamos.
4. Sin host MCP-Apps no dibuja nada — y Claude Code no lo es.
5. Durable Objects, 50 checkpoints, TTL de 7 días: es lo opuesto a efímero.

### Qué sí robar

- **El formato *focused*** (`src/widget/focused/`): ids string cortos, `_type`
  plano, paleta cerrada, nombres de forma humanos (`pill`, `cloud`). Está
  diseñado para que un modelo lo escriba sin equivocarse. Copiar el vocabulario.
- **`fromId`/`toId` en las flechas** — el agente conecta por id y el servidor
  calcula la geometría. De ahí sale la mitad del ahorro.
- **Los tres niveles de detalle del `agent-template`** (blurry / focused /
  peripheral) como modelo mental para *qué devolver* al agente: un resumen,
  nunca el estado.
- **~900 tokens de peaje fijo como techo a batir.** tldraw demuestra que un MCP
  de dibujo cabe por debajo de 1.000; tres herramientas semánticas bien
  descritas deben quedar por debajo de eso.

**Y lo que hay que invertir:** tldraw gasta ~730 tokens empujando estado tras
cada operación. Un MCP efímero debe gastar ~10-30 en la confirmación y **cero en
estado**, salvo que el agente lo pida.

Método, tablas completas, fuentes y el repaso a otros MCP de dibujo (Excalidraw,
Mermaid): [research/04-mcp-de-tldraw.md](../research/04-mcp-de-tldraw.md)
