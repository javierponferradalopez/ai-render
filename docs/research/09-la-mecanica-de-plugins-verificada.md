# La mecánica de plugins, verificada

Investigación disparada el 2026-09-01 para comprobar corriendo lo que
[research 06](./06-plugins-de-claude-code.md) documentó leyendo. Su nota de
método avisaba de que nada estaba verificado; la resolución de
[La experiencia de instalación][14] se apoyó en tres de esos huecos, y **uno
tumba el diseño**: la descarga del binario dentro del `command` del `.mcp.json`
no cabe en el plazo que Claude Code da a un servidor MCP para arrancar.

## Nota sobre el método

Todo lo que sigue está **ejecutado** en macOS 25.6 arm64 con Claude Code
**2.1.228**, salvo lo que se marca explícitamente como *leído del bundle*. El
banco de pruebas es un plugin de juguete cuyo lanzador registra lo que ve y tarda
lo que se le pida: [prototipo 15](./prototipos/15-plugin-de-juguete/). Un
servidor MCP stdio de 60 líneas, un skill invocable solo por `/`, y un
marketplace propio del que se instala de verdad.

Las mediciones de tokens salen del `usage` del propio Claude Code (`--output-format
stream-json`), sumando `input_tokens + cache_creation + cache_read` del primer
turno, desde un directorio vacío y con el mismo prompt. Cada condición se corrió
dos veces y dio el mismo número al token.

## 1. El plazo de arranque son 30 segundos, y el plugin no puede tocarlo

Medido, con el lanzador durmiendo antes de responder al handshake:

```
MCP server "plugin:toy:toy": Starting connection with timeout of 30000ms
MCP server "plugin:toy:toy": Connection timeout triggered after 30003ms (limit: 30000ms)
MCP server "plugin:toy:toy": Connection failed after 30005ms (CONNECT_TIMEOUT)
```

- Con retardo 0 conecta en **88 ms**; con 45 s corta a los **30,0 s** y manda
  **SIGTERM** al proceso **2,0 s después** del corte.
- El nombre interno de un servidor de plugin es **`plugin:<plugin>:<servidor>`**.
- **`MCP_TIMEOUT=90000` sí lo sube**: con esa variable, el mismo lanzador de 45 s
  conectó en 45 120 ms. También existe `MCP_CONNECT_TIMEOUT_MS` (*leído del
  bundle*, no probado).
- **Pero es del usuario, no del plugin.** Probado y descartado:
  - `settings.json` del plugin con `{"env": {"MCP_TIMEOUT": "90000"}}` → sigue
    arrancando con `timeout of 30000ms`.
  - `timeout`, `startupTimeout` e `initializationTimeout` en la entrada del
    servidor en `.mcp.json` → ignorados los tres (y `claude plugin validate` no
    se queja de ellos: solo valida el manifiesto).

Un plugin no puede pedir más plazo para sí mismo. 30 s es el techo, y el binario
universal de flipchart hay que traerlo por debajo de él **en cualquier red del
usuario**, o no traerlo ahí.

## 2. El hallazgo: un arranque fallido veta el servidor 15 minutos

No es que la sesión se quede sin pizarra. Es que **las siguientes tampoco la
intentan**. Claude Code apunta el fallo en `~/.claude/mcp-needs-auth-cache.json`:

```json
{"plugin:toy:toy": {"timestamp": 1788263727174, "id": "09fa9d4ad81f5fcb"}}
```

Prueba controlada, plugin instalado desde su marketplace, misma configuración en
los cinco pasos ("lanzado" = si el proceso del servidor llegó a arrancar):

| Paso | Situación | ¿Lanzado? | Cache |
|---|---|---|---|
| p1 | todo bien, cache limpio | **sí** | vacío |
| p2 | el `command` falla | sí | **entrada escrita** |
| p3 | fallo ya corregido | **no** | entrada intacta |
| p4 | tras un `claude mcp list` con éxito | **no** | entrada intacta |
| p5 | entrada con 16 minutos de antigüedad | **sí** | **borrada al conectar** |

Lo que se deduce, y encaja con el código (*leído del bundle*):

- El veto vive **15 minutos** (`nLS = 900000` ms; son 4 h —`oLS`— para conectores
  claude.ai y para servidores http/sse de plugin).
- Se aplica solo si el **fingerprint** de la configuración coincide: cambiar el
  `command` o pasar de `--plugin-dir` a instalado lo esquiva sin querer.
- **Solo afecta a los servidores stdio de un plugin.** La condición del bundle es
  literal: si el transporte es stdio y **no** hay `pluginSource`, nunca hay veto.
  Comprobado: el mismo lanzador roto, registrado con `--mcp-config` fuera de un
  plugin, se reintenta en cada sesión y no deja rastro en el cache. **El veto es
  un precio del empaquetado como plugin.**
- No lo cura reinstalar el plugin (`claude plugin install` deja la entrada). Un
  `claude mcp list` **sí reintenta** —ignora el veto y conecta— pero **no limpia**
  la entrada, así que la sesión siguiente vuelve a estar vetada. Lo único que la
  borra es una conexión con éxito dentro de una sesión, que es justo lo que el
  veto impide. En la práctica: **se espera, o se borra el fichero**.

Y esto vale para **cualquier** fallo, no solo para el timeout: el lanzador que
sale con código 1 al instante (64 ms, `-32000 Connection closed`) deja la misma
entrada. Una descarga que falla por estar sin red envenena los 15 minutos
siguientes.

### Lo que ve el usuario: dos palabras

En el TUI, dentro de `/mcp` (capturado en un pty):

```
Built-in MCPs (always available)
plugin:toy:toy · ✘ failed
※ Run claude --debug to see error logs
```

En la pantalla de bienvenida no apareció aviso alguno. El estado que sale por la
API headless es `{"name": "plugin:toy:toy", "status": "failed"}`. El único sitio
donde se lee la causa en texto plano es `claude mcp list`:

```
plugin:toy:toy: …/launcher.sh --root … - ✘ Failed to connect —
  MCP server "plugin:toy:toy" connection timed out after 30000ms
```

El **stderr del `command` sí se captura**, pero va al log de depuración, no a la
pantalla:

```
[ERROR] "MCP server \"plugin:toy:toy\" Server stderr: flipchart: no se pudo
        descargar el binario: curl: (6) Could not resolve host\n"
```

O sea: el mensaje de error que el producto escriba para el usuario **no llega al
usuario**. Solo `✘ failed`.

## 3. El arranque no bloquea el primer turno

Con el lanzador tardando 10 s —muy por debajo del plazo—, la sesión respondió en
8,9 s y el turno se ejecutó **con el skill y sin la herramienta**:

```
toy_slash=1  toy_tools=0
```

El servidor conectó después. No hay que tardar 30 s para quedarse sin pizarra:
basta tardar más que el usuario en escribir su primera frase. Cualquier trabajo
en el arranque —descargar, verificar, descomprimir— se paga en primeros turnos
sin herramientas, no solo en el riesgo de timeout.

## 4. `${CLAUDE_PLUGIN_DATA}` sí se expande en `.mcp.json`

El hueco de research 06 se cierra en positivo. El lanzador recibió:

```
argv=[--root /…/toy --data /Users/ponfe/.claude/plugins/data/toy-toy-market]
CLAUDE_PLUGIN_ROOT=[/…/toy]  CLAUDE_PLUGIN_DATA=[/Users/…/data/toy-toy-market]
```

Las dos variables llegan **como argumentos y como entorno**. El directorio se
**crea vacío** antes de arrancar, y su nombre es `<plugin>-<marketplace>` cuando
está instalado (`toy-toy-market`) o `<plugin>-inline` cuando se carga con
`--plugin-dir`. Es el sitio oficial para el binario descargado, sin inventarse un
`~/.cache/flipchart`.

## 5. `uninstall` borra los datos; el código queda huérfano

`claude plugin uninstall toy -y` **borró** `~/.claude/plugins/data/toy-toy-market/`
con el fichero de prueba que le había dejado dentro. El flag `--keep-data` existe
justo para evitarlo. Lo que **no** se borra es el código del plugin en
`~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`: se queda con un
fichero `.orphaned_at` y un timestamp, a expensas del barrido por antigüedad.

Para flipchart: el binario en `${CLAUDE_PLUGIN_DATA}` se va solo al desinstalar.
**El README no necesita una línea de `rm -rf`.**

## 6. El marketplace propio: formato, y la secuencia exacta

El fichero es `.claude-plugin/marketplace.json` en la raíz del repo:

```json
{
  "name": "toy-market",
  "description": "…",
  "owner": {"name": "…"},
  "plugins": [
    {"name": "toy", "source": "./plugins/toy", "description": "…"}
  ]
}
```

`claude plugin validate <ruta>` lo valida (y con `--strict` exige `description` y
`author`). En el marketplace oficial —291 plugins— `source` aparece de tres
formas: cadena con ruta relativa (53), `{"source": "url", …}` (153) y
`{"source": "git-subdir", "url", "path", "ref", "sha"}` (85), esta última para
plugins que viven en el repo de otro.

La secuencia del usuario, ejecutada:

```
claude plugin marketplace add <owner>/<repo>     # o una ruta local
claude plugin install <plugin>@<marketplace>
```

Detalles que importan al escribir el README de flipchart:

- `marketplace add owner/repo` **clona por SSH** (`Cloning via SSH:
  git@github.com:owner/repo.git`), con **timeout de 120 s**, a
  `~/.claude/plugins/marketplaces/<nombre>/`. Un repo privado exige, por tanto,
  SSH configurado en la máquina del usuario.
- El nombre del marketplace para el `install` **no es el del repo**: es el campo
  `name` del manifiesto. `anthropics/claude-code` se añade como
  `claude-code-plugins`.
- Un marketplace de tipo directorio **no copia nada**: apunta al directorio
  original, así que el `command` del servidor instalado sigue señalando allí. Útil
  para desarrollar; no es lo que verá un usuario.

## 7. El skill de peaje cero: cero de verdad

Medido, y el resultado es exacto:

| Condición | Entrada del primer turno |
|---|---|
| sin plugin | 27 814 |
| plugin con **solo** el skill (`disable-model-invocation: true`) | **27 814** |
| plugin con solo el servidor MCP (una herramienta) | 27 829 |

**El skill no cuesta un token**, y sin embargo está: aparece como `toy:check` en
los comandos de la sesión (78 frente a 77 sin él) y se invoca —probado—
escribiendo `/toy:check`, que devolvió lo que el `SKILL.md` mandaba. El
namespace es el del plugin, como la documentación prometía.

Contra eso, **`claude plugin details` proyecta un coste que no existe**:

```
Projected token cost
  Always-on:   ~29 tok   added to every session
  component  always-on  on-invoke
  check            ~30        ~20
```

Su inventario sí es útil, y sobre el servidor MCP es honesto —*"tool schemas
resolved at runtime; not counted"*—, pero su cifra de *always-on* para un skill
que el modelo no puede invocar es ruido. **La medición manda sobre la
proyección.**

## 8. La sorpresa: el peaje del servidor MCP depende del host, no del servidor

La misma herramienta de juguete cuesta distinto según el modo de búsqueda de
herramientas del host:

| `ENABLE_TOOL_SEARCH` | Sesión base | Con la herramienta | Coste |
|---|---|---|---|
| 1 (activo) | 27 814 | 27 829 | **+15 tok** |
| 0 | 87 852 | 87 921 | **+69 tok** |

Dos cosas, y las dos importan al presupuesto del mapa:

1. Con búsqueda de herramientas activa, las herramientas MCP entran **por el
   nombre** y su schema se carga a demanda. Una herramienta trivial pasa de 69 a
   15 tokens. El **peaje fijo de 738 tokens** que el mapa da por bueno se midió en
   el otro régimen; en este sería una fracción.
2. El prompt base **triplica** al desactivarla (27,8k → 87,9k). El coste del canal
   no lo decide el servidor: lo decide cómo esté configurado el host de cada
   usuario. Cualquier cifra de peaje que el spec prometa tiene que decir en qué
   régimen se midió.

## 9. La muerte llega por SIGINT, y la manda Claude Code

Anotado por el servidor de juguete en las cinco corridas donde la sesión terminó
bien:

```
MCP server "plugin:toy:toy": Sending SIGINT to MCP server process
MCP server "plugin:toy:toy": UNKNOWN connection closed after 24s (cleanly)
MCP server "plugin:toy:toy": MCP server process exited cleanly
```

[Transporte y ciclo de vida][13] cerró con *"muerte por EOF en stdin"*. Es cierto
que el EOF llega —el servidor de juguete lo vio en una corrida—, pero **lo
primero que llega es SIGINT**, y el proceso tiene que sobrevivir a él lo justo
para cerrar su ventana. Un binario que ignore SIGINT se queda huérfano; uno que
muera al instante sin cerrar el event loop deja la ventana colgada. Va a
[Transporte y ciclo de vida][13] como corrección de mecanismo, no de decisión.

## Qué se lleva cada ticket

- **La entrega del binario queda sin mecanismo viable tal como está escrita.** La
  descarga síncrona dentro del `command` que eligió [La experiencia de
  instalación][14] tiene 30 s duros, no ampliables por el plugin, y su fallo cuesta
  15 minutos de veto en el que ni se reintenta. Además, cualquier trabajo en el
  arranque se paga en primeros turnos sin herramientas (§3). Necesita decisión
  nueva; este documento no la toma.
- **[¿Servidor MCP o skill?][18]** gana dos datos duros: el skill de invocación
  manual cuesta **0 tokens**, medido; y el peaje del servidor MCP **depende del
  régimen de búsqueda de herramientas del host** (§7, §8).
- **[El presupuesto de tokens][10]**: cualquier cifra necesita decir en qué régimen
  se midió; el rango para una herramienta trivial va de 15 a 69 tokens.
- **[Transporte y ciclo de vida][13]**: la señal de muerte es SIGINT antes que EOF.
- Y lo que ya se puede dar por bueno del diseño de instalación:
  `${CLAUDE_PLUGIN_DATA}` funciona desde `.mcp.json` (§4), la desinstalación es
  limpia sin ayuda del README (§5), y la secuencia de marketplace propio es la que
  se suponía, con tres precisiones (§6).

## Fuentes

- Ejecución propia: [prototipo 15](./prototipos/15-plugin-de-juguete/).
- Cadenas y lógica leídas del binario de Claude Code 2.1.228
  (`/opt/homebrew/Caskroom/claude-code/2.1.228/claude`), marcadas como tal.
- <https://code.claude.com/docs/en/plugins-reference>

[10]: https://github.com/javierponferradalopez/ai-render/issues/10
[13]: https://github.com/javierponferradalopez/ai-render/issues/13
[14]: https://github.com/javierponferradalopez/ai-render/issues/14
[18]: https://github.com/javierponferradalopez/ai-render/issues/18
