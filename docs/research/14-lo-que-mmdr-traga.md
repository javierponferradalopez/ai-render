# Lo que mmdr traga

Censo disparado el 2026-09-02 por [¿Qué traga mmdr sin dibujar?][27], que
[¿Qué añade esto sobre Mermaid?][15] dejó pendiente al descubrir que `namespace`
sale con `exit 0` y sin caja.

**Hay tres fugas más de las que el ticket sospechaba, y la peor no es que mmdr
trague sin dibujar: es que dibuja lo que nadie ha escrito.** El `Límite honesto`
saltó **una vez en sesenta y tres casos**, y esa vez fue un falso positivo: rechaza
`<<interface>>`, que es Mermaid válido y que el propio mmdr sabe dibujar por su
camino permisivo, medido.

## Nota sobre el método

Todo lo que sigue está **compilado y ejecutado**: macOS 26.6.2 arm64, `rustc`
1.98.0, crate `mermaid-rs-renderer` **`=0.3.1`** de crates.io, `default-features
= false`. No interviene el binario `mmdr`.

El banco corre la tubería que flipchart va a ejecutar, entera y en ese orden:
`parse_mermaid_strict` → vaciado de los siete campos de estilo ([#11]) →
`Direction::LeftRight` impuesta en `flowchart` y `classDiagram` ([#25]) →
`compute_layout` → `render_svg`. Scripts y casos: [prototipo 21][p21].

Son **63 casos**: 40 constructos de las dos familias del `Límite honesto` más
`stateDiagram-v2`, `sequenceDiagram` y `erDiagram`, y **un ejemplo mínimo de cada
uno de los 23 tipos de diagrama** que mmdr declara.

Y entra un segundo motor que el esfuerzo no había usado nunca: **el parser de
Mermaid 11.12.0**, sobre jsdom, sin renderizar y sin Chromium. Sólo contesta *¿es
esto Mermaid válido?*. Hace falta porque sin él **un caso que mmdr destroza no se
distingue de un caso mal escrito**, y toda la tesis de [#15] —*el agente escribe
Mermaid*— descansa en esa diferencia. Resultado: **61 de los 63 casos son Mermaid
válido**; las dos excepciones aparecen abajo y las dos dicen algo.

## 1. Cuatro desenlaces, y sólo uno de ellos se ve

El ticket buscaba *"lo que mmdr acepta y no dibuja"*. Medido, eso es una de cuatro
cosas que le pueden pasar a un constructo, y sólo la primera avisa:

| desenlace | qué pasa | casos |
|---|---|---|
| **Rechazo** | `ParseError`. El `Límite honesto` funcionando | **1** |
| **Fuga** | parsea, se pierde significado, no se dibuja nada, no se dice nada | **12** |
| **Invento** | se dibuja un nodo que nadie escribió | **6** |
| **Deformación** | se dibuja lo escrito, mal | **2** |
| Correcto | | 42 |

Ningún caso provocó pánico del renderer, que era el quinto desenlace de [#11].

**Invento y Deformación son categorías que el ticket no tenía**, y son peores que
la Fuga: en una fuga el usuario ve *de menos* y puede sospecharlo; en un invento
ve *de más*, y lo que ve de más tiene el mismo aspecto que lo que sí escribió el
agente.

## 2. El censo de fugas

Doce constructos entran, se pierden y no dejan rastro. Los agrupo por lo que se
pierde, porque el remedio no puede ser el mismo para los tres grupos.

**Se pierde estructura** — es la fuga que existe este producto para no cometer:

| constructo | qué se pierde |
|---|---|
| `namespace Dominio { … }` (`classDiagram`) | la caja y el nombre del grupo; las clases salen sueltas |
| `namespace` con clases con cuerpo | igual, y no mejora por llevar miembros dentro |
| `note "…"` y `note for X "…"` (`classDiagram`) | la nota entera, texto incluido |
| una línea de prosa suelta en `classDiagram` | la línea entera, en silencio |
| `title` en `C4Context` | el título del diagrama (le pasa igual a `zenuml`, ver §4) |
| iconos de `architecture-beta` (`(cloud)`, `(database)`) | lo único que distingue un servidor de una base de datos |

**Se pierde una pista de layout** — menos grave, pero es significado que el agente
creía estar dando:

| constructo | qué se pierde |
|---|---|
| `A ----> B` (longitud de arista) | el `Edge` del IR no tiene campo de longitud: `---->` y `-->` son idénticos |

**Se pierde estilo, sin el aviso que [#11] prometió** — y esto es un agujero en una
decisión ya tomada, no una fuga nueva:

| constructo | dónde aterriza |
|---|---|
| `click A "url" "tooltip"` (`flowchart`) | `graph.node_links`, que **no es uno de los siete campos** que se vacían y avisan |
| `%%{init: {"theme": "forest"}}%%` | `ParseOutput.init_config`, que **ni siquiera está en el `Graph`** |
| `cssClass "X" clase` (`classDiagram`) | **en ningún sitio**: se descarta al parsear |
| `link X "url"` (`classDiagram`) | **en ningún sitio** |

Los siete campos de [#11] cubren `classDef`, `style`, `class` y `:::` de
`flowchart`, y nada más. El caso `fc-15` lo confirma por el lado bueno —tres de los
siete campos vienen llenos y el aviso saltaría—; los cuatro de arriba pasan sin
que se llene ninguno, así que **el estilo se limpia y se avisa en `flowchart`, y se
limpia callando en todo lo demás**.

## 3. El invento: mmdr no tiene «esta línea no la entiendo»

Los seis inventos tienen una sola causa, y está en el fuente. La última rama del
bucle de `parse_flowchart` es:

```rust
if let Some((node_id, node_label, node_shape, node_classes)) = parse_node_only(&line) {
    graph.ensure_node(&node_id, node_label, node_shape);
    …
}
```

No hay rama de error. **Lo que no es cabecera, ni estilo, ni `click`, ni arista, es
un nodo.** De ahí salen los seis:

| fuente | lo que se dibuja |
|---|---|
| `flowchart` sin dirección | un tercer nodo, `flowchart`, con etiqueta `flowchart` |
| `Uno@{ shape: cyl, label: "X" }` | un **rombo** con id `Uno@` y etiqueta `shape: cyl, label: "X"` — y `Uno` sigue siendo un rectángulo |
| `class Pedido["Pedido de venta"]` | un tercer nodo con id y etiqueta `Pedido["Pedido de venta"]` |
| `Izquierda <--> Derecha` (`classDiagram`) | un nodo llamado `> Derecha` |
| una línea de prosa suelta en `flowchart` | un nodo con su primera palabra: `esto` |
| `radar-beta` | **el fuente entero** metido en la etiqueta de un solo nodo, corchetes y comillas incluidos |

Dos de ellos —`@{ shape: … }` y `class X["…"]`— son **sintaxis vigente de Mermaid**,
la que un agente escribe cuando escribe el Mermaid de hoy. El parser de Mermaid
11.12.0 los acepta.

Y el sexto es el que mejor lo resume: **la prosa suelta es el único caso del banco
que Mermaid rechaza** (`Parse error on line 3`) **y mmdr la dibuja**. En el único
sitio donde el idioma dice que no, mmdr dice que sí.

Ninguno de los seis lo caza la regla de la asimetría: **los seis traen etiqueta**.
Un nodo vacío al lado de uno lleno es lo que [#11] enseñó a mirar, y esto es lo
contrario — un nodo lleno, indistinguible de los que sí escribió el agente.

## 4. La deformación

Dos casos dibujan lo escrito, y lo dibujan mal:

- **Cadenas markdown.** `Uno["`` `**Negritamarkdown** y salto` ``"]` se dibuja con
  las comillas invertidas y los asteriscos **literales** en pantalla. Mermaid lo
  pone en negrita y parte la línea.
- **`zenuml`.** `Cliente->Servidor.peticion()` se convierte en un `sequenceDiagram`
  con un participante llamado `Servidor.peticion()`. El texto llega, colgado del
  sitio equivocado.

## 5. El `Límite honesto` saltó una vez, y se equivocó

El único `ParseError` de los 63 casos:

```
classDiagram
  class Repositorio {
    <<interface>>     ← unexpected token '<<interface>>' at 3:5; expected node identifier
    +guardar()
  }
```

No es la sangría: las cuatro variantes de `probe/` fallan igual, con y sin
sangría, dentro y fuera del cuerpo. Es el **validador** que `parse_mermaid_strict`
antepone a `parse_mermaid`, y en concreto su regla *«ninguna línea puede empezar
por un operador de flecha»*:

```rust
match bytes[0] {
    b'-' | b'=' | b'~' | b'.' | b'<' => { … }
```

`<<interface>>` empieza por `<`. Y **mmdr sabe dibujarlo**, medido por el camino
permisivo (`bench --permisivo`, que llama a `parse_mermaid` sin validador): sale
en pantalla encima del nombre de la clase, como en Mermaid.

```
<<interface>>Repositorio      ← el mismo fuente, por parse_mermaid
+guardar()
RepositorioSql
```

Es decir: el validador que compra el rechazo tipado le quita a mmdr algo que mmdr
hace bien. La prueba unitaria del propio crate —`parse_class_stereotype_annotation`—
lo da por bueno, porque llama al camino permisivo.

Así que la anotación de estereotipo —lo más idiomático que tiene un diagrama de
clases, y justo lo que se escribe para explicar un refactor— **se rechaza siendo
válida**. Es un fallo honesto (se ve, no se calla), pero es la frontera equivocada
en el sentido contrario a todos los demás hallazgos.

## 6. La dirección de un `subgraph` escapa a la imposición de [#25]

Medido con control: `direction RL` dentro de un `subgraph` **sí se honra**, incluso
con `Direction::LeftRight` impuesta sobre el `Graph`. Las posiciones se invierten
respecto al mismo caso sin esa línea:

| | Uno | Dos |
|---|---|---|
| con `direction RL` en el grupo | x = 209,8 | x = 87,0 |
| sin ella (control) | x = 87,8 | x = 210,6 |

[#25] decidió imponer la dirección y avisar. `Subgraph.direction` es un segundo
mando, dentro del `Graph`, que la imposición no toca: la decisión se aplica al
diagrama y no a sus grupos.

## 7. El careo palabra a palabra sirve para medir y no para vigilar

El ticket proponía dos formas de detectar una fuga: comparar el `Graph` con el
SVG, o mantener una lista de constructos vetados. Se ha implementado una tercera,
más barata y del mismo espíritu que `render_strict` —comparar **las palabras del
fuente con las palabras del SVG**—, y **careada contra el censo, no vale**. Sobre
los 40 casos de las dos familias del `Límite honesto`:

- marcó **12 sospechas**, de las que **3 eran falsas**: un id con etiqueta propia no se
  dibuja (correcto), el estilo de `classDef` sí avisa por otro sitio (correcto), y
  `<<choice>>` no se dibuja **porque se convierte en un rombo** (correcto);
- y **se le escaparon las 5 peores**, que son los inventos y la deformación:
  ninguna de ellas pierde una palabra. `@{ shape: cyl, label: "X" }` sale entero en
  pantalla — como basura, pero sale.

Y el motivo de fondo es el que mata el método: **para que funcione hace falta una
lista de palabras que son sintaxis de Mermaid**. Esa lista tuvo que crecer tres
veces durante el barrido y hoy tiene noventa entradas. Es exactamente la lista
negra que [#15] rechazó al elegir `render_strict`, con otro nombre.

Lo que sí funciona es el otro lado, y es más simple de lo que parecía: **los seis
inventos se ven desde el `Graph`, sin mirar el SVG y sin lista de nada** —cinco
producen un `id` que no es un token del fuente (`Uno@`, `> Derecha`,
`Pedido["Pedido de venta"]`, `esto`, el nodo entero de `radar`), y el sexto es el
`id` que coincide con la palabra de la cabecera—. El censo no lo decide, pero deja
el dato: **la detección barata está del lado del inventar, no del lado del tragar.**

## 8. El barrido de las 23 familias

Ninguna de las 23 familias declaradas se cae, y ninguna sale vacía. Lo que hay es
degradación desigual, y sólo tres merecen mención:

- **`radar-beta`** no está implementada de verdad: el fuente se dibuja como texto
  dentro de un nodo. Es el peor invento del banco.
- **`C4Context`** dibuja formas y relaciones, y **pierde el título**.
- **`architecture-beta`** dibuja grupo, servicios y arista, y **pierde los iconos**.

El resto —`pie`, `mindmap`, `journey`, `timeline`, `gantt`, `requirement`,
`gitGraph`, `sankey`, `quadrant`, `block`, `packet`, `kanban`, `treemap`,
`xychart`— dibuja lo que se le da. Sostiene la decisión de partida 4: no están
prohibidas, están no probadas, y ahora están **una vez** probadas.

## 9. Lo que no se ha medido

- **La calidad del dibujo** de las 21 familias fuera del `Límite honesto`: se ha
  mirado que llegue el significado, no que se lea bien.
- **Los constructos de Mermaid que ni Mermaid 11.12 ni mmdr conocen.** El banco
  cubre lo que un agente escribe hoy; la evolución del idioma es justo lo que
  ninguna medición puntual cubre, y es el argumento de [#15].
- **Qué dibuja Mermaid.js** en los casos deformados. Se ha comprobado que son
  válidos, no cómo quedan: renderizarlos pedía Chromium, y el punto —que son
  Mermaid legítimo— no lo necesita.
- **Diagramas grandes**, que siguen siendo niebla del mapa.

[27]: https://github.com/javierponferradalopez/ai-render/issues/27
[15]: https://github.com/javierponferradalopez/ai-render/issues/15
[#11]: https://github.com/javierponferradalopez/ai-render/issues/11
[#25]: https://github.com/javierponferradalopez/ai-render/issues/25
[p21]: ./prototipos/21-lo-que-mmdr-traga/
