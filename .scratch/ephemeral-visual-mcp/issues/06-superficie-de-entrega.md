# La superficie de entrega del visor

Type: grilling
Status: open
Blocked by: 01

## Question

Con los hechos sobre la mesa, ¿cómo llega el canvas a los ojos del usuario en el
MVP?

La decisión de partida 2 ya fija el suelo: **una ventana de navegador en
localhost es aceptable**. Lo que falta es todo lo demás:

- Si la investigación encontró un mecanismo embebido viable, ¿entra en el MVP,
  se queda como mejora posterior, o se descarta?
- **Quién abre la ventana y cuándo.** ¿La abre el comando de arranque? ¿La abre
  el servidor la primera vez que el agente llama a `visual.show`? ¿La abre el
  usuario a mano? Cada opción tiene un coste de fricción distinto y afecta
  directamente al criterio de éxito (§25: "en cuestión de segundos").
- **Qué ve el usuario cuando no hay nada que ver.** Una ventana en blanco
  permanente en el escritorio es un coste, no un producto.
- **Qué pasa si el usuario cierra la ventana** y luego el agente llama a
  `visual.show`.
- Si son varias las superficies posibles, ¿cuál es la del MVP y cuáles quedan
  fuera de alcance?

De esta decisión cuelgan el transporte y la instalación.

## Context

[Qué puede mostrar Claude Code hoy](./01-que-puede-mostrar-claude-code.md) ya
cerró el hecho: no hay mecanismo embebido en Claude Code, así que la primera
viñeta de arriba se responde sola — no entra en el MVP porque no existe.

Lo que ese ticket abre a cambio, y que hay que decidir aquí:

- **¿El visor se construye desde el principio como un recurso `ui://` de MCP
  Apps servible también como página suelta?** Es casi el mismo artefacto: HTML
  autocontenido que recibe una escena y la pinta. Hacerlo así cuesta poco ahora
  y significa que el día que Claude Code soporte MCP Apps —o que alguien use
  esto desde Claude Desktop, Cursor o ChatGPT, que ya lo soportan hoy— funciona
  sin reescribir nada.
- Si la respuesta es sí, ¿entra en el MVP servir *ambos* modos, o solo se
  respeta la restricción de diseño y se sirve uno?
- Restricciones que hereda el visor si quiere ser un MCP App: HTML con su JS y
  CSS empaquetados, sin acceso al DOM padre ni a cookies del host, orígenes
  externos declarados en `_meta.ui.csp`, y comunicación por `postMessage`
  (JSON-RPC, métodos `ui/…`) en lugar de una conexión propia. Ojo: eso choca de
  frente con "WebSocket contra nuestro servidor", así que el transporte no puede
  darse por supuesto — ver [Transporte y ciclo de vida](./12-transporte-y-ciclo-de-vida.md).
