#!/bin/bash
# The Launcher — and, when there is no Flipchart process to hand its place over
# to, the Unavailable server (docs/adr/0014-the-launcher-never-fails.md).
#
# The constraint is first-class: it never fails. It always answers the
# handshake, in milliseconds, binary or no binary, and exits with 0. The reason
# is not the host's 30 s deadline, which is a probability: it is that a failed
# start bans the server for 15 minutes, reinstalling the plugin does not cure
# it, and all the user sees are two words —✘ failed— with our stderr buried in
# the debug log.
#
# Hence what is not here: no `set -e`, no `jq`, no `python3` —which on a machine
# without Xcode does not exist—, no `perl`, no `ruby`, not one line of network.
# Bare Bash 3.2, the one macOS ships.

trap 'exit 0' INT TERM HUP

readonly BINARY="${CLAUDE_PLUGIN_ROOT:-$(dirname "$0")}/flipchart"

readonly MISSING='the flipchart binary is not in the plugin directory'
readonly UNRUNNABLE='the flipchart binary could not be given execute permission'
readonly FOREIGN='this machine refused to execute the flipchart binary, which is a macOS build - another platform or architecture cannot run it'

# A backstop, not the mechanism: the host preserves the 0755 from the Info-ZIP
# zip, but nobody promises it in its schema.
chmod +x "$BINARY" 2>/dev/null

if [ ! -e "$BINARY" ]; then
  DIAGNOSIS=$MISSING
elif [ ! -x "$BINARY" ]; then
  DIAGNOSIS=$UNRUNNABLE
else
  # `execfail` is the other half of the promise: without it, an `exec` that
  # fails —a Mach-O of another architecture, Gatekeeper— kills the script, and
  # with it the only voice that could have told anyone.
  shopt -s execfail
  exec "$BINARY" "$@"
  DIAGNOSIS=$FOREIGN
fi

# ── The Unavailable server ────────────────────────────────────────────────────
#
# The JSON is built by hand, so no text here carries a `"` or a `\`.

# The `command`'s stderr ends up in the host's debug log, so no user reads this
# line: it is for whoever looks at the log after them.
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

# The first `"id":` on the line is always the JSON-RPC one: with a single tool
# that takes no arguments, no message this loop ever parses carries a nested
# `"id"`. The fragility is not mitigated, it is removed.
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

# We answer with whatever version the client speaks: this loop depends on
# nothing a protocol revision could move, and staying anchored to an old version
# would be the way to fail the handshake three releases from now.
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
    # The protocol's `ping` is not asked for by the box, it is asked for by the
    # session: an unanswered request is a connection the host can give up for
    # dead, and with it would go the only warning the user was going to get.
    ping) answer "$(the_id_in "$line")" '{}' ;;
  esac
done

exit 0
