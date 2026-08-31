# Transporte y ciclo de vida

Type: grilling
Status: open
Blocked by: 06

## Question

¿Cómo viaja el estado del servidor MCP al visor, y cómo nace y muere un canvas?

La decisión de partida 6 fija el principio — el servidor es dueño del estado, el
visor es tonto, el canvas muere con la sesión MCP — pero no la mecánica:

- **El transporte.** §14 propone WebSocket. ¿Sigue siendo la elección con la
  superficie de entrega ya decidida, o hay algo más simple (SSE, polling) que
  baste para un visor de un solo consumidor?
- **Arranque y descubrimiento.** ¿Quién levanta el servidor HTTP/WS? ¿En qué
  puerto? ¿Qué pasa si está ocupado? ¿Cómo sabe el visor a dónde conectarse?
- **Conexión tardía y reconexión.** El visor se abre después del primer `show`, o
  se recarga: ¿pide el estado completo? ¿El servidor lo empuja?
- **Sesión MCP ↔ canvas.** ¿Cómo detecta el servidor que la sesión terminó? ¿Qué
  ve el usuario en ese momento — el canvas se vacía, la ventana se cierra, se
  queda el último diagrama con una marca de "sesión terminada"?
- **Varias sesiones a la vez.** Si el usuario tiene dos Claude Code abiertos en
  dos repos, ¿comparten visor, pelean por el puerto, o cada uno tiene el suyo?
  Esto no es un caso raro: es el martes de cualquiera.
- **Seguridad mínima.** Un servidor que escucha en localhost y acepta comandos de
  dibujo: ¿hace falta algún token de sesión, o el riesgo es despreciable?

## Context

Hechos verificados en
[Qué puede mostrar Claude Code hoy](./01-que-puede-mostrar-claude-code.md) que
tocan a este ticket:

- **Claude Code soporta WebSocket como transporte MCP** (`ws://`, `wss://`),
  además de HTTP (`streamable-http`), SSE (obsoleto) y stdio. Ojo: eso es el
  transporte entre agente y servidor, no entre servidor y visor — pero cambia
  las opciones de arranque.
- **`claude/channel`**: un servidor puede **empujar mensajes a la sesión** para
  que el agente reaccione a eventos externos, si el usuario arranca con
  `--channels`. Es el único empuje servidor→sesión que existe en Claude Code.
  ¿Sirve de algo aquí — avisar al agente de que el visor se cerró, de que el
  usuario no está mirando, de que el layout falló? Tiene una limitación
  documentada: un servidor que negocie la revisión `2026-07-28` no puede
  entregar mensajes de canal.
- Si el visor va a poder vivir como recurso `ui://`, su canal natural es
  `postMessage` contra el host, **no** un WebSocket propio. Decidir si se
  soportan los dos caminos o solo uno.
