# El HTML en las etiquetas, decidido con el dibujo delante

Decisión disparada el 2026-09-04 por [El HTML en las etiquetas: decidir con el
dibujo delante][42], que es el riesgo 3 del §11.2: la única del spec que se dejó
sin tomar a propósito, porque se toma **mirando el dibujo**.

**No son dos casos, son uno con una excepción — y la excepción es la frecuente.**
Barridos treinta casos por la tubería de verdad y mirados los píxeles, mmdr
interpreta **exactamente dos cadenas** dentro de una etiqueta —`<br>` y `<br/>`—
y dibuja como texto literal **todo lo demás que no sea texto**: el resto de
etiquetas HTML, las entidades `&…;`, los escapes `#…;` de Mermaid, y también
`<br />` con un espacio dentro. Así que no hay una familia de "HTML" contra un
caso especial: hay una lista blanca de dos y una frontera que no la puso una
política nuestra sino el tokenizador de mmdr.

**El desenlace es aviso, y por constructo:** `<br>` conviven **sin aviso** —hacen
lo que el agente quería, cobrarles sería cobrar por algo que funciona— y todo lo
demás enciende el **cuarto aviso del §4.4**, con la Vista dibujada. Ni rechazo
—cobraría la estructura entera por un defecto de texto, y el agente que tropieza
no insiste: se pasa a prosa— ni convivencia —dejaría basura permanente que **nadie
más puede corregir**, porque el agente es ciego y el usuario no lee el fuente—.

## Nota sobre el método

El criterio del ticket era ver los constructos **dibujados, no descritos**, y eso
manda el instrumento. Tres piezas, todas en el [prototipo 24][p24]:

- **`casos/`** — 30 casos, 21 de `flowchart` y 8 de `classDiagram` más el
  espontáneo real que abre el ticket. El marcado se pone en los cuatro sitios
  donde hay texto en las dos familias que se prometen: etiqueta de nodo, etiqueta
  de arista, etiqueta de grupo y miembro de una clase.
- **`svg/`** — el SVG y el PNG de cada caso. El SVG sale del `bench` del
  [prototipo 21][p21] por el camino permisivo (`--permisivo`), que es la tubería
  de hoy; el PNG lo saca `pixeles/`, que es `usvg` con las fuentes del sistema y
  `resvg` sobre un `Pixmap` — **el mismo código que `src/raster.rs`**, así que lo
  que se mira son los píxeles que el Visor sube a la textura.
- **`mira.py`** — la Pizarra de verdad: habla con el binario por stdio como haría
  el host, muestra el caso y fotografía la pantalla. Es el camino corto a "en la
  ventana" **cuando hay permiso de grabación de pantalla**; sin él `screencapture`
  contesta `could not create image from display` y el careo se hace con los PNG.

Y el desenlace de cada caso se lee con `flipchart check`, la tubería entera sin
abrir ventana, que es lo que dice si el aviso salta.

Una advertencia sobre el banco, porque cuesta media hora descubrirla: en
`classDiagram`, **mmdr no entiende `class Pedido["Pedido<br/>agregado"]`**. No es
que pierda el marcado: fabrica un nodo cuyo id es la línea entera, y el `Límite
honesto` lo rechaza como apócrifo antes de dibujar nada. Es un defecto suyo, ya
apuntado en el banco del prototipo 21 (`cd-13-etiqueta-clase`), y no tiene nada
que ver con esta pregunta — así que los casos de `classDiagram` de aquí ponen el
marcado donde `classDiagram` sí lo lleva: la etiqueta de la relación y el miembro.

## 1. Lo que se ve, mirando el dibujo

El caso espontáneo del ticket, dibujado, pone los dos comportamientos en la misma
imagen. El dibujo es `svg/mix-01-espontaneo.png`; esto es su transcripción:

```
┌─────────────────┐      ┌───────────────────────────┐
│    store.lua    │ ───► │         marks.lua         │
│   persistencia  │      │  <b>recolocacion</b>      │
└─────────────────┘      └───────────────────────────┘
```

`<br/>` parte la etiqueta en dos líneas: exactamente lo que el agente quería.
`<b>` sale con los picos puestos, en la caja, para siempre.

## 2. El barrido: qué más viaja dentro de la etiqueta

| Constructo | Qué llega al dibujo | ¿Es lo que el agente quería? |
|---|---|---|
| `<br>`, `<br/>` | salto de línea | **sí** |
| `<br />`, `<br  />`, `<BR/>` | `<br />` literal | no |
| `<b>`, `<i>`, `<em>`, `<strong>`, `<u>`, `<code>` | la etiqueta literal | no |
| `<span style=…>`, `<a href=…>`, `<img src=…>` | la etiqueta literal, atributos incluidos | no |
| `&amp;`, `&nbsp;`, `&lt;`, `&gt;` | la entidad literal | no |
| `&#35;` | la entidad literal | no |
| `#quot;`, `#35;` (escapes de Mermaid) | el escape literal | no |
| `&`, `<`, `'` crudos | el carácter, bien escapado en el SVG | **sí** |
| `<<interface>>` | la anotación, bien dibujada | **sí** |

Dos cosas que este barrido cambia respecto a lo que el §11.2 daba por supuesto:

1. **El corte no es "HTML sí / HTML no", es una lista blanca de dos cadenas.**
   `<br />` con un espacio ya sale literal, y `<BR/>` también. Quien escriba la
   regla por el nombre de la etiqueta —"`br` se interpreta"— se equivoca en tres
   de cada cinco formas de escribir un `br`.
2. **Las entidades están en el mismo saco, y nadie las había contado.** `&amp;`
   no llega como `&`: llega como `&amp;`. Y los escapes propios de Mermaid
   (`#quot;`) tampoco, aunque sean del idioma y no HTML.

Los cuatro sitios se portan igual en las dos familias: etiqueta de nodo, de
arista, de grupo y miembro de clase, en `flowchart` y en `classDiagram`. **No hay
que decidir por sitio, sólo por constructo.**

## 3. Los números de la balanza

Sobre los 17 diagramas espontáneos del [prototipo 22][p22], que es material real:

| | |
|---|---|
| Con `<br>` — que sale bien | **15 / 17** |
| Con basura visible hoy | **5 / 17** (`<b>` en tres, `<i>` en uno, `&lt;`/`&gt;` en uno) |

El 15/17 del ticket es la frecuencia del constructo **que funciona**. Lo que un
desenlace cobra es el 5/17, y eso mueve la balanza: rechazar habría tirado casi
uno de cada tres diagramas espontáneos por una palabra fea.

## 4. Por qué aviso, y no las otras dos

**Contra el rechazo.** El §4.3 tiene cinco desenlaces y el ticket pedía que un
rechazo cupiera en ellos sin ser un sexto; no cabe — no es entrada inválida, ni
`ParseError`, ni invento, ni pánico. Pero el argumento que decide no es de
taxonomía: es que **cobra la estructura entera por un defecto de texto**. Tirar un
diagrama de dependencias bien escrito porque una caja dice `<b>` cambia *ver una
palabra fea* por *no ver nada*, y el listón que el §11.2 puso para el riesgo 2
vale igual aquí: ante un tropiezo el agente **no insiste, se pasa a prosa y no lo
dice**. El reparto del §4 aguanta: esto es ver de más **una palabra**, no un nodo
— el usuario no se cree
`<b>` como contenido, y ninguna relación del dibujo miente.

**Contra la convivencia.** El aviso es el **único canal que existe** para
arreglarlo. Nadie más puede: el agente no ve el dibujo y el usuario no ve el
fuente. Con el aviso, basura permanente pasa a ser basura de un turno.

**El aviso, y su precio.** Literal fijo, acumulable, `isError: false`, con la
forma de los otros tres del §4.4. **39 tokens** contados con `peaje-30.py`
(cl100k_base), el más caro de los cuatro —el de estilo son 34, la dirección 25,
`namespace` 18—, y se paga sólo cuando el constructo vino: 5 de 17.

```
Note: only <br> is rendered inside labels; other tags, HTML entities and #-escapes reached
the drawing as literal text. The view was drawn — write those labels as plain text.
```

Nombra `<br>` a propósito: es la mitad útil del mensaje, porque el 15/17 dice que
es el constructo que el agente ya usa y hay que confirmárselo, no prohibírselo.

## 5. Cómo se pregunta, y qué no cuenta

Se le pregunta a **las etiquetas del `Graph`**, no al fuente —igual que las reglas
del §4.1—, y eso no es un detalle: en el fuente el `&` de `A & B` parece una
entidad y `-->` parece cualquier cosa; en la etiqueta ya no queda sintaxis de
Mermaid con la que confundirse. Cuentan como marcado tres formas: una etiqueta
`<nombre …>` que no sea `<br>`, una entidad `&…;` y un escape `#…;`.

Dos que se le parecen y **no** cuentan, las dos por un motivo medido:

- **`<<interface>>`** — mmdr lo dibuja bien, es lo más idiomático que tiene un
  diagrama de clases, y ya tumbó al validador estricto del §3.1. Un `<` pegado a
  otro `<` no abre etiqueta.
- **`Map<String,Int>`** — la coma delata que no hay nombre de etiqueta detrás del
  pico. Se dibuja tal cual, que es lo correcto, y no se avisa.

**Falsos positivos: uno en 74 casos, y no lo es.** Corrido el banco de 63 del
prototipo 21 más las familias y las nueve sondas del prototipo 23, el aviso salta
en **un** caso: `fc-br-01-html-etiqueta`, que lleva un `<b>`. Sobre los 30 casos
nuevos salta en los 20 que llevan algo literal y calla en los 10 que salen bien.

## 6. Lo que no se ha medido

- **La ventana, fotografiada.** Los píxeles son los del Visor —mismo `usvg` y
  mismo `resvg`— pero la captura de la ventana real necesita permiso de grabación
  de pantalla, que esta corrida no tenía. `mira.py` queda escrito para cuando lo
  haya.
- **Si el aviso cambia lo que el agente escribe.** Nadie ha visto todavía a un
  agente recibir este aviso y reescribir la etiqueta. Es lo mismo que le falta a
  los otros tres, y se mide cuando el MVP exista.
- **Los canales laterales.** El aviso mira nodos, grupos y aristas; un `Note over`
  de `sequenceDiagram` con `<b>` dentro se dibuja literal y callando. Queda
  escrito en el §11.4. Las dos familias que se prometen no tienen ese hueco.
- **La descripción de las herramientas.** El ticket puso en la balanza que cubre
  un fallo de 0/17 e ignora éste; sigue siendo un dato y no se ha tocado, porque
  el §5.3 la revisa cuando el MVP exista y el aviso ya cierra el agujero sin
  cobrar peaje en cada llamada.

[42]: https://github.com/javierponferradalopez/ai-render/issues/42
[p21]: ./prototipos/21-lo-que-mmdr-traga/
[p22]: ./prototipos/22-obedece-el-agente/
[p24]: ./prototipos/24-el-html-en-las-etiquetas/
