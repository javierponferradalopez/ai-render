# Plugins de Claude Code como capa de empaquetado

Investigación disparada al preguntarse si el proyecto debería ser un plugin de
Claude Code en lugar de un servidor MCP. **La premisa era falsa: no son
alternativas.** Un plugin *empaqueta* un servidor MCP, entre otras cosas. De ahí
sale la decisión de partida 7 del mapa, y respuesta parcial a cuatro tickets.

## Nota sobre el método

Todo lo que sigue viene de la **documentación oficial** de Claude Code
(`code.claude.com/docs/en/`), citada literalmente. Nada está verificado
ejecutándolo: no se ha construido un plugin de prueba. Las citas son sólidas; el
comportamiento real bajo carga —tiempos de arranque, fiabilidad de los hooks,
qué pasa si el hook de instalación falla— **está sin comprobar**, y es lo primero
que debe verificar quien implemente.

## 1. Qué es un plugin, exactamente

Un directorio con manifiesto en `.claude-plugin/plugin.json`. Componentes que
puede llevar, todos en la raíz del plugin (no dentro de `.claude-plugin/`):

| Directorio | Para qué |
|---|---|
| `skills/` | Skills como `<nombre>/SKILL.md`. Invocables por el modelo o solo por `/` |
| `commands/` | Skills como ficheros Markdown planos (legado; usar `skills/`) |
| `agents/` | Definiciones de subagentes |
| `hooks/hooks.json` | Manejadores de eventos del ciclo de vida |
| **`.mcp.json`** | **Configuración de servidores MCP** |
| `.lsp.json` | Servidores LSP |
| `monitors/monitors.json` | Monitores en background |
| `bin/` | Ejecutables añadidos al `PATH` del Bash tool mientras el plugin esté activo |
| `settings.json` | Ajustes por defecto al activar el plugin (`agent`, `subagentStatusLine`) |

Se instala con `/plugin install` desde un marketplace, o se prueba en local con
`claude --plugin-dir ./mi-plugin` (acepta también un `.zip`, y hay `--plugin-url`
para un archivo remoto). Los skills de plugin van **namespaced**:
`/nombre-del-plugin:skill`.

## 2. El hallazgo: el plugin envuelve al MCP, no lo sustituye

`.mcp.json` en la raíz del plugin, con el mismo formato de siempre y
`${CLAUDE_PLUGIN_ROOT}` disponible:

```json
{
  "mcpServers": {
    "plugin-database": {
      "command": "${CLAUDE_PLUGIN_ROOT}/servers/db-server",
      "args": ["--config", "${CLAUDE_PLUGIN_ROOT}/config.json"]
    }
  }
}
```

Comportamiento, literal:

> - Plugin MCP servers start automatically when the plugin is enabled
> - Servers appear as standard MCP tools in Claude's toolkit
> - Plugin servers can be configured independently of user MCP servers
> - If you run `/reload-plugins` mid-session, Claude Code keeps the live
>   connections of servers whose configuration is unchanged

**Consecuencia para la decisión de partida 2.** El plugin es **estrictamente
aditivo**: el servidor MCP de dentro sigue sirviendo a cualquier host que hable
MCP, instalado a mano. No se ata el proyecto a una capacidad de Claude Code que no
controlamos; se le añade una capa que solo Claude Code aprovecha.

## 3. Lo que el plugin **no** compra: ni un token

> "Plugin MCP servers start automatically when the plugin is enabled."

**Arrancan automáticamente y sus tools aparecen en el toolkit estándar.** El peaje
fijo medido —**738 tokens** con tres herramientas semánticas de descripciones
escuetas— se paga igual, en todas las conversaciones, se use la pizarra o no. El
plugin cambia la distribución, no el coste. La pregunta de
[¿Servidor MCP o skill de Claude Code?][18] sobrevive intacta.

La documentación **no** describe forma de filtrar qué tools expone un servidor MCP
de plugin, ni de arrancarlo bajo demanda.

### Y dos cosas que parecen canal lateral y no lo son

- **`monitors/`**: *"Each stdout line from `command` is delivered to Claude as a
  notification during the session."* Va **al contexto del modelo**.
- **`bin/`** invocado desde el Bash tool: la salida del comando va al contexto,
  como cualquier Bash.

**El único canal lateral sigue siendo la ventana propia del visor.** Ver
[research 05][r05] §2.

## 4. La opción nueva: skill invocable solo por `/`, peaje ~0

Frontmatter de un `SKILL.md`, del quickstart oficial:

```markdown
---
description: Greet the user with a friendly message
disable-model-invocation: true
---
```

`disable-model-invocation: true` lo deja **invocable solo por el usuario**. El
modelo no necesita saber que existe, así que **no paga peaje fijo**.

Eso da salida real a la tensión de los 738 tokens, y **no son excluyentes**: el
mismo plugin puede llevar el servidor MCP (para que el agente decida solo cuándo
dibujar) y un skill `/pizarra` (para quien no quiera pagar el peaje). La elección
queda en manos de quien paga los tokens. Va a [¿Servidor MCP o skill?][18].

El precio de la vía skill es el de siempre: sin schema no hay validación de
argumentos, y **un skill no tiene proceso propio ni ciclo de vida**, así que la
propiedad del estado de las N vistas (decisión de partida 6) queda sin dueño.

## 5. `SessionEnd`: lo efímero deja de ser un efecto colateral

La lista de eventos de hook incluye, literal:

| Evento | Cuándo dispara |
|---|---|
| `SessionStart` | *"When a session begins or resumes"* |
| `SessionEnd` | *"When a session terminates"* |

La decisión de partida 6 dice que *"el canvas muere con la sesión MCP, no con la
disciplina del agente"*, y hasta ahora eso dependía de que el proceso del servidor
muriese y arrastrase al visor. **`SessionEnd` es un mecanismo declarado para cerrar
la ventana**, y `SessionStart` para prepararla. Va a
[Cómo se abre y se cierra la ventana de terminal][17].

### Dónde va la salida de un hook

> "For most events, Claude Code writes stdout to the debug log and doesn't show it
> in the transcript. The exceptions are `UserPromptSubmit`,
> `UserPromptExpansion`, `SessionStart`, and `PostModelSwitch`, where Claude Code
> adds plain-text stdout as context that Claude can see and act on."

Es decir: **el stdout de la mayoría de hooks no entra en el contexto del modelo**
— va al debug log. Para hablarle al usuario hay `systemMessage` en la salida JSON,
y `terminalSequence` para notificación de escritorio, título de ventana o campana.

**Ojo con `SessionStart`**: es una de las cuatro excepciones, su stdout **sí** se
inyecta como contexto. Un hook de instalación de Python ahí debe callarse o pagará
tokens en cada arranque.

## 6. Python: hay patrón oficial, y menciona Python por su nombre

Sobre dependencias de Node, la instalación automática corre con `--ignore-scripts`:

> "**No lifecycle scripts:** `--ignore-scripts` keeps `preinstall`, `install`, and
> `postinstall` scripts from running."

Y para lo que eso no cubre:

> "For dependencies the automatic install can't provide, such as packages that
> need their lifecycle scripts to build, **Python dependencies**, or a plugin
> locked with Yarn or pnpm, install them from a hook into the persistent data
> directory."

El patrón es un hook `SessionStart` que instala en `${CLAUDE_PLUGIN_DATA}`. Va a
[Cómo se distribuye Python en un proyecto TypeScript][16].

**Letra pequeña de `bin/`:**

> "Executables added to the Bash tool's `PATH` and invokable as bare commands
> while the plugin is enabled. You can't include this directory in a plugin you
> distribute through claude.ai organization settings."

## 7. Instalación: bate a `npx`

§4 del handoff pone el listón — *"el usuario NO debería tener que configurar
manualmente una aplicación compleja"* — y §16 propone `npx ephemeral-visual-mcp`.

Un plugin da `/plugin install` desde marketplace, versionado por el campo
`version` del manifiesto, activable y desactivable con `/plugin`, y sin editar
JSON de configuración a mano. Marketplaces: `claude-plugins-official` (curado por
Anthropic), `claude-community` (envío con revisión), o **repositorio privado** para
uso interno. Hay `claude plugin validate ./plugin` para comprobar antes de enviar.

Va a [La experiencia de instalación][14], que casi se contesta con esto.

## Fuentes

- <https://code.claude.com/docs/en/plugins>
- <https://code.claude.com/docs/en/plugins-reference>
- <https://code.claude.com/docs/en/hooks>

[14]: https://github.com/javierponferradalopez/ai-render/issues/14
[16]: https://github.com/javierponferradalopez/ai-render/issues/16
[17]: https://github.com/javierponferradalopez/ai-render/issues/17
[18]: https://github.com/javierponferradalopez/ai-render/issues/18
[r05]: ./05-termaid-y-la-terminal.md
