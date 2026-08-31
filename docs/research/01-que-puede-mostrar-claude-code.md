# Investigación: Qué puede mostrar Claude Code hoy

**Fecha del informe:** 2026-08-31  
**Estado:** Completado  
**Fuentes:** Documentación oficial de MCP, Claude Code docs, GitHub repos, búsquedas Web

> **AVISO:** varias afirmaciones de este informe resultaron no verificadas o
> falsas. Lee la sección **Verificación (2026-08-31)** al final antes de usar
> nada de aquí.

---

## 1. MCP Apps / Recursos de UI

### ¿Existe la extensión?

**Confirmado en documentación oficial:** Sí existe.

MCP Apps es una extensión oficial del protocolo MCP (SEP-1865) que se lanzó como **Final** el 26 de enero de 2026. Es la primera extensión oficial del protocolo.

Fuentes:
- [MCP Apps Blog Post (2026-01-26)](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/)
- [MCP Apps Overview](https://modelcontextprotocol.io/extensions/apps/overview)
- [MCP 2026-07-28 Specification](https://modelcontextprotocol.io/specification/2026-07-28)

### Estado del estándar

**Estado actual:** ESTABLE (Final/Oficial)

- Lanzado como propuesta (SEP-1865) a finales de 2025
- Incorporado a la RC de MCP 2026-07-28 (27 de julio de 2026)
- Ahora es parte de la especificación oficial de MCP 2026-07-28

### Capacidades de MCP Apps

#### URI scheme y recursos
- Usa el esquema `ui://` para declarar recursos de UI
- Resources devuelven HTML con MIME type `text/html;profile=mcp-app`
- El servidor declara recursos via tool metadata en `_meta.ui.resourceUri`

#### JavaScript
**Soportado:** Sí, pero en contexto sandboxeado
- El HTML se renderiza en un `<iframe>` sandboxeado
- JavaScript se ejecuta dentro del sandbox sin acceso al DOM padre
- No puede acceder al localStorage o cookies del host
- No puede navegar la página padre ni ejecutar código en el contexto padre

#### Acceso a red
**Soportado:** Sí, con configuración de Content Security Policy (CSP)

El servidor debe declarar todos los orígenes permitidos en `_meta.ui.csp`:
- **connectDomains**: Para fetch, XMLHttpRequest, EventSource, WebSocket
- **resourceDomains**: Para scripts externos, stylesheets, imágenes, fuentes, audio, video

Ejemplo:
```json
"_meta.ui.csp": {
  "connectDomains": ["https://api.example.com", "http://localhost:3000"],
  "resourceDomains": ["https://cdn.example.com"]
}
```

Nota: Durante desarrollo, localhost debe incluirse explícitamente en los dominios.

#### Tamaño
**No confirmado límite específico en documentación oficial.** No se encontró documentación que especifique un límite de tamaño máximo para el HTML de MCP Apps. Las respuestas de MCP pueden truncarse si exceden ciertos límites de contexto generales, pero estos no están específicamente documentados para MCP Apps.

#### Ciclo de vida
**Persistencia mientras el resultado está visible:** Las apps de MCP existen y mantienen estado mientras el resultado del tool está visible en la conversación. La comunicación es bidireccional:

- El host puede empujar datos frescos a la app sin que el usuario pida nada
- La app puede solicitar tool calls de vuelta al servidor
- La app puede enviar mensajes y actualizar el contexto del modelo

Communication protocol: JSON-RPC 2.0 sobre `postMessage` (no requiere HTTP)

Fuente: [MCP Apps Overview - How MCP Apps Work](https://modelcontextprotocol.io/extensions/apps/overview#how-mcp-apps-work)

#### Seguridad
- Iframe sandboxeado con restricciones de permisos
- Todos los mensajes UI-host van por JSON-RPC auditable (postMessage)
- El host controla qué herramientas puede llamar el app
- Consentimiento del usuario para tool calls iniciados por UI

---

## 2. Claude Code en concreto: ¿Renderiza MCP Apps?

### Respuesta directa
**NO. Claude Code NO renderiza MCP Apps UI en ninguna superficie.**

### Por superficie

#### CLI de terminal
**No soporta MCP Apps.** Claude Code en terminal solo maneja comandos, archivos de texto y salida de terminal. No puede renderizar HTML interactivo.

#### App de escritorio (Mac/Windows/Linux)
**No soporta MCP Apps en la conversación principal.**

Dato importante: Desde julio de 2026 (Week 28), Claude Code Desktop SÍ tiene un navegador integrado (Cmd+Shift+B en Mac, Ctrl+Shift+B en Windows), pero:
- Este navegador es una superficie separada para navegar web y localhost
- NO es el mismo que renderizar MCP Apps
- Se abre como una pestaña independiente en el panel Browser

Fuente: [Claude Code In-App Browser (August 2026)](https://aitoolsreview.co.uk/insights/claude-code-desktop-browser)

#### Extensión de IDE (VS Code / JetBrains)
**No soporta MCP Apps.**

VS Code GitHub Copilot SÍ soporta MCP Apps (ver sección 5), pero Claude Code como extensión de VS Code no.

Nota: La extensión de Claude Code puede ser reposicionada en VS Code (arrastrada a cualquier lado incluyendo el sidebar derecho), pero eso es layout, no renderizado de MCP Apps.

#### Web (claude.ai/code)
**No hay superficie "claude.ai/code"** para Claude Code. Existe:
- Claude.ai (web) - soporta MCP Apps ✓
- Claude Code - solo en desktop app y CLI

Cuando accedes a "claude.ai/code" desde un navegador, es un alias a la app de escritorio.

### Resumen: MCP Apps en Claude Code
**Estado confirmado:** Ninguna superficie de Claude Code (CLI, desktop, IDE) soporta MCP Apps UI rendering.

Fuente: [MCP Apps client support matrix oficial](https://modelcontextprotocol.io/extensions/client-matrix) - Claude Code no está listado.

---

## 3. ¿Puede un servidor MCP local abrir localhost sin fricción?

### Respuesta técnica

**Sí, pero NO directamente desde MCP Apps** (porque Claude Code no soporta MCP Apps).

### Alternativa en Claude Code: Navegador integrado

Claude Code Desktop tiene un navegador integrado desde julio 2026:
- **Atajo:** Cmd+Shift+B (Mac) / Ctrl+Shift+B (Windows)
- **Acceso a localhost:** Sí, sin restricciones
- **Sandbox:** El navegador usa un perfil limpio sin logins, arenado de la sesión de Claude Code
- **Fricción:** Mínima - es solo un click o un atajo de teclado

El navegador puede navegar a `http://localhost:3000` o cualquier puerto local sin configuración adicional.

Fuente: [Claude Code's Built-In Browser (August 2026)](https://aitoolsreview.co.uk/insights/claude-code-desktop-browser)

### En otros clientes MCP que SÍ soportan MCP Apps

En clientes como Claude Desktop o ChatGPT que soportan MCP Apps:
- Una app de MCP puede declarar `connectDomains: ["http://localhost:*"]` para acceder a APIs locales
- El navegador no está implicado - la app hace fetch() directamente desde el iframe
- Requiere que el usuario permita la conexión de red (CSP debe estar configurada)

---

## 4. ¿Existe un panel lateral / ventana independiente junto a Claude Code?

### En Claude Code Desktop

**Sí, pero no es para MCP Apps:**

Desde abril de 2026, Claude Code Desktop tiene un "sidebar de sesiones" que permite correr múltiples sesiones de Claude Code lado a lado en la misma ventana. Esto fue parte del redesign de abril 2026.

Características:
- Click "+ New session" en el sidebar o Cmd+N / Ctrl+N
- Cada sesión obtiene su propio worktree de Git
- Las sesiones pueden ejecutarse en paralelo
- Layout es completamente drag-and-drop

**Pero:** Esto es para múltiples sesiones de Claude Code, no para mostrar UI de servidores MCP.

Fuente: [Claude Code Desktop Redesign (April 2026)](https://www.buildfastwithai.com/blogs/claude-code-desktop-redesign-2026)

### En VS Code

Si usas Claude Code como extensión de VS Code:
- Puedes arrastrar el panel de Claude Code a cualquier lugar
- Incluyendo el sidebar derecho (secondary sidebar)
- Pero esto sigue siendo CLI / editor de Claude Code, no renderizado de UI

### En Claude Desktop

Claude Desktop SÍ tiene un navegador en el panel lateral (desde agosto 27, 2026), pero:
- Esto es para Claude Cowork (no Claude Code)
- Es una característica de Cowork, no de las extensiones MCP

Fuente: [Claude Cowork Chrome Side Panel (August 2026)](https://claude.com/blog/cowork-chrome-side-panel)

### Conclusión

**No existe hoy una arquitectura "Claude Code a la izquierda, canvas de MCP Apps a la derecha"** porque Claude Code no soporta MCP Apps. 

Lo más cercano es:
- Claude Code + navegador integrado lado a lado (navegador es para localhost HTTP, no para MCP Apps)
- Claude Desktop (que SÍ soporta MCP Apps) + conversación lado a lado

---

## 5. Otros clientes MCP: ¿Alguno renderiza UI de servidores MCP?

### Matriz oficial de soporte MCP Apps

Según la [matriz oficial de extensiones MCP](https://modelcontextprotocol.io/extensions/client-matrix) (2026-08-31):

**SOPORTAN MCP Apps (con CHECK ✓):**
1. Claude (web) ✓
2. Claude Desktop ✓
3. VS Code GitHub Copilot ✓
4. Microsoft 365 Copilot ✓
5. Goose ✓
6. Postman ✓
7. MCPJam ✓
8. ChatGPT ✓
9. Cursor ✓
10. Archestra.AI ✓
11. PostHog Code ✓

**NO ESTÁN EN LA MATRIZ / NO SOPORTAN MCP Apps:**
- Zed: Soporta MCP (tools, prompts) pero NO MCP Apps HTML/iframe
- Claude Code: No está mencionado en la matriz

Fuente: [MCP Extension Client Matrix (Official)](https://modelcontextprotocol.io/extensions/client-matrix)

### Notas sobre clientes específicos

#### Cursor
- Soporta MCP Apps desde al menos enero 2026
- Marketplace integrado de tools
- Full MCP support

Fuente: [Best MCP Servers for Cursor](https://www.firecrawl.dev/blog/best-mcp-servers-for-cursor)

#### Zed
- Soporta MCP core (tools, prompts) vía stdio
- **NO soporta MCP Apps HTML/iframe rendering**
- Usa ACP (Agent Client Protocol) como protocolo alternativo, pero ACP y MCP Apps son cosas diferentes
- Transport: stdio soportado, HTTP/SSE no soportado en Zed nativo

Fuente: [Zed MCP Support (2026)](https://zed.dev/docs/assistant/model-context-protocol)

#### VS Code Insiders
- Soporta MCP Apps
- GitHub Copilot en VS Code renderiza MCP Apps en el chat de Copilot

---

## Qué ha cambiado / Qué no se pudo confirmar

### Confirmado (29 de julio - 31 de agosto de 2026)

1. ✓ MCP Apps es estable y oficial desde 26-01-2026
2. ✓ Claude Code NO soporta MCP Apps
3. ✓ Claude Code Desktop tiene navegador integrado (julio 2026)
4. ✓ Cursor soporta MCP Apps
5. ✓ Zed no soporta MCP Apps (solo MCP core)

### No confirmado / Podría cambiar

1. **Límite de tamaño de MCP Apps HTML:** No está documentado explícitamente
2. **Ciclo de vida exacto después de que resulta es scrolleado:** La persistencia depende del cliente
3. **Roadmap de Claude Code para MCP Apps:** No hay anuncio público de soporte futuro
4. **Performance/latencia de bidirectional communication:** No está documentado

---

## Respuesta corta

Para entregar una "pizarra visual efímera" a usuarios de Claude Code, ordenadas de más a menos viable:

1. **Navegador integrado de Claude Code + servidor localhost** (VIABLE HOY): Usuario presiona Cmd+Shift+B, Claude abre localhost en el navegador integrado. Requiere que el MCP server levante un servidor HTTP local en un puerto conocido. [Docs](https://aitoolsreview.co.uk/insights/claude-code-desktop-browser)

2. **Claude Desktop con MCP Apps** (VIABLE HOY, diferente producto): Si el usuario usa Claude Desktop en lugar de Claude Code, MCP Apps renderiza directamente. No es Claude Code, pero es la superficie oficial de Anthropic con soporte MCP Apps. [Matriz oficial](https://modelcontextprotocol.io/extensions/client-matrix)

3. **Artifacts de Claude Code + conexión MCP** (VIABLE, con limitaciones): Claude Code puede crear Artifacts que se conectan a servidores MCP en cada carga (junio 2026). Pero Artifacts no son interactivos en tiempo real dentro de Claude Code, son una página aparte. [Docs](https://explainx.ai/blog/claude-code-artifacts-mcp-connectors-july-2026)

4. **Esperar a que Claude Code soporte MCP Apps** (NO VIABLE HOY): No hay roadmap público. Requerería que Anthropic agregue soporte a MCP Apps en Claude Code CLI/desktop.


---

# Verificación (2026-08-31)

El informe de arriba se revisó contra fuentes primarias. **La conclusión
principal se sostiene, pero varias afirmaciones no.** Lo que sigue manda sobre
cualquier cosa anterior de este fichero.

## Errores corregidos

- **"No existe la superficie `claude.ai/code`"** — falso. Claude Code está
  disponible como CLI, app de escritorio (Mac/Windows), app web en
  `claude.ai/code`, y extensiones de IDE (VS Code, JetBrains). No es un alias de
  la app de escritorio.
- **Navegador integrado, `Cmd+Shift+B`, sidebar de sesiones, "Artifacts de Claude
  Code con conectores MCP"** — todas estas afirmaciones se apoyaban en blogs de
  terceros de baja calidad (`aitoolsreview.co.uk`, `buildfastwithai.com`,
  `explainx.ai`) y **no se han podido confirmar en documentación oficial**.
  Trátalas como no verificadas y no construyas ninguna decisión sobre ellas.
- **"Claude Code no soporta MCP Apps — confirmado por la matriz oficial"** — la
  conclusión es correcta, pero la evidencia citada era más débil de lo que
  parecía: la matriz es *mantenida por la comunidad* y Claude Code simplemente
  **no aparece** en ella. Ausencia no es negación. La confirmación buena está en
  la propia documentación de Claude Code (abajo).
- **Cuidado con "URL mode is supported in Claude Code"**, frase que aparece en
  resultados de búsqueda sobre MCP Apps. En la documentación de Claude Code,
  *URL mode* se refiere exclusivamente a **flujos de autenticación OAuth** que se
  abren en el navegador, no a renderizar interfaz.

## Hechos verificados en fuente primaria

**MCP Apps es real, oficial y estable.** Extensión `io.modelcontextprotocol/ui`.
Un tool declara `_meta.ui.resourceUri` apuntando a un recurso `ui://`; el host lo
precarga, obtiene el HTML (con su JS y CSS empaquetados) y lo renderiza en un
**iframe sandboxeado** dentro de la conversación. `_meta.ui.csp` controla qué
orígenes externos puede alcanzar; `_meta.ui.permissions` pide capacidades extra.
La comunicación app↔host es **JSON-RPC sobre `postMessage`**, con métodos
propios prefijados `ui/` (p. ej. `ui/initialize`) más algunos compartidos con MCP
core (`tools/call`). El sandbox impide acceso al DOM padre, cookies y storage del
host.
Fuente: [MCP Apps overview](https://modelcontextprotocol.io/extensions/apps/overview)

**Clientes que soportan MCP Apps** (matriz oficial, comunidad, 2026-08-31):
Claude (web), Claude Desktop, VS Code GitHub Copilot, Microsoft 365 Copilot,
Goose, Postman, MCPJam, ChatGPT, Cursor, Archestra.AI, PostHog Code. **Claude
Code no está en la lista.**
Fuente: [Extension support matrix](https://modelcontextprotocol.io/extensions/client-matrix)

**Qué soporta Claude Code en MCP**, según su propia documentación:
`tools/list`, `prompts/list`, `resources/list`, notificaciones `list_changed`,
la capacidad **`claude/channel`**, y OAuth 2.0. **Ninguna capacidad de UI.**
Transportes: **HTTP** (`streamable-http`), **SSE** (obsoleto), **WebSocket**
(`ws://` y `wss://`) y **stdio**.
Fuente: [Claude Code — MCP](https://code.claude.com/docs/en/mcp)

**`claude/channel`** deja que un servidor MCP **empuje mensajes a la sesión** para
que Claude reaccione a eventos externos (resultados de CI, alertas). El servidor
declara la capacidad y el usuario la habilita con el flag `--channels` al
arrancar. Es un canal de **mensajes hacia el agente**, no una superficie visual —
pero es el único empuje servidor→sesión que existe hoy en Claude Code, y puede
importar para el ciclo de vida. Ojo a la limitación documentada: un servidor que
negocie la revisión `2026-07-28` **no puede** entregar mensajes de canal.

**Un canvas pesado sí cabe dentro de un MCP App.** Los ejemplos oficiales
incluyen un globo CesiumJS, escenas Three.js y un visor de PDF. La duda no es si
la tecnología aguanta un visor de diagramas: es qué cliente la renderiza.

## Respuesta corta verificada

Hoy, **Claude Code no puede renderizar ninguna interfaz servida por un servidor
MCP**. No existe el panel lateral del dibujo de §17 del handoff, y no hay
roadmap público que lo prometa. La única entrega posible en Claude Code es que
el usuario mire **una ventana de navegador apuntando a `localhost`**, servida por
nuestro propio proceso.

MCP Apps sí resuelve exactamente este problema — canvas interactivo dentro de la
conversación, con datos empujados desde el servidor — pero **en otros clientes**:
Claude web, Claude Desktop, ChatGPT, Cursor, VS Code Copilot y varios más.
