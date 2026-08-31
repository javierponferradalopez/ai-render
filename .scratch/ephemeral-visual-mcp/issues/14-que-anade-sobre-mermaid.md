# ¿Qué añade esto sobre Mermaid?

Type: grilling
Status: open

## Question

Antes de diseñar un protocolo propio: ¿por qué no Mermaid?

[El MCP de tldraw](./04-mcp-de-tldraw.md) destapó, de pasada, el competidor real
de este proyecto — y no es tldraw. Es Mermaid:

- **Es más barato en tokens que nuestro propio protocolo semántico.** El mismo
  diagrama de 5 nodos: **88 tokens en Mermaid** contra **178** en un
  `visual.show` con `nodes[]` y `edges[]`. El coste en tokens era la
  preocupación de primer nivel (§22.14 del handoff), y perdemos en ella.
- **Ya resuelve el layout**, que es justo la pieza que nos falta y que motivó
  partir el renderer en dos.
- **Ya tiene diagramas de clases y `subgraph` para contención** — el caso de uso
  protagonista, tal cual.
- **Ya existe la pizarra efímera**: `claude-mermaid` es un MCP con preview y
  live-reload. Literalmente el producto del handoff, con Mermaid como lenguaje.
- **El agente ya sabe escribirlo** sin que le enseñemos nada. Nuestro protocolo
  hay que explicárselo, y esa explicación se paga en el peaje fijo de cada
  conversación.

Esto no es motivo para abandonar, pero sí para tener una respuesta honesta antes
de invertir en el contrato, el motor de layout y el adapter. Las candidatas a
respuesta, que hay que aceptar o tumbar una a una:

- **Updates incrementales.** Mermaid obliga a reenviar el diagrama entero. Pero
  si el diagrama entero son 88 tokens... ¿cuánto vale de verdad un patch? ¿A
  partir de qué tamaño empieza a importar?
- **Calidad visual y control del layout.** Mermaid decide por ti y su resultado
  con clases anidadas es discutible. ¿Es una diferencia que se note, o es
  quisquillosidad?
- **Vistas múltiples y persistentes en pantalla** (actual vs propuesto lado a
  lado), que un render de Mermaid no da.
- **Independencia del renderer**, que es un valor arquitectónico — pero
  arquitectura sin usuario no vale nada. ¿A quién le sirve?
- **Interacción futura**: hover, colapsar grupos, saltar al fichero. Mermaid
  produce una imagen; nosotros podríamos producir algo vivo. Pero eso está fuera
  del MVP, así que no puede justificar el MVP.

Salidas posibles, y todas son legítimas:

1. **Seguir con protocolo propio**, con las razones escritas y asumiendo que hay
   que ganarle a Mermaid en algo concreto y demostrable.
2. **Usar Mermaid como lenguaje de entrada** y quedarnos con la pizarra, el
   ciclo de vida efímero y las vistas múltiples — que es donde estaría el valor
   diferencial. El protocolo semántico desaparece o se adelgaza mucho.
3. **Mermaid como primer renderer** detrás de nuestro protocolo: el agente habla
   semántica, el adapter genera Mermaid. Barato de construir, y valida la
   arquitectura de adapters de §3 mejor que tldraw.
4. **Parar.** Si `claude-mermaid` ya hace el 90 % por 0 € de esfuerzo, saberlo
   ahora vale más que descubrirlo dentro de tres semanas.

Instala y prueba `claude-mermaid` antes de opinar. Media hora de uso real vale
más que una tabla comparativa.

Este ticket bloquea el contrato y el stack de rendering a propósito: si la
respuesta es (2), (3) o (4), gran parte de ese trabajo no se hace.
