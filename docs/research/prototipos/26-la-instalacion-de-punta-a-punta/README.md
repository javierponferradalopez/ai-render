# Prototipo 26 — la instalación de punta a punta, y la línea que la remata

Banco de [research 20](../../20-la-instalacion-y-la-linea-verificadas.md), del ticket
[#45](https://github.com/javierponferradalopez/ai-render/issues/45). Dos preguntas que sólo
se contestan con el plugin de verdad delante:

1. **¿Con qué nombre presenta el host la herramienta?** De eso depende la línea del §8.2,
   que es el disparador principal del producto — y la respuesta no era la que la decisión
   daba por supuesta.
2. **¿Dibuja el agente sin que nadie se lo pida, con la línea pegada?** El prototipo 22 lo
   midió contra un servidor falso y con la línea en su primera redacción. Esto lo repite con
   el release `v0.1.0` instalado y con la redacción que va al README.

Y una tercera, que llegó publicando: **¿trae `/plugin update` la versión nueva?** La mecánica
estaba medida con un plugin sonda (research 15 §5); lo que faltaba era el primer `v0.1.1` de
flipchart, y con él está `mide-el-update.sh`.

No hay aquí nada del vehículo del zip: eso es el [prototipo 17](../17-el-vehiculo-del-zip/) y
research 19 §5.

## Las piezas

| Fichero | Qué es |
|---|---|
| `nombre-de-la-herramienta.sh` | Un turno que responde «ok» y del que sólo se lee el evento `init`: el nombre interno del servidor y los nombres de herramienta que ve el modelo. |
| `corre-con-la-linea.sh` | El escenario `refactor` del prototipo 22 —tres turnos, y el usuario no pide dibujo— contra el plugin del release. Con `--control-positivo`, el turno que pide el dibujo a la cara. |
| `condiciones/CLAUDE.md.con-la-linea` | Lo único que se le da al sujeto: la línea del §8.2, con el nombre de herramienta ya medido. |
| `condiciones/concede-la-pizarra.sh` | El hook `PreToolUse` que concede la pizarra y nada más — el único mecanismo que la deja ejecutarse en modo `-p`. |
| `mide-el-update.sh` | El `update` del plugin de verdad, de la versión vieja a la nueva: el catálogo, el desenlace, la caché, el disco y el handshake del binario nuevo. |
| `destila.py` | Del stream de cada turno se queda con lo que dice algo: la pizarra en la lista de herramientas, las llamadas, lo que contestaron y la respuesta del agente. |
| `registros/` | Las cuatro corridas: `r1-sin-permiso/`, `r2-con-permiso/`, `r3-control-positivo/` y `r4-el-update/`. |

## El banco

macOS 26.6.2 (build 25G83) arm64, Claude Code **2.1.228**, release **v0.1.0** instalado en
un `CLAUDE_CONFIG_DIR` propio desde el catálogo de `main` por
`raw.githubusercontent.com` — el camino del usuario, sin atajos.

**La caja que se pone a prueba es la que dejó el host**, extraída en
`plugins/cache/flipchart/flipchart/0.1.0/`, y las corridas la cargan con `--plugin-dir` en
vez de con el plugin instalado. El motivo es prosaico: el `CLAUDE_CONFIG_DIR` aislado no
tiene sesión, y un turno sin sesión no es un turno. Con `--plugin-dir` la sesión es la del
usuario y el binario sigue siendo el del release. Lo que se paga por ello está escrito en la
nota de método de research 20: el banco lleva dentro los conectores de la máquina, incluido
uno que dibuja diagramas.

**El sujeto es siempre una copia**, en el scratchpad de la sesión: `pickypen.nvim`, los
mismos cuatro módulos Lua del prototipo 22, sin su `AGENTS.md` ni su `CLAUDE.md` —que traen
la arquitectura ya escrita, que es justo lo que la pizarra viene a dibujar—. Y el arnés va
con `--permission-mode default` y `--disallowedTools Edit Write NotebookEdit`: el turno 2 del
guion es un «Sí, adelante», y con `acceptEdits` el agente hace el refactor de verdad.

## Las tres corridas, y el permiso que costó dos de ellas

Conceder una herramienta MCP en modo `-p` no es lo que uno supone, y esto lo midió a base de
corridas perdidas:

| Cómo se intenta conceder | Resultado |
|---|---|
| `--allowedTools mcp__plugin_flipchart_flipchart__show` | **no la concede** (ya lo decía la nota de método del §8.1) |
| `permissions.allow` en `<sujeto>/.claude/settings.json` | **no se aplica**: los settings del proyecto piden haber confiado en el directorio |
| `permissions.allow` en un fichero pasado con `--settings` | **no la concede** tampoco |
| un hook `PreToolUse` que contesta `permissionDecision: allow` | **la concede** |

Y `acceptEdits` o `bypassPermissions`, que la concederían de paso, están descartados: el turno
2 del guion es un «Sí, adelante» y los dos auto-aprueban las ediciones al margen de cualquier
lista.

| Corrida | Permiso | Qué mide |
|---|---|---|
| `r1-sin-permiso` | ninguno | **La conducta**, tres turnos. El agente llama, el host deniega, y el `tool_use` con su diagrama dentro queda en el registro. |
| `r2-con-permiso` | `permissions.allow` por `--settings` | La conducta otra vez, y **la prueba de que ese camino no concede**: dos llamadas, las dos denegadas. |
| `r3-control-positivo` | el hook | **La tubería.** La llamada llega al Servidor MCP del release, la ventana aparece y el acuse vuelve con los avisos del §4.4 dentro. |
