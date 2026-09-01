# Prototipo 15 — un plugin de juguete, para medir la mecánica

Banco de pruebas de [research 09](../../09-la-mecanica-de-plugins-verificada.md).
Un plugin mínimo de Claude Code —un servidor MCP por stdio, un skill invocable
solo por `/`— cuyo lanzador **registra lo que ve y tarda lo que le mandes**. Con
eso se mide el timeout de arranque, la expansión de variables, el peaje en tokens
y el veto que deja un arranque fallido.

Medido en macOS 25.6 arm64 con Claude Code **2.1.228**.

## Las piezas

| Fichero | Para qué |
|---|---|
| `toy/` | El plugin: manifiesto, `.mcp.json`, lanzador, servidor MCP y el skill `check` |
| `toy/launcher.sh` | Lo que arranca Claude Code. Registra timestamps, argv y entorno en `$TOY_LOG`; duerme `$TOY_DELAY` segundos; sale con error si `$TOY_FAIL` |
| `toy/mcp_server.py` | Servidor MCP stdio de 60 líneas: `initialize`, `tools/list`, `tools/call`, y anota cada señal que recibe |
| `market/.claude-plugin/marketplace.json` | El marketplace propio desde el que se instala |
| `run.py` | Lanza un comando con timeout (macOS no trae `timeout`) |
| `hold.py` | Sesión `stream-json` viva N segundos **sin llamar al API**: arranca los MCP y no gasta tokens |
| `converse.py` | Igual, pero manda un mensaje de usuario para provocar el `system/init` |
| `usage.py`, `showinit.py` | Extraen del stream el inventario (tools, slash commands) y el tamaño de entrada |
| `veto_probe.sh` | Un paso de la prueba del veto: corre una sesión y dice si el proceso del servidor se lanzó |
| `tty_probe.py` | Arranca el TUI en un pty, teclea, y guarda la pantalla — para leer lo que ve el usuario |
| `hold_run.sh`, `sweep.sh` | Envoltorios del barrido de retardos |

## Cómo se corre

```sh
LAB=$PWD
export TOY_LOG=$LAB/toy.log TOY_DELAY=0

# 1. Validar y cargar el plugin sin instalarlo
claude plugin validate ./toy --strict
python3 hold.py 25 /tmp/a claude -p --input-format stream-json \
  --output-format stream-json --verbose --plugin-dir ./toy --debug-file /tmp/a-debug.txt
grep 'plugin:toy:toy' /tmp/a-debug.txt      # -> "timeout of 30000ms"

# 2. Provocar el timeout: el lanzador tarda más que el límite
TOY_DELAY=45 python3 hold.py 60 /tmp/b claude -p --input-format stream-json \
  --output-format stream-json --verbose --plugin-dir ./toy --debug-file /tmp/b-debug.txt
grep 'CONNECT_TIMEOUT' /tmp/b-debug.txt

# 3. Ver el veto: la sesión siguiente ni lanza el proceso
cat ~/.claude/mcp-needs-auth-cache.json    # aparece plugin:toy:toy
TOY_DELAY=0 sh veto_probe.sh $LAB p3        # -> lanzado=NO

# 4. Instalar de verdad, desde el marketplace propio
claude plugin marketplace add ./market && claude plugin install toy@toy-market
claude plugin details toy                   # inventario y coste proyectado
claude mcp list | grep toy                  # health check, texto plano
claude plugin uninstall toy -y              # borra ~/.claude/plugins/data/<id>/
```

Para medir el peaje se duplica `toy/` en dos variantes —una sin `.mcp.json`, otra
sin `skills/`— y se compara el total de entrada del primer turno con el de una
sesión sin plugin, con `ENABLE_TOOL_SEARCH` a 1 y a 0.

## Avisos

- El plugin escribe en `$TOY_LOG`; si no la exportas, va a `/tmp/toy-plugin.log`.
- Las pruebas dejan entradas en `~/.claude/mcp-needs-auth-cache.json`. Borrar la
  clave `plugin:toy:toy` es lo que devuelve el banco a cero — o esperar 15 minutos.
- `hold.py` no llama al API; `converse.py` sí (un turno de Haiku por corrida).
