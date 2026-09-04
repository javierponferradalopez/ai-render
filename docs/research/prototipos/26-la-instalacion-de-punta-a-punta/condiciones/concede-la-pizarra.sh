#!/usr/bin/env bash
# Hook `PreToolUse`: concede la pizarra, y nada más.
#
# Es el único mecanismo que se ha encontrado para que una herramienta MCP llegue
# a ejecutarse en modo `-p`: `--allowedTools` no las concede (nota de método del
# §8.1), y `permissions.allow` tampoco, ni en los settings del proyecto ni en un
# fichero pasado con `--settings`. El hook sí.
#
# Concede sólo lo que empareja el `matcher` de `condiciones/permisos-con-hook.json`,
# que son las dos herramientas de la pizarra. Las escrituras siguen denegadas.
printf '%s' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","permissionDecisionReason":"banco del prototipo 26"}}'
