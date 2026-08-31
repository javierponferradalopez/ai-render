# Semántica de `visual.update`

Type: grilling
Status: open
Blocked by: 05

## Question

¿Qué es exactamente un patch incremental, y qué pasa cuando no encaja?

§11 del handoff quiere que el agente modifique sin recrear, y §22.14 hace del
coste en tokens una preocupación de primer nivel. Pero un protocolo de patches
tiene esquinas afiladas que hay que decidir antes de escribir código:

- **Las operaciones.** ¿`add` / `remove` / `update` como propone §7, o hace falta
  más — mover un nodo de grupo (el caso "muevo la carpeta"), renombrar,
  reconectar una arista? El caso de uso protagonista es un refactor: mover cosas
  de sitio es *la* operación, y "borrar y volver a añadir" pierde la identidad
  del nodo.
- **Referencias.** El patch nombra ids. ¿Qué pasa si el id no existe — error,
  silencio, o creación implícita?
- **Atomicidad.** Si una operación del patch falla, ¿se aplica el resto o se
  rechaza entero?
- **Qué se le devuelve al agente.** Un patch que falla en silencio deja al agente
  describiendo un diagrama que el usuario no está viendo. ¿Confirmación? ¿Estado
  resultante? Ojo: la respuesta también cuesta tokens.
- **Cuándo NO usar update.** Si el cambio es grande, reenviar el documento
  entero puede ser más barato en tokens *y* en razonamiento. ¿Hay una regla que
  se le pueda dar al agente?
- **Deriva.** Servidor y visor deben coincidir. ¿Cómo se detecta y corrige que no
  lo hagan?
