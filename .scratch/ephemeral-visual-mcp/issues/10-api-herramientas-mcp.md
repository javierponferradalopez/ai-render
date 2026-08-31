> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/11 — copia congelada, no la edites.

# La API de las tres herramientas MCP

Type: grilling
Status: open
Blocked by: 05, 08, 09

## Question

¿Cuál es la firma exacta de `visual.show`, `visual.update` y `visual.clear`?

Es donde el agente toca el sistema, y donde se paga el peaje de tokens en cada
conversación.

A decidir:

- **Nombres.** ¿`visual.show` o algo que se lea mejor desde el punto de vista del
  agente? El nombre es literalmente lo primero que lee.
- **Parámetros de cada una**, con el contrato de
  [El contrato del VisualDocument](./05-contrato-visual-document.md) dentro.
- **Qué devuelven.** Confirmación mínima, estado, errores. Cada byte cuesta.
- **Las descripciones de las herramientas.** No son documentación: son el prompt
  que decide si el agente usa la pizarra bien, de más o nunca. Hay que
  escribirlas de verdad, no dejarlas para el final, y cabiendo en el presupuesto.
- **`visual.clear` con vistas.** Si un canvas tiene N vistas (decisión de partida
  6), ¿`clear` las borra todas o acepta un `id`?
- **Errores.** Qué ve el agente si el visor no está conectado, si el documento no
  valida, si el layout falla.
