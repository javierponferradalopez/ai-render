# El primer valor entre comillas detrás de una clave, en bash pelado.
#
# Vale para el `plugin.json` y sólo para él: es un fichero versionado que
# escribimos nosotros, con una clave por línea y sin un `"` ni un `\` dentro de
# ningún valor. `catalogo.sh` comprueba esa premisa antes de apoyarse en ella,
# y `jq` no está aquí por la misma razón que no está en el Lanzador (ADR-0014).

campo() {
  local resto=${2#*\"$1\"}
  [ "$resto" = "$2" ] && return 1
  resto=${resto#*:}
  resto=${resto#*\"}
  printf '%s' "${resto%%\"*}"
}
