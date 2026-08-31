# Qué puede mostrar Claude Code hoy

Type: research
Status: resolved

## Question

¿Qué mecanismos existen **hoy** para que un servidor MCP local haga aparecer una
superficie visual delante del usuario de Claude Code, y qué soportan realmente
los clientes MCP más comunes?

Hay que averiguar hechos, no opinar. En concreto:

- **MCP Apps / recursos de UI**: ¿existe una extensión del protocolo MCP para que
  un servidor devuelva interfaz (`ui://`, `mcp-ui`, recursos HTML embebidos)? ¿En
  qué estado está — propuesta, experimental, estable? ¿Qué puede y qué no puede
  hacer una UI así (JS, red, tamaño, ciclo de vida, comunicación de vuelta al
  servidor)?
- **Claude Code en concreto**: ¿renderiza algo de eso? ¿En la CLI de terminal, en
  la app de escritorio, en la extensión de IDE, en la web? Las respuestas pueden
  ser distintas por superficie y hay que separarlas.
- **Abrir una ventana**: ¿puede un servidor MCP local abrir una pestaña de
  navegador en `localhost` sin fricción, y hay algún impedimento (sandbox,
  permisos, headless)?
- **Panel lateral / ventana independiente**: ¿existe hoy algo que se parezca al
  dibujo de §17 del handoff — Claude Code a la izquierda, canvas a la derecha?
- **Otros clientes MCP** (Claude Desktop, Cursor, VS Code, Zed): ¿alguno ya
  renderiza UI de servidores MCP? Interesa como señal de hacia dónde va el
  ecosistema, no como objetivo del MVP.

Fechar los hallazgos: el ecosistema MCP se mueve rápido y una respuesta sin
fecha caduca en silencio.

Esto **no** decide la superficie de entrega — eso es
[La superficie de entrega del visor](./06-superficie-de-entrega.md). Aquí solo se
levantan los hechos.

## Context

Hallazgos: [research/01-que-puede-mostrar-claude-code.md](../research/01-que-puede-mostrar-claude-code.md)

## Answer

**Claude Code no puede renderizar hoy ninguna interfaz servida por un servidor
MCP, en ninguna de sus superficies.** El panel lateral que dibujaba §17 del
handoff no existe, y no hay roadmap público que lo prometa. La única entrega
posible en Claude Code es una **ventana de navegador contra `localhost`** servida
por nuestro propio proceso — que es exactamente el suelo que fijó la decisión de
partida 2, ahora confirmado por hechos y no por prudencia.

Verificado en fuente primaria:

- **MCP Apps existe, es oficial y es estable** (extensión
  `io.modelcontextprotocol/ui`). Un tool declara `_meta.ui.resourceUri` → recurso
  `ui://` con HTML autocontenido → el host lo renderiza en un **iframe
  sandboxeado dentro de la conversación**, con `_meta.ui.csp` controlando los
  orígenes alcanzables y JSON-RPC sobre `postMessage` (métodos `ui/…`) para la
  comunicación bidireccional.
- **Lo soportan Claude web, Claude Desktop, Claude Cowork, ChatGPT, Cursor, VS
  Code Copilot, Microsoft 365 Copilot, Goose, Postman, MCPJam, Archestra.AI y
  PostHog Code. Claude Code no.**
- **Lo que Claude Code sí soporta en MCP**: `tools/list`, `prompts/list`,
  `resources/list`, notificaciones `list_changed`, `claude/channel` y OAuth 2.0.
  Transportes: HTTP (`streamable-http`), SSE (obsoleto), **WebSocket** (`ws://`,
  `wss://`) y stdio.
- **`claude/channel`** permite a un servidor **empujar mensajes a la sesión**
  (habilitado por el usuario con `--channels`) para que el agente reaccione a
  eventos externos. Es empuje hacia el *agente*, no hacia los ojos del usuario —
  pero es el único canal servidor→sesión que existe. Limitación documentada: un
  servidor que negocie la revisión `2026-07-28` no puede entregar mensajes de
  canal.
- **Un canvas pesado cabe sin problema dentro de un MCP App**: los ejemplos
  oficiales incluyen CesiumJS, Three.js y un visor de PDF. La tecnología no es el
  límite; el cliente sí.

**Consecuencia para el mapa, y es grande:** el visor que necesitamos para el
suelo de Claude Code — una página web autocontenida que recibe una escena y la
pinta — es *casi exactamente* lo que un recurso `ui://` debe ser. Si se
construye desde el principio como una página autocontenida sin suposiciones
sobre quién la aloja, el mismo artefacto sirve a Claude Code vía navegador y a
Claude Desktop, ChatGPT o Cursor vía MCP Apps, sin trabajo tirado. Esto deja de
ser "mejora posterior improbable" y pasa a ser **una restricción de diseño
barata que conviene respetar ya**.

Detalle, fuentes y correcciones del informe original (que contenía varias
afirmaciones falsas apoyadas en blogs de terceros):
[research/01-que-puede-mostrar-claude-code.md](../research/01-que-puede-mostrar-claude-code.md)
