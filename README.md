# flipchart — una pizarra efímera para agentes

Un canal visual temporal para que un agente de IA se explique: cuando te cuenta una
estructura o un cambio de estructura, lo dibuja en una ventana nativa en vez de en ASCII o
en prosa. Guarda varias vistas y enseña una cada vez; muere con la sesión y no guarda nada.

Se instala como plugin de Claude Code, y es el único camino de instalación.

## Instalación

Dos líneas dentro de Claude Code:

```
/plugin marketplace add https://raw.githubusercontent.com/javierponferradalopez/ai-render/main/marketplace.json
/plugin install flipchart@flipchart
```

Y un tercer paso que **no es opcional**: pegar esta línea en tu `CLAUDE.md`.

```
Cuando me expliques una estructura o un cambio de estructura, dibújalo en la
pizarra con mcp__plugin_flipchart_flipchart__show en vez de en ASCII o en prosa.
```

Sin ella la pizarra queda instalada y no se usa jamás. Está medido, y no es un matiz: **0
de 36 turnos** con cuatro redacciones distintas del texto de la herramienta — por su cuenta
el agente no la ofrece, y lo que hace en su lugar es dibujar el grafo en ASCII dentro de la
respuesta. Los dos canales que sí disparan lo hacen al 100 %: esta línea, y que la nombres
tú. Que la nombres tú también pasa por aquí: si no lees esto, no sabes que existe.

Ese `mcp__plugin_flipchart_flipchart__show` es el nombre con el que Claude Code presenta la
herramienta cuando flipchart llega como plugin, y es feo por una razón que no controlamos:
el host compone el nombre del servidor como `plugin:<plugin>:<servidor>`. Si lo dejas en
`mcp__flipchart__show` estás nombrando una herramienta que no existe.

## Requisitos

- **macOS 11 o superior**, Intel o Apple Silicon. El número sale del propio binario: la
  mitad Apple Silicon declara `minos 11.0` y la Intel `10.12`, y por debajo de lo que
  declara no lo arranca el sistema. **Probado en macOS 26.6.2 arm64**, que es el único
  banco que hay; entre 11 y 26 no hay medición, sólo la declaración del build.
- Una versión de Claude Code con soporte de plugins.
- **Nada más**: no hay Node, ni Python, ni navegador, ni toolchain de Rust.

**Linux y Windows no son imposibles: son no probados y no prometidos.** Nadie ha corrido esto
ahí y no hay fecha, así que lo que sigue es lo que el diseño garantiza, no una medición. En
**Linux** el fallo no es mudo: el binario es un Mach-O que esa máquina no ejecuta, y el
lanzador —que es bash— se queda hablando él para decirlo. Aparece una sola herramienta,
`unavailable`, cuyo texto explica que aquí no hay pizarra y que no ofrezca diagramas. En
**Windows** no hay ni eso: el lanzador es un script de bash, así que no arranca nada y lo
único que se ve es un `✘ failed` en `/mcp`.

## Por qué sólo por `/plugin`

Porque es el camino que está medido, y el único que llega a una máquina utilizable:

- Claude Code descarga el zip, **comprueba su `sha256`** y **rechaza la instalación** si no
  casa, con el error delante.
- El binario que extrae llega **sin `com.apple.quarantine`**, así que su ejecución no pasa
  por Gatekeeper. Un fichero traído por el navegador o por Mail **sí** lleva ese atributo, y
  es el caso que Gatekeeper mata.
- No se ejecuta `git` ni una vez, y lo que queda en disco se poda solo.

## Actualizar y desinstalar

```
/plugin update flipchart@flipchart
/plugin uninstall flipchart@flipchart
```

**El `@flipchart` del final no es opcional en el `update`**: con el nombre corto contesta
`Plugin "flipchart" not found`, aunque esté instalado y `/plugin` lo liste. Lo de después del
`@` es el marketplace, y se llama igual que el plugin. Si prefieres no teclear nombres,
`/plugin` abre el menú y hace lo mismo.

`uninstall` se lleva también los datos del plugin, así que no hay ningún `rm -rf` que
teclear. Para desactivarla sin desinstalarla, `/plugin`.

Dos avisos de disco, los dos medidos: entre una actualización y la poda automática la caché
guarda **las dos versiones** del binario, no una —47 MB pasan a 94—; y tras desinstalar, el
binario **sigue en disco** —marcado como huérfano— hasta que esa misma poda pase. Son unos
49 MB por versión.

Y una regla que es nuestra, no tuya, pero explica lo que verías si la rompiéramos: `update`
sólo trae algo si la versión ha subido. Un arreglo publicado sin subirla se descarga entero,
se tira, y `update` contesta `already at the latest version`.

## Lo que ya sabemos que molesta

Nada de esto es un fallo abierto: es lo que la pizarra hace hoy, escrito para que no lo
descubras chocándote.

- **La ventana no roba el teclado, y ése es el precio.** Aparece delante y el foco se queda
  donde lo tenías, así que ⌘W no le llega hasta que la cliques — el de reflejo se lo come el
  terminal. Y en cuanto vuelves al terminal la pizarra pasa detrás: **el siguiente dibujo no
  la trae de vuelta**, la clicas tú.
- **Pasar hojas es a ciegas.** Una hoja a la vista, las flechas `‹ ›` de la cabecera y
  «hoja N de M». No hay índice de las demás: quien pasa la hoja es el agente, y tú
  retrocedes.
- **Un `/clear` no borra la pizarra.** La conversación acaba, la sesión no, así que la
  ventana se queda enseñando el diagrama de antes.
- **Un diagrama patológico puede colgar el turno.** No hay tope de tiempo ni barrera de
  tamaño: si el cálculo del layout tarda un minuto, no se puede abortar.
- **Lo que se promete dibujar es el grafo dirigido** — flujos, dependencias, arquitectura y
  diagramas de clases. Las demás familias de Mermaid no están prohibidas, están no probadas,
  y el agente las elige por su cuenta.
- **La pizarra decide cómo se ve.** Colores, formas y dirección son suyos: si el agente
  escribe estilo, se le tira y se le avisa. Los grupos y las cajas se dibujan como los
  dibuje el motor, sin plan B.
- **El marcado dentro de una etiqueta sale literal.** Sólo `<br>` y `<br/>` parten la línea;
  un `<b>` acaba con los picos puestos en la caja. La pizarra se lo avisa al agente en las
  etiquetas de nodos, grupos y aristas — pero un `Note over` de un diagrama de secuencia se
  dibuja literal y callando.

## Desarrollo

Las decisiones del producto están en [`DECISIONS.md`](./DECISIONS.md), el idioma del
dominio en [`CONTEXT.md`](./CONTEXT.md), y lo medido en [`docs/research/`](./docs/research/).
Cómo se construye y cuál es la puerta, en [`CLAUDE.md`](./CLAUDE.md).
