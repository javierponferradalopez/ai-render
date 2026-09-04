#!/bin/bash
# El Lanzador — y, cuando no hay Proceso de la pizarra al que cederle el sitio,
# el Servidor de aviso (DECISIONS §10.4, §10.5).
#
# La restricción es de primer nivel: nunca falla. Contesta al handshake siempre,
# en milisegundos, haya binario o no, y sale con 0. El motivo no es el plazo de
# 30 s del host, que es una probabilidad: es que un arranque fallido veta el
# servidor 15 minutos, no lo cura reinstalar el plugin, y lo único que el
# usuario ve son dos palabras —✘ failed— con nuestro stderr enterrado en el log
# de depuración.
#
# De ahí lo que no hay aquí: ni `set -e`, ni `jq`, ni `python3` —que en una
# máquina sin Xcode no existe—, ni `perl`, ni `ruby`, ni una línea de red.
# Bash 3.2 pelado, el que trae macOS.

trap 'exit 0' INT TERM HUP

readonly BINARY="${CLAUDE_PLUGIN_ROOT:-$(dirname "$0")}/flipchart"

readonly MISSING='the flipchart binary is not in the plugin directory'
readonly UNRUNNABLE='the flipchart binary could not be given execute permission'
readonly FOREIGN='this machine refused to execute the flipchart binary, which is a macOS build - another platform or architecture cannot run it'

# Respaldo, no mecanismo: el host preserva el 0755 del zip de Info-ZIP, pero
# nadie lo promete en su esquema.
chmod +x "$BINARY" 2>/dev/null

if [ ! -e "$BINARY" ]; then
  DIAGNOSIS=$MISSING
elif [ ! -x "$BINARY" ]; then
  DIAGNOSIS=$UNRUNNABLE
else
  # `execfail` es la otra mitad de la promesa: sin él, un `exec` que falla
  # —un Mach-O de otra arquitectura, Gatekeeper— mata el script, y con él la
  # única voz que podía contarlo.
  shopt -s execfail
  exec "$BINARY" "$@"
  DIAGNOSIS=$FOREIGN
fi

# ── El Servidor de aviso ──────────────────────────────────────────────────────
#
# El JSON se construye a mano, así que ningún texto de aquí lleva `"` ni `\`.

# El stderr del `command` acaba en el log de depuración del host, así que esta
# línea no la lee ningún usuario: es para quien mire el log detrás de él.
printf 'flipchart: %s\n' "$DIAGNOSIS" >&2

readonly TOOL=unavailable
readonly MESSAGE="The flipchart is not available in this session and cannot draw anything: ${DIAGNOSIS}. Nothing will appear on screen, so do not offer the user a diagram - explain in prose instead. Reinstalling the plugin is what brings it back."
readonly LAST_KNOWN_PROTOCOL=2025-06-18

string_field() {
  local rest=${2#*\"$1\"}
  [ "$rest" = "$2" ] && return 1
  rest=${rest#*\"}
  printf '%s' "${rest%%\"*}"
}

# El primer `"id":` de la línea es siempre el de JSON-RPC: con una sola
# herramienta sin argumentos, ningún mensaje que este bucle llegue a parsear
# trae un `"id"` anidado. La fragilidad no se mitiga, se elimina.
the_id_in() {
  local rest=${1#*\"id\"}
  [ "$rest" = "$1" ] && return 1
  rest=${rest#*:}
  while [ "${rest# }" != "$rest" ]; do rest=${rest# }; done
  case $rest in
    '"'*)
      rest=${rest#\"}
      printf '"%s"' "${rest%%\"*}"
      ;;
    *)
      rest=${rest%%,*}
      rest=${rest%%\}*}
      printf '%s' "${rest%% *}"
      ;;
  esac
}

answer() {
  printf '{"jsonrpc":"2.0","id":%s,"result":%s}\n' "$1" "$2"
}

# Se devuelve la versión que el cliente hable: este bucle no depende de nada que
# una revisión del protocolo pueda mover, y quedarse anclado a una versión vieja
# sería la forma de fallar el handshake dentro de tres releases.
greeting() {
  printf '{"protocolVersion":"%s","capabilities":{"tools":{}},"serverInfo":{"name":"flipchart","version":"unavailable"}}' \
    "$(string_field protocolVersion "$1" || printf '%s' "$LAST_KNOWN_PROTOCOL")"
}

the_only_tool() {
  printf '{"tools":[{"name":"%s","description":"%s","inputSchema":{"type":"object","properties":{}}}]}' \
    "$TOOL" "$MESSAGE"
}

the_message_as_a_result() {
  printf '{"content":[{"type":"text","text":"%s"}],"isError":true}' "$MESSAGE"
}

while IFS= read -r line || [ -n "$line" ]; do
  case "$(string_field method "$line")" in
    initialize) answer "$(the_id_in "$line")" "$(greeting "$line")" ;;
    tools/list) answer "$(the_id_in "$line")" "$(the_only_tool)" ;;
    tools/call) answer "$(the_id_in "$line")" "$(the_message_as_a_result)" ;;
    # El `ping` del protocolo no lo pide la caja, lo pide la sesión: una
    # petición sin contestar es una conexión que el host puede dar por muerta, y
    # con ella se iría el único aviso que el usuario iba a recibir.
    ping) answer "$(the_id_in "$line")" '{}' ;;
  esac
done

exit 0
