# The first quoted value after a key, in bare bash.
#
# Good for `plugin.json` and only for it: a versioned file we write ourselves,
# one key per line and without a `"` or a `\` inside any value. `catalog.sh`
# checks that premise before leaning on it, and `jq` is absent here for the same
# reason it is absent from the Launcher (ADR-0014).

field() {
  local rest=${2#*\"$1\"}
  [ "$rest" = "$2" ] && return 1
  rest=${rest#*:}
  rest=${rest#*\"}
  printf '%s' "${rest%%\"*}"
}
