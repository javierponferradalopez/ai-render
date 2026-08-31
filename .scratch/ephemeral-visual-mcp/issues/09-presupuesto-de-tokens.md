> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/10 — copia congelada, no la edites.

# El presupuesto de tokens

Type: grilling
Status: open
Blocked by: 04

## Question

§22.14 declara el coste en tokens "preocupación de primer nivel", pero sin una
cifra eso no es un requisito: es un sentimiento. ¿Cuál es la barra medible?

A fijar:

- **El peaje fijo.** Cuántos tokens puede ocupar la descripción de las tres
  herramientas MCP juntas. Se paga en cada conversación, se use la pizarra o no,
  y es el coste más fácil de dejar engordar sin darse cuenta.
- **El coste de un `show` típico** — pongamos un diagrama de clases de 6 nodos
  con relaciones — y el de un `update` pequeño.
- **El coste de las respuestas** que las herramientas devuelven al agente.
- **Contra qué se compara.** Los números de
  [El MCP de tldraw](./04-mcp-de-tldraw.md) son la línea base. También conviene
  compararse con la alternativa honesta: que el agente pinte ASCII art en la
  conversación, que cuesta ~0 de instalación y un puñado de tokens. Si la
  pizarra no gana claramente a eso en algún eje, hay que saberlo.
- **Qué se sacrifica si no se cumple.** Menos azúcar semántico, descripciones más
  cortas, menos tipos de arista: decidir el orden de recorte por adelantado.

La cifra que salga de aquí condiciona el diseño de la API de las herramientas.

## Context

La línea base ya está medida en [El MCP de tldraw](./04-mcp-de-tldraw.md):

- **Peaje fijo de tldraw: ~900 tokens** con dos herramientas (`search` + `exec`).
  Es el techo a batir: tres herramientas semánticas deben caber por debajo.
- **Un retoque de un nodo: ~780 tokens**, de los que solo 51 son la llamada — el
  94 % es el re-volcado del lienzo entero. Con 20 nodos, 3.163 por retoque.
- **Escenario pintar + 3 retoques: ~5.000 tokens (tldraw) vs ~590** (semántico
  efímero con confirmación corta).
- **La alternativa honesta ya tiene número: Mermaid, 88 tokens** para el mismo
  diagrama — más barato que nuestro protocolo. Ver
  [¿Qué añade esto sobre Mermaid?](./14-que-anade-sobre-mermaid.md).
