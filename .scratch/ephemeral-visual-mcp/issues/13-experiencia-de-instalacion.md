> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/14 — copia congelada, no la edites.

# La experiencia de instalación

Type: grilling
Status: open
Blocked by: 06, 12

## Question

¿Qué escribe el usuario, una sola vez, para que su agente tenga pizarra?

§16 propone `npx ephemeral-visual-mcp`. Falta convertirlo en algo real:

- **Qué hace exactamente el comando.** ¿Arranca el servidor MCP en primer plano?
  ¿Solo registra la configuración y luego lo arranca Claude Code cuando lo
  necesita? Son modelos distintos: un servidor MCP por stdio lo lanza el cliente,
  no el usuario.
- **El registro en Claude Code.** ¿Se edita configuración a mano, se usa el
  comando que ofrezca la CLI, o el paquete lo hace por el usuario? ¿Ámbito de
  usuario o de proyecto?
- **Qué se distribuye.** El visor es una app web: ¿se publica compilada dentro
  del paquete npm y la sirve el propio servidor? Es lo que evita pedirle al
  usuario que instale nada más.
- **Requisitos previos** honestos: versión de Node, sistemas operativos.
- **Cómo se desinstala** y cómo se comprueba que funciona (un "hola mundo"
  visual que confirme que el circuito entero está vivo).
- **Nombre del paquete.** `ephemeral-visual-mcp` es el del handoff. ¿Se queda?

El listón lo pone §4: *el usuario NO debería tener que configurar manualmente una
aplicación compleja.*
