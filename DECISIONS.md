# flipchart — las decisiones del MVP

> **Qué es esto.** El estado final de lo que quedó decidido al cartografiar el MVP,
> escrito para quien no ha leído ningún ticket. Hace par con
> [`CONTEXT.md`](CONTEXT.md): allí el idioma, aquí las decisiones. Los términos en
> **negrita** están definidos allí.
>
> **Qué no es.** No es el spec de construcción: no trae historias de usuario, ni seams de
> test, ni los cortes verticales del trabajo. Eso lo produce `/to-spec`, que cita este
> documento —y en particular su §11— en vez de repetirlo.
>
> La historia, por dónde se pasó para llegar hasta aquí, vive en los issues del
> [mapa](https://github.com/javierponferradalopez/ai-render/issues/1) y no se repite en
> este documento; lo único que entra de ella es lo que sigue siendo una restricción viva.
> **Lo que no esté escrito aquí, no existe para quien construya.**

---

## 0. Qué se construye

Una **Pizarra** efímera para que un agente de IA se explique: un canal visual temporal
en el que el agente pone diagramas mientras habla, y que muere con él.

El **caso protagonista** es entender un refactor o un movimiento de estructura antes de
hacerlo: `actual` junto a `propuesto`, diagramas de clases junto a flujos.

La forma del producto, en una línea: **un plugin de Claude Code que trae un solo binario
de macOS; ese binario es a la vez servidor MCP por stdio y una ventana nativa que dibuja
Mermaid**.

Cinco cosas lo distinguen de dibujar Mermaid en cualquier otro sitio, por orden de lo
irreplicable:

1. **Se niega a dibujar una mentira.** Mermaid inventa nodos —un typo produce una clase
   con cara de verdad— y todos los renderers lo dibujan. Éste no.
2. **Muere con la sesión MCP.** Ni archivo, ni galería, ni `save`, ni fichero que
   sobreviva.
3. **N Vistas con nombre** conviviendo en una ventana, conducidas por el agente.
4. **Los dos estados vacíos**: la pizarra dice cuándo está vacía y cuándo la sesión
   terminó.
5. **El estilo es de la pizarra, no del agente.**

Y lo que cuesta, al lado, o la lista no es honesta: **los grupos se dibujan peor** que
con Mermaid.js; **prometemos "Mermaid" y entregamos "el Mermaid de mmdr"**, que divergen
en la sintaxis de borde; **sólo macOS**; y sin exportar ni editar.

---

## 1. El artefacto

**Un ejecutable de Rust**, universal binary de macOS (arm64 + x86_64), sin runtime, sin
navegador, sin Node, sin Python y sin proceso hijo. Dentro:

| Pieza | Crate | Papel |
|---|---|---|
| Servidor MCP por stdio | `rmcp` (SDK oficial, `modelcontextprotocol/rust-sdk`) | expone `show` y `clear` |
| Parse + layout + SVG | `mermaid-rs-renderer` **`=0.3.1`**, MIT, `default-features = false` | el idioma y el dibujo |
| Rasterizado | `resvg` | SVG → textura, al zoom y DPI actuales |
| Ventana | `eframe`/`winit` + `objc2` / `objc2-app-kit` | la ventana, el Dock y el foco |

**La versión de `mermaid-rs-renderer` es exacta, y se sube a mano mirando las imágenes.**
En un motor de layout un parche cambia el dibujo, y el proyecto compra de él dos
propiedades: determinismo byte a byte (mismo fuente → mismo SVG) y estabilidad al
actualizar (añadir una clase a un diagrama de seis movió un nodo). Se renuncia a los
arreglos de layout gratis a cambio de que el dibujo no se mueva a espaldas nuestras.

Riesgo asumido y escrito: `0.3.1`, **un solo mantenedor**, y aviso del propio autor de
que la calidad visual *"may not yet match mermaid-cli"*. Mitigaciones, no soluciones:
versión exacta, licencia MIT para poder forkear, y el `ParseError` tipado que su autor
escribió para *"LLM correction loops"*.

**Lo que se entrega es un fichero.** Ninguna pieza puede convertirse en un segundo
artefacto versionado que deba coincidir de versión con el primero.

---

## 2. Un proceso, dos hilos

El host lanza el binario como proceso hijo y le habla por stdio. Dentro:

- **Hilo principal: el event loop** de `winit`/`egui` — es donde macOS lo exige. Sólo
  dibuja.
- **Hilo secundario: el Servidor MCP** — dueño del estado de las N Vistas, y quien
  decide cuándo sale el proceso.

Con ello **desaparece el IPC entero**: ni socket, ni protocolo interno, ni handshake, ni
procesos huérfanos. Y lo efímero deja de ser una regla que alguien aplica: es la vida del
proceso, garantizada por el sistema operativo.

**Lo que viaja del servidor al Visor es el SVG**, por un canal en memoria — no la
geometría y no un bitmap. El `Layout` de mmdr **es** la **PositionedScene**, vive en el
hilo del servidor y **no cruza**. El pixmap es caché: `resvg` rasteriza bajo demanda en un
hilo de trabajo y el event loop sólo sube la textura; si viajara el bitmap, el zoom se
vería borroso o pediría un render por cada rueda de ratón.

### La regla que gobierna el reparto: el event loop no es un reloj

**Con la ventana completamente tapada por otra, macOS no ralentiza el event loop: lo
para.** Medido: cuatro `show` seguidos sin un solo repaint en 12 s, y una sesión muerta
que el proceso tardó 11 s en notar porque el aviso se procesa en el `update()` y no había
`update()`.

Que no se repinte mientras nadie mira no es un defecto —al destapar la ventana se dibuja
lo último—. El defecto es el otro lado: **procesos que sobreviven a su sesión**, con una
pizarra fantasma enseñando el diagrama de una conversación muerta. Y tapada es el caso
*normal*: el usuario está en su terminal.

> **Todo lo que dependa del tiempo o de la muerte de la sesión vive en el hilo del
> servidor, que sí corre siempre. El hilo del servidor tiene la última palabra sobre la
> salida del proceso; el event loop sólo dibuja.**

`beginActivityWithOptions` (App Nap) arregla la latencia de la muerte —de 11 s a 1 ms—
pero no el repaint, que sigue errático. Se pone por lo primero.

### Números medidos (macOS 26.6.2 arm64, `eframe` 0.36.1 / `winit` 0.30.13, build de debug)

| | |
|---|---|
| Event loop arriba | 72–350 ms |
| stdio vivo con la ventana abierta | responde en 0,1 ms |
| Repaint desde el hilo del servidor, ventana en segundo plano | 52–55 ms |
| Muerte al cerrarse stdin, ventana a la vista | sale solo, código 0, 3,1 s |
| RSS de una sesión que nunca muestra nada | **96,8 MB** |
| La misma, tras el primer `show` | 97,2 MB |
| Render de mmdr | 3,3 ms (6 clases) – 14,9 ms (17) |

**El 99,6 % del coste de memoria se paga al crear el event loop, no al mostrar la
ventana** — de ahí el arranque diferido de §6.

---

## 3. La tubería de render

Todo ocurre en el hilo del servidor, en proceso, dentro de la llamada a `show`.

```
1  parse_mermaid(source) -> Result<ParseOutput, _>        // permisivo: el validador no vota
     └─ si falla: parse_mermaid_strict(source)            // sólo para el ParseError tipado
2  vaciado de los nueve canales de estilo del ParseOutput
3  imposición de la dirección sobre el Graph y sus Subgraph
4  reglas del Límite honesto sobre el Graph               // §4
5  compute_layout(&graph, &theme, &config) -> Layout
6  render_svg(&layout, &theme, &config)   -> String
```

Los pasos **5 y 6, y también el 1, van envueltos en un guardián de pánico**
(`catch_unwind`). Servidor y Visor comparten proceso, así que un pánico sin capturar se
llevaría la pizarra entera por delante, callando. Un pánico es un rechazo más (§4.3).

### 3.1 `parse_mermaid`, y el validador degradado a diagnóstico

Se entra por **`parse_mermaid`**, el camino permisivo. `parse_mermaid_strict` antepone un
validador de seis comprobaciones de forma que **no sostiene nada**: sobre un banco de 63
casos aportó cero rechazos correctos y uno del lado equivocado —tira `<<interface>>`, que
es Mermaid válido, que mmdr dibuja bien y que es lo más idiomático que tiene un diagrama
de clases—.

No hay que elegir entre los dos, porque **strict es el validador antepuesto al
permisivo**: si el permisivo falla, el strict falla seguro. Así que:

```
parse_mermaid(source)              → si sale, se dibuja
   └─ si falla
      parse_mermaid_strict(source) → sólo para obtener el ParseError tipado del mensaje
```

El segundo parse se paga únicamente cuando ya no se va a dibujar nada, que es donde la
latencia no importa.

### 3.2 El vaciado: nueve canales de estilo

**El estilo es de la pizarra, no del agente.** Se vacía del IR después del parse — no se
persigue en el texto, porque por muchas formas nuevas de escribir estilo que Mermaid
invente, para tener efecto tienen que aterrizar en uno de estos campos:

| | |
|---|---|
| `class_defs` | `classDef` |
| `node_classes` | `class`, `:::` |
| `node_styles` | `style` |
| `subgraph_styles` | |
| `subgraph_classes` | |
| `edge_styles` | `linkStyle` |
| `edge_style_default` | |
| `node_links` | `click` (además es interacción, y está fuera de alcance) |
| `init_config` | `%%{init: …}%%` (está en el `ParseOutput`, no en el `Graph`) |

**Se avisa cuando venían llenos** (§4.4) — limpiar en silencio sería reintroducir la
mentira por nuestra parte: un agente que escribió `classDef danger fill:#f00` le diría al
usuario *"los módulos de riesgo van en rojo"* y no hay nada rojo en pantalla.

Residuo escrito: **la prohibición de píxeles es limpiada-y-avisada, no imposible.** Nada
estructural impide que un campo *nuevo* de una versión futura se escape; con la versión
clavada en `=0.3.1`, eso se ve al subir la versión y no antes.

Dos canales de estilo **se pierden callando** porque no dejan nada que mirar: `cssClass` y
`link` de `classDiagram` se descartan al parsear y no aterrizan en ningún campo. Coste
conocido y aceptado.

### 3.3 La dirección: se impone, y se avisa

**`graph.direction = Direction::LeftRight`, impuesta tras el parse, sólo en `flowchart` y
`classDiagram`** — y **también vaciando la dirección de cada `Subgraph`**, para que herede
la del diagrama (o `LeftRight` si el campo no admite vacío, que da igual porque la del
diagrama es ésa).

Que baje a los grupos no es un fleco: un `subgraph` con `direction` propia es la misma
perilla en manos del agente, dentro del grupo, y **el estilo se prohíbe pero la dirección
se decide por él**.

Por qué LR, medido sobre noventa configuraciones y diecisiete casos: en el caso
protagonista los rodeos pasan de 1 a 0, el desvío de las aristas de 3,92× a 1,00 y la
banda vacía del lienzo del 54 % al 28 %. **Gana en dieciséis de diecisiete casos** y
pierde en uno por un rodeo. Cero aristas perdidas y cero cajas sueltas en las dos
columnas: ninguna dirección miente.

Se descartó imponer sólo cuando el agente no declara dirección: el caso protagonista
empieza por `flowchart TB`, así que esa vía curaba los dieciséis diagramas de clases y
dejaba el caso que importa exactamente como estaba.

**Nota de implementación:** mmdr **no distingue `flowchart TB` de `flowchart`** — el
parser inicializa a `TopDown` y luego pisa. Así que *"¿el fuente declaraba dirección?"*,
que es lo que decide si hay que avisar, se responde **mirando el fuente antes de
entregárselo**, igual que las reglas del §4.

Fuera de esas dos familias no se toca nada: forzar LR es no-op en `sequenceDiagram` y
`mindmap`, y cambia `stateDiagram-v2` y `erDiagram` sin romperlos — y esas familias están
declaradas no probadas (§9), así que se quedan como vengan.

### 3.4 `Theme` y `LayoutConfig`: los de fábrica, sin tocar

```rust
Theme::mermaid_default()   // fuente 16 px — no Theme::modern(), que es 14 px
LayoutConfig::default()
```

**Ninguna perilla de mmdr mejora el dibujo y varias hacen daño**, medido:
`preferredAspectRatio` no reordena nada —sólo estira el lienzo hasta cuadrar la razón
pedida, inflando los grupos de hueco vacío: pidiendo 3,0 el desvío sube a 10,70×—;
`nodeSpacing`, `rankSpacing` y los paddings mueven ±60 px de lienzo con el mismo trazado;
los cinco temas sólo cambian color, salvo por el tamaño de fuente, y a 10 px el dibujo
empeora.

Y **media `FlowchartLayoutConfig` no está enchufada**: `objective.*` y `routing.*`
completos no cambian un solo byte del layout. Sólo mueven algo `order_passes`,
`port_side_bias` y `auto_spacing`, y ninguno cura.

`LayoutConfig` y `Theme` son `Serialize + Deserialize`, así que si algún día hay valores
propios se escriben como datos y se mezclan sobre el default.

### 3.5 Los grupos: aceptados con techo, y sin plan B

El listón no es *"¿es tan bueno como Mermaid.js?"* —eso está fuera de alcance— sino *¿se
lee un refactor sin seguir aristas con el dedo?*. Con LR impuesta, el caso protagonista
cumple los tres criterios: ninguna arista cruza una región vacía, las capas salen en un
orden que se puede contar, y cabe en una ventana.

**Precio aceptado:** LR endereza el trazado tumbando el dibujo. El caso protagonista queda
a **5,23:1**, y con la regla *encoger nunca agrandar* en una hoja de 1200×800 su zoom baja
del 72 % al 56 %. Un dibujo pequeño que se sigue con la vista bate a uno grande donde hay
que seguir una arista con el dedo.

**No hay plan B, y es una decisión, no un descuido.** Arreglar mmdr nosotros nos
convertiría en mantenedores de un motor de layout; cambiar de motor abre una puerta que
este MVP no necesita.

---

## 4. El Límite honesto

La frontera que la pizarra no cruza dibujando. **Es nuestra**: la sostienen dos reglas que
miramos sobre el `Graph` ya parseado, no el validador del crate.

La raíz, de la que sale todo el reparto:

> **Lo que se ve de más se rechaza. Lo que se ve de menos se dibuja y se avisa.**

No lo decide la gravedad ni cuántos casos hay, sino **qué ve el usuario**. Ver de menos con
aviso es honesto —falta un `namespace`, una `note`, un icono, y se puede sospechar—; ver de
más no lo es a ningún precio, porque el nodo inventado **trae etiqueta** y no se distingue
del bueno, y el usuario no tiene el fuente delante para carearlo.

Y el reparto tiene un segundo argumento que conviene cobrarse porque es raro: **la
detección barata está justo del lado que hay que rechazar.** Los inventos se ven desde el
`Graph` sin lista de nada; detectar las fugas exigiría noventa palabras de sintaxis de
Mermaid, o sea una lista negra que crece con el idioma. El principio ético y el que sale
barato coinciden.

### 4.1 Las dos reglas, un solo rechazo

Se comprueban las dos **sobre el `Graph`**, en el mismo paso y momento que el vaciado de
estilo. Mirar el texto buscando sintaxis es lo que no queremos.

**Regla de la asimetría** — caza el **Nodo fantasma**, el que inventa *el idioma* al ver un
id suelto:

> Un id es Nodo fantasma cuando aparece **sólo en relaciones**, sin etiqueta ni cuerpo ni
> forma propia, **y en el mismo diagrama hay al menos un nodo que sí los tiene.**

- `A --> B` a secas → **se dibuja.** Grafo de ids desnudos, honesto: dos cajas con su id
  dentro, y eso es todo lo que se dijo. Exigir `A[A] --> B[B]` sería cobrarle tokens por
  nada y rechazar el Mermaid más idiomático que existe.
- `class Order { … }` + `Ordr --> Money` → **rechazado**, con `Ordr`, su línea y `Order`
  como candidato.
- `API[API Layer] --> Db`, con `Db` sin aparecer en ningún otro sitio → **rechazado**.

**Regla del nodo rastreable. MEDIDA EL 2026-09-04, Y SE CAE.** Lo que sigue es cómo quedó
redactada y por qué; el censo está en `docs/research/16-el-nodo-rastreable-medido.md` y el
número que la tumba es **12 falsos positivos sobre los 42 casos correctos del banco, nueve
de ellos suyos**: mmdr sí fabrica ids sintéticos legítimos —`__start_root__` por el `[*]` de
`stateDiagram-v2`, y `journey_0`, `quadrant_0`, `packet_0`, `treemap_0`—, y perdonarlos por
la forma del id libera `radar_0`, que es el invento que mejor mataba. La rebaja pide saber
qué familias implementa mmdr de verdad: **la lista de familias, con otro nombre**. Lo que
vuelve a estar abierto es **qué se hace con los seis inventos** —no el reparto de arriba, que
sigue en pie—; el §9 del informe deja los datos para elegir. La regla sigue en el código
hasta que esa decisión se tome.

Caza el **Nodo apócrifo**, el que fabrica *el parser* al rendirse con una línea que no supo
clasificar:

> Todo `id` del `Graph` tiene que aparecer como token en el fuente que escribió el agente,
> careado **contra el fuente sin su primera línea**.

El careo sin la cabecera no es un caso especial, es la regla dicha bien: **la primera línea
declara de qué tipo es el diagrama y no declara nodos en ninguna familia de Mermaid**. Es
además lo que caza el sexto invento —el nodo `flowchart` que mmdr fabrica de una cabecera
sin dirección— y lo que mata `radar-beta` (que mete el fuente entero en la etiqueta de un
nodo) **sin necesidad de ninguna lista de familias no soportadas**.

**No es la lista negra al revés, y la dirección del careo es lo que lo decide.** Ir de las
palabras del fuente al dibujo exige saber *qué palabras son sintaxis y por tanto no deben
salir* — noventa entradas que crecen con Mermaid. Esta regla va al revés: un `id` que el
`Graph` afirma tener no necesita ninguna teoría de la sintaxis para preguntarle al fuente si
estaba allí. **Crece con Mermaid sin que la toquemos.**

**Las dos son el mismo rechazo con dos causas**, y se informan **juntas y todas de una vez**
—no una por turno—. El mensaje **no reparte culpas**: distinguir el Mermaid legítimo del que
no lo es exigiría el parser de Mermaid como juez, y eso es Node y un segundo artefacto. Dice
**qué pasó y qué hacer**:

```
Rejected: 2 nodes appear in the drawing that you did not declare; nothing was drawn.
View "propuesto" is unchanged.
  "Uno@"   line 3   — not in your source
  "Ordr"   line 6   — only used in a relation
Declare every node you name, and rewrite any line the renderer turned into one.
```

Las dos causas piden cosas distintas —*declara el id* frente a *reescribe la línea*—, y por
eso el mensaje las distingue línea a línea. **No enseña sintaxis**: nada de *"usa
`A[(Label)]`"*. El aparato existe para que el agente no cuente algo falso, no para dar clase
de Mermaid.

### 4.2 Un rechazo es un resultado, no un error de transporte

Todos los rechazos viajan **dentro del resultado de la herramienta, con `isError: true`**,
nunca como error JSON-RPC.

- Un `ParseError` diseñado para bucles de corrección devuelto como error de transporte
  **tira a la basura la línea, la columna y los candidatos**, y el agente reintenta a ciegas
  o abandona.
- Y `isError: true` es lo que separa *"no se dibujó"* de *"se dibujó"*. Un rechazo sin la
  marca se lee como éxito, y entonces el agente le describe al usuario un diagrama que no
  está en pantalla — **la misma mentira que este producto existe para no dibujar, un nivel
  más arriba**.

**Un rechazo no toca la pizarra.** Si `show("propuesto", …)` falla y ya había una Vista
`propuesto`, la vieja se queda intacta y en pantalla: no se borra ni se sustituye por un
hueco.

**Y hay que saber para quién es todo esto:** el usuario **no lee nada** de lo que devuelven
`show` y `clear`, y no debe — el único canal del servidor a sus ojos es la ventana. Lo que
el usuario acaba viendo es al agente hablando. De ahí que el aparato se parta en dos piezas
con dueños distintos:

- **Que el agente no mienta** lo compra `isError: true` **más la línea fija**, y nada más.
- **Que el agente arregle y vuelva** es lo que paga el diagnóstico, y no es hipotético:
  medido, cuando la pizarra no le sale el agente **no insiste — se pasa a ASCII o a prosa,
  y no lo dice**. Un rechazo sin pistas no produce un reintento: produce una explicación en
  texto y un usuario que nunca sabrá que hubo una pizarra.

### 4.3 Los cinco desenlaces de `show`

| # | Desenlace | `isError` | ¿Se dibuja? |
|---|---|---|---|
| 0 | Entrada inválida — `view_id` vacío o >64, o `diagram` vacío | `true` | no |
| 1 | `ParseError` de mmdr, incluida la variante desconocida | `true` | no |
| 2 | Nodo fantasma o Nodo apócrifo — uno o varios, todos informados | `true` | no |
| 3 | Pánico del renderer | `true` | no |
| 4 | Éxito, con o sin nota | `false` | sí |

El orden es **completo** —cualquier `show` cae en una de las cinco filas, y en las cinco el
agente sabe si en la ventana hay algo nuevo— y **1 va antes de 2 por narices**: las reglas
se comprueban sobre el `Graph`, y si el parse falló no hay `Graph`.

Cada rechazo lleva **dos piezas**, en inglés, porque es texto para el modelo:

1. **Una primera línea fija que dice qué no pasó**, siempre igual —
   `Rejected: nothing was drawn; view "<id>" is unchanged.` Es la que impide que el agente
   siga hablando de un dibujo que no existe.
2. **El diagnóstico, una línea**, con lo que traiga el caso: id, línea, columna,
   encontrado/esperado, candidatos. Si el `expected` es una lista larga, los tres primeros.

**No se ecoa la línea del fuente**, salvo dentro del `ParseError` (donde viene en el
pass-through y desmontarlo costaría más de lo que ahorra): se le estaría cobrando al agente
por devolverle algo que acaba de escribir y tiene delante en el mismo turno. **El fuente
completo nunca se devuelve.**

**Desenlace 0 — entrada inválida.** Se rechaza **en la puerta, sin consultar a mmdr**: es el
único sitio donde validamos antes de parsear, y es porque no hay nada que parsear.

**Desenlace 1 — `ParseError`, con pass-through híbrido.** Texto propio para las variantes que
nos importan (`UnknownParticipant { name, line, candidates }`, `UnclosedSubgraph
{ opened_at }`, `UnexpectedToken { line, col, found, expected }`) y **el `Display` de mmdr
como relleno del brazo comodín**, precedido de una admisión de que no la hemos clasificado.
Ni el `Display` pelado —es lo que su autor escribió el enum para evitar— ni texto propio para
las veinte variantes de hoy y las de mañana. Con la versión clavada, el brazo comodín de
`#[non_exhaustive]` es una **red de seguridad, no una cinta de mantenimiento**.

```
Rejected: nothing was drawn; view "propuesto" is unchanged.
Unexpected token at line 4, column 3 — found "-->", expected "class", "}" or an identifier.
  4 |   --> Money : usa
```

**Desenlace 3 — el pánico, y es el único que dice que la culpa es nuestra**, a propósito: si
le decimos al agente que arregle su diagrama, lo intentará en bucle sobre algo que no tiene
arreglo.

```
Rejected: the renderer failed on this diagram; nothing was drawn.
View "propuesto" is unchanged. This is a bug in the flipchart, not in your diagram —
try a simpler diagram, or the same one with fewer nodes.
```

### 4.4 Los tres avisos, que no son rechazos

La Vista **se dibuja** y se avisa, con `isError: false`. Los tres son **literales fijos**, así
que su coste es predecible, y son acumulables. Regla común: **se avisa por lo que venía, no
por lo que tuvo efecto** — un `classDef` que ninguna clase usaba también avisa, porque el
agente creyó que estaba pintando.

**(a) Estilo descartado** — un solo aviso lo cubre todo (estilo, `click` y `%%{init}%%`), no
uno por categoría. ~35 tokens:

```
Note: style directives (classDef, class, style, linkStyle) and click links were dropped —
the flipchart decides how views look. The view was drawn.
```

**(b) Estructura que mmdr no dibuja** — sólo `namespace` y `note`, y **sólo en
`classDiagram`**. Dice algo distinto del anterior: el de estilo dice *decidimos nosotros cómo
se ve*, éste dice *no sabemos dibujar esto*.

```
Note: the flipchart could not draw namespace here; the classes were drawn without it.
```

Se escriben como lo que son: **dos deudas de mmdr con nombre y fecha, no una política.**
Están dentro porque `classDiagram` es el caso protagonista y `namespace` es cómo un diagrama
de clases dice *módulo* — y el refactor que se quiere entender es un movimiento entre
módulos. Perderlo en silencio sería que el agente diga *"estas tres clases se van a
`Dominio`"* con tres clases sueltas y ninguna caja en pantalla. Son **dos comprobaciones de
texto**, y **sólo aviso, nunca rechazo**: rechazar dejaría al agente sin poder dibujar un
diagrama de clases válido por una limitación nuestra.

**(c) Dirección impuesta** — sólo cuando el fuente declaraba otra dirección. Si no declaraba
nada, no hay nada que avisar y no se paga. Hace falta porque **el agente es ciego**: si
escribe `flowchart TB` y le damos LR sin decírselo, describirá un dibujo que no está.

Redacción de referencia, no medida:

```
Note: the flipchart lays diagrams out left to right; the direction in your source was ignored.
The view was drawn.
```

**Un rechazo nunca lleva avisos.** Si no se dibujó nada, contarle además que le tiramos los
colores es ruido sobre algo que va a reescribir entero.

### 4.5 Lo que se pierde y no se avisa, escrito

Nada de esto es un fallo a arreglar: es coste conocido.

- **Pistas de layout**: `A ----> B` y `A --> B` son el mismo byte — el `Edge` del IR no tiene
  campo de longitud.
- **`cssClass` y `link` de `classDiagram`**: estilo sin campo que mirar (§3.2).
- **Cinco fugas de estructura** en familias no probadas: `note for`, prosa suelta en
  `classDiagram`, títulos de `C4Context` y `zenuml`, iconos de `architecture-beta`. Perderlas
  es el precio ya aceptado de dibujar familias sin garantía.
- **Dos deformaciones**: markdown literal y `zenuml` se dibujan feos. Convivencia, sin aviso.

---

## 5. La API MCP

**Dos herramientas y sólo dos.** No hay `update` —no hay patch: `show` sobre un `view_id`
existente **reemplaza** la Vista— y no hay tercera herramienta de consulta.

```
show(view_id: string, diagram: string)   // Mermaid; reusar un view_id reemplaza la Vista
clear(view_id?: string)                  // una Vista, u omitir para toda la pizarra
```

El servidor se llama **`flipchart`**, así que el host las presenta cualificadas
(`mcp__flipchart__show`) y el prefijo en el nombre propio no compra namespace: lo duplica y
tartamudea.

> **Fleco de redacción, pagado el 2026-09-03.** Los textos medidos (la descripción de §5.3 y
> la línea recomendada de §8.2) estaban escritos con `flipchart_show` / `flipchart_clear`, que
> es con lo que se midieron el peaje y el disparo. Las herramientas se registran como **`show`
> y `clear`**, y el nombre que el host presenta —comprobado contra una sesión real de Claude
> Code— es **`mcp__flipchart__show`** / **`mcp__flipchart__clear`**. Es el que llevan ahora los
> dos textos, y no ha cambiado nada más de ellos.

**No hay `title`.** El `view_id` **es** el nombre visible. Un título aparte permitiría que la
ventana ponga *"Estructura propuesta del módulo de pedidos"* y el agente diga *"el
propuesto"*: dos nombres para una cosa, que pueden desincronizarse. Además cuesta peaje fijo
en todas las conversaciones, incluidas las que nunca dibujan, y Mermaid ya trae su
frontmatter `title:` para quien quiera prosa.

### 5.1 Validación de entrada

Lo único que se valida antes de parsear:

- **`view_id`**: **prosa, no un slug.** No vacío tras recortar espacios, y tope de **64
  caracteres**. **Ninguna policía de caracteres** — `Estructura actual` es un id
  perfectamente bueno.
- **`diagram`**: no vacío.

### 5.2 Qué devuelve

**`show`, cuando sale bien** — ~20 tokens, tres piezas que se pagan solas:

```
Shown as view "propuesto" (8 nodes, 9 edges). Views on the flipchart: actual, propuesto.
```

- **El acuse** confirma el `view_id` con el que se guardó, que es el nombre por el que el
  agente va a referirse a la Vista.
- **El recuento** es su única realimentación sobre el dibujo, porque **la imagen no vuelve
  nunca al contexto**. La divergencia grave ya se rechaza; el recuento pilla la que las
  reglas permiten a propósito (el grafo de ids desnudos) y cualquier auto-creación futura no
  prevista.
- **La lista de Vistas vivas** tapa un agujero que si no queda abierto: tras un `/clear` la
  conversación se va pero **la pizarra sobrevive**, así que el agente nuevo no sabe que en
  pantalla siguen `actual` y `propuesto` y **no tiene forma de preguntarlo**. Con la lista en
  la respuesta se lo cuenta gratis; sin ella, la salida sería una tercera herramienta.

**No devuelve** nada de la ventana (si se abrió, si estaba oculta), nada de geometría, y **ni
un byte del SVG**.

**`clear`, simétrico e idempotente:**

```
clear("propuesto")   →  Cleared view "propuesto". Views on the flipchart: actual.
clear()              →  Cleared the flipchart. No views.
clear("propeusto")   →  No view "propeusto" on the flipchart. Views: actual, propuesto.
clear() sobre vacío  →  The flipchart was already empty.
```

- **Borrar un id que no existe no es un error** (`isError: false`): lo que se pedía era que
  esa Vista no estuviera, y no está. Pero **sí se dice, con la lista al lado**, porque un typo
  en el `view_id` significa que el agente le está hablando al usuario de una Vista que no es
  la de la pantalla. Marcarlo como error invitaría a un reintento a ciegas por algo que no hay
  que arreglar.
- **`clear()` no cierra la ventana**: la deja en el estado *pizarra vacía* (§6.3).

**Factura por uso** (`cl100k_base`): ~20 tokens un `show` que sale bien, ~35 más por cada
aviso, ~30-40 un rechazo. El **peaje fijo** de las dos herramientas, medido el 2026-09-03 sobre
el `tools/list` que emite el binario, es de **302 tokens en su peor caso conocido** — 264 si se
descuenta el `$schema` que `rmcp` cuelga de cada esquema y que no dice nada — y no es criterio
de diseño: lo que se vigila es el coste por uso. (El peaje real lo decide además el host: la
misma herramienta cuesta +15 tokens con búsqueda de herramientas activa y +69 sin ella.)

### 5.3 La descripción de las herramientas

**Es un manual de uso, no un canal de persuasión.** Le habla a un agente que **ya ha decidido
llamar**, no a uno al que hay que convencer — y con ese listón, el *cuándo* usar la pizarra se
cae solo: se muda a la línea de instrucciones de proyecto (§8.2), que es el único canal que
funciona (§8.1).

Texto (los **302 tokens** de §5.2 son este texto más los esquemas que lo acompañan):

```
mcp__flipchart__show
  Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.

  Any id used in a relationship must carry a label or a body when another id in the same
  diagram does; a bare id alongside a labelled one is rejected.

  Showing an existing view id replaces it and brings it to the front; several named views
  coexist. The flipchart dies with the session.

  view_id   Short human-readable name, shown to the user above the diagram - e.g.
            "Current dependencies", not "v1". Reusing a name replaces that view.
  diagram   Mermaid source.

mcp__flipchart__clear
  Remove one view from the flipchart, or all of them. Does not close the window.

  view_id   View to remove. Omit to clear the whole flipchart.
```

Por qué está cada pieza:

- **Qué hace y qué toma.** Obligatorio.
- **El `view_id` con su ejemplo** (13 tokens). Es lo único de la descripción con eficacia
  medida: 17 de 17 nombres espontáneos salieron en prosa legible, ni un `v1`.
- **La cláusula de la asimetría** (34 tokens). No está por frecuencia —medido, el agente no
  deja ids desnudos nunca: 0 de 17 diagramas espontáneos con Nodo fantasma, 0 ids desnudos—
  sino porque **el fallo que evita es silencioso**: ante un tropiezo el agente no insiste, se
  pasa a prosa y no lo dice. No hay control sin ella, así que retirarla sería apostar sin
  dato. **Se revisa cuando el MVP exista**, que es cuando habrá conversaciones de verdad que
  contar.
- **Reemplazo y coexistencia.** Volver a mostrar sobre un id existente pisa esa Vista y **no
  da error de nada**: es un fallo sin canal de rechazo posible, así que hay que contarlo por
  adelantado.
- **Que muere con la sesión.** El agente se lo repite al usuario; sin ella le promete
  permanencia.

---

## 6. El Visor

Una ventana nativa, escrita entera en `egui` — sin CSS y sin tecnología web.

### 6.1 Un rotafolio: una hoja y un foco, sin índice

**Una Vista a la vista**: la hoja de delante, su nombre, dos flechas `‹` `›` y «hoja 5 de 7».
**Sin barra de pestañas y sin lista de las demás.**

- **El nombre siempre visible** es el `view_id`, que es lo que el agente dice en voz alta y lo
  que el usuario teclea de vuelta.
- **El zoom es de la hoja**: cada Vista recuerda el suyo.
- **Encaje: encoger, nunca agrandar.** Agrandar miente — pone un diagrama de 3 nodos al 128 %
  al lado de uno de 20 al 27 %.
- **Orden: el de creación**, que es literalmente el orden de la pila. No se reordena al hacer
  `show`: con el agente moviendo la hoja activa, reordenar además dejaría al usuario sin
  ningún punto fijo en pantalla.
- **`show` deja su Vista delante** — es pasar la hoja. Nueva o reemplazada, queda activa; sin
  esto el agente dice *"mira el propuesto"* y el usuario tiene que ir a buscarlo.
- **Si el agente retira la Vista activa**, pasa a la del `show` vivo más reciente. Si no queda
  ninguna, la ventana entra en *pizarra vacía*.
- **El feedback del usuario va por el chat.** *«tira por la opción C»* se teclea en la
  conversación, no se clica en la ventana. **No se abre ningún canal del Visor al agente.**

Por qué no columnas, ni rejilla, ni apiladas —las cuatro maquetadas sobre SVG real—:
**la comparación no necesita la ventana.** Tres variantes como `subgraph` de un `flowchart`
caben en una Vista, etiquetadas y legibles, así que enfrentar N versiones de un diseño es
trabajo del idioma. Y lo que rompe las columnas no es N, es la disparidad de tamaños: dos
Vistas salen al 73 % y 88 %, cinco heterogéneas mandan una al 20 %.

Un índice, además, es **mando de administración sobre la pizarra**, y el usuario no
administra: observa.

**Precio aceptado:** volver atrás es **lineal y a ciegas** —de la hoja 7 a la 2 son cinco
pasos, y no se sabe qué hay en la 2 hasta llegar—. Se acepta porque el usuario no navega: se
lo pide al agente y la hoja pasa sola, que es un `show` y un turno. **Las flechas se quedan**
porque cuestan cero y salvan el *«déjame volver un segundo a lo anterior»* sin gastar un
turno: es el único mando que el usuario tiene, y no toca lo que hay, sólo cuál se mira.

### 6.2 La ventana: cuándo aparece, cuándo roba el foco, cuándo desaparece

- **Arranque diferido.** El hilo principal **no llama a `run_native` hasta el primer `show`**:
  arranca el hilo del servidor y se bloquea esperando en el canal. Una sesión que abre un repo
  y nunca pide un diagrama no debe pagar 97 MB por una ventana que no existe; hasta entonces
  el coste es del orden de 5 MB. Y si la sesión muere sin usar la pizarra, sale sin haber
  tocado la GPU.
- **La ventana nace robando el foco, y sin pedir permiso.** Al primer `show` la app sube de
  `Accessory` a `Regular` (`NSApplication.setActivationPolicy`) y llama a `activate()`: eso es
  lo que le da icono en el Dock y foco. **Precio aceptado:** macOS te roba el teclado a media
  frase si estabas en otra ventana, **una vez por sesión**. Se acepta porque lo que un permiso
  compraría ya está comprado y gratis: medido, el agente **anuncia** antes de dibujar, 8 de 8
  —*«Te lo dibujo en la pizarra»*, y llama—, y el consentimiento real está antes, en que el
  usuario la pidió o pegó él la línea que la manda. Ver la pregunta abierta de §11.1.
- **El foco se roba exactamente cuando la ventana nace o renace** —primer `show`, o primer
  `show` tras haberla cerrado el usuario— y **nunca en una actualización**. Un `show` sobre
  ventana abierta repinta en ~55 ms sin tocar el foco; saltar al frente cada vez que el agente
  retoca, mientras el usuario escribe en la terminal, es intolerable.
- **Cerrar la ventana oculta, no mata.** En `eframe` cerrar la ventana termina la aplicación,
  o sea que ⌘W se llevaría por delante el servidor MCP y dejaría al agente sin herramientas a
  media conversación. Y en macOS un event loop de `winit` **no se puede volver a arrancar** en
  el mismo proceso, así que esto no es preferencia sino obligación: si muriera, no habría
  segunda ventana nunca.
- **El título lleva el directorio de trabajo de la sesión** — `Pizarra — ai-render` —, que es
  lo que el usuario tiene en la cabeza cuando mira dos terminales.

### 6.3 Los dos estados vacíos, que son obligatorios

Son de naturaleza distinta:

- ***Pizarra vacía*** — tras `clear()`. Es un **estado en el que la ventana se queda**, no se
  oculta: `clear()` lo pide el agente, no el usuario, así que si la ventana se esfuma sola el
  usuario ve un parpadeo sin causa y pierde el sitio y el tamaño que le había dado. Si estorba,
  la cierra él, y por lo anterior eso no rompe nada.
- ***Sesión terminada*** — **un adiós de 2-3 segundos** y la ventana se cierra sola, con el
  reloj en el hilo del servidor. Dejarla en pantalla convertiría lo efímero en una promesa
  incumplida; el margen existe para el caso del segundo monitor, para que quien estuviera
  mirando se entere de por qué desaparece.

### 6.4 Varias sesiones

Dos Claude Code son dos procesos y **dos ventanas**. No es un enjambre gracias al arranque
diferido. Una ventana compartida exigiría demonio, descubrimiento y arbitraje.

---

## 7. Ciclo de vida

**Manda la sesión MCP, no la conversación.** Una sesión MCP no es una conversación: `/clear`
acaba la conversación y deja viva la sesión, así que **la pizarra le sobrevive**.

Las dos señales de muerte, en el orden en que llegan:

1. **`SIGINT`** — es lo primero que manda el host (`Sending SIGINT to MCP server process`). El
   binario tiene que **sobrevivirle lo justo** para cerrar su ventana.
2. **EOF en stdin** — lo que la especificación de MCP sobre stdio manda hacer al cliente.

Las dos se atienden en el hilo del servidor, por la regla del §2.

**Precio aceptado:** tras un `/clear` la ventana sigue enseñando el diagrama de la conversación
anterior hasta que el agente muestre otra cosa. Atarlo al hook `SessionEnd` está descartado:
sin canal, para que el hook le hable al proceso haría falta un fichero centinela vigilado —que
devuelve por la puerta de atrás el canal que este diseño elimina— y sólo existiría para quien
instale el plugin, así que el mismo producto tendría **dos comportamientos según cómo se
instaló**.

---

## 8. Cómo se usa de verdad: el disparo vive fuera del servidor

Esto no es un detalle de documentación: es el disparador principal del producto, y está fuera
del binario a sabiendas.

### 8.1 El agente no ofrece la pizarra por su cuenta. Medido

**0 de 36 turnos**, con cuatro redacciones de la descripción —la decidida; sin la cláusula de
la asimetría; sin ninguna norma de pedir permiso; y una que ataca de frente a dibujar en
ASCII— y 22 sesiones sobre un repo con capas de verdad. Lo que hace en su lugar es pintar el
grafo **en ASCII dentro de la respuesta**; y el ASCII no era la causa sino el síntoma:
apagándolo con un `CLAUDE.md` que lo prohíbe, la pizarra **siguió sin usarse** y se pasó a
prosa con listas.

**Lo que sí dispara son dos cosas, las dos al 100 %:**

| Canal | Resultado |
|---|---|
| Una línea en las instrucciones de proyecto | **8 intentos en 5 turnos** |
| Que el usuario nombre la herramienta | 9 en 7 |

Con la línea salió el caso protagonista sin pedirlo nadie: `Dependencias actuales`,
`Quién sabe de líneas hoy`, `Después · variante A`, `Después · variante B`.

**El modo degradado se acepta como el comportamiento correcto del producto:** instalado y sin
la línea, la pizarra no se usa jamás por iniciativa del agente. La alternativa —que la
descripción sostenga un mínimo por su cuenta— es literalmente lo que se probó cuatro veces.
**Precio escrito:** el producto tiene dos calidades de instalación, con línea y sin ella, y la
que funciona depende de que el usuario pegue algo.

*Nota de método, para quien repita la medición:* en Claude Code 2.1.228 **`--allowedTools` no
concede herramientas MCP en modo `-p`**, así que el instrumento es el recuento de `tool_use`
del historial, no el registro del servidor. Y el sujeto es el modelo de hoy: el resultado
accionable es *el texto no basta*, no una cifra.

### 8.2 La línea recomendada es el último paso de la instalación

**flipchart no escribe el `CLAUDE.md` de nadie** — la caja lleva `.mcp.json` y el binario y
nada más. Lo que hace es **pedírselo al usuario desde la documentación de instalación**, como
último paso y no como apéndice opcional. Es un consejo en un README, no un canal que el plugin
controle — y el consejo funciona donde la descripción no.

Es también el **canal de descubrimiento**: si el disparador fiable es que el usuario nombre la
pizarra, el usuario tiene que saber que existe, y el único canal del producto es la instalación.

Redacción de referencia:

```
Cuando me expliques una estructura o un cambio de estructura, dibújalo en la
pizarra con mcp__flipchart__show en vez de en ASCII o en prosa.
```

---

## 9. El idioma: Mermaid, y qué se promete de él

**El agente escribe Mermaid.** No hay protocolo propio, ni primitivas, ni `kind`, ni patch: un
protocolo semántico propio se implementaría igual sobre Mermaid —todas las puertas de entrada
del crate empiezan por un `&str`—, así que lo único que decidiría es *quién teclea ese texto*,
y costaría 738 tokens de peaje contra 271, con break-even a 4,8 retoques sobre la misma Vista.
Y el argumento de la independencia se da la vuelta: Mermaid lo leen Mermaid.js, mmdr,
mermaid-cli, Kroki, GitHub, GitLab y el CLI de Cursor; nuestro JSON lo leería **una**
implementación.

Lo que sobrevive del protocolo es **la prohibición** —nada de HTML, SVG, CSS, coordenadas,
tamaños, colores ni formas concretas—, defendida vaciando el IR y avisando (§3.2), y **las dos
reglas de honestidad** (§4.1).

**Una sola familia medida: el grafo dirigido.** `architecture`, `flow`, `dependency-graph` y
`class-diagram` son el mismo motor, y son el listón de lo que se promete — y también el ámbito
donde se impone la dirección (§3.3).

**Las demás familias no están prohibidas: están no probadas.** Con Mermaid como idioma se
dibujan solas y prohibirlas costaría escribir código para prohibirlas. Están **una vez**
probadas: ninguna de las 23 sale vacía; `radar-beta` no está implementada de verdad y la mata
la regla del nodo rastreable; `C4Context` pierde el título; `architecture-beta` pierde los
iconos. `wireframe` es la única excepción real — Mermaid no lo tiene.

Dato que conviene tener presente: el agente elige familias no medidas por su cuenta
(`sequenceDiagram`, 4 de 17 diagramas espontáneos).

---

## 10. Empaquetado, distribución e instalación

**Un plugin de Claude Code, y es el único camino de instalación.** El binario sigue siendo un
servidor MCP por stdio corriente —gratis por construcción— pero **no se documenta, no se
prueba y no se soporta** fuera de Claude Code.

**La caja lleva `.mcp.json` y el binario, y nada más:** sin `skills/`, sin `commands/`, sin
`hooks/`. Un skill de peaje cero es exactamente el que el modelo **no** puede invocar, así que
no puede ser dueño de nada; y un `/flipchart:*` en el menú es superficie de producto que
promete un mando sobre la pizarra que el usuario no tiene.

### 10.1 El vehículo: un zip verificado, y cero git en el cliente

- **El catálogo es un JSON servido por URL** — `source: "url"`, sobre
  `raw.githubusercontent.com` en `main`. URL **mutable y estable**: el usuario la teclea una
  vez y tiene que seguir sirviendo el catálogo bueno tres releases después. La rama `url` del
  host descarga el JSON a la caché y termina: **no invoca git en ningún punto.**
- **El plugin es un zip por HTTPS con `sha256` verificado** — `source: "archive"`, alojado como
  asset del release. URL **inmutable** (`…/releases/download/v0.1.0/flipchart-0.1.0.zip`): un
  digest pinneado apunta a un byte exacto, así que si la URL pudiera cambiar de contenido el
  pin no valdría nada.

Con eso **el cliente no ejecuta git ni una vez**, la integridad pasa a estar **verificada**
—el host comprueba el digest en cada descarga y **rechaza la instalación** si no casa, con el
error en primer plano—, y lo que queda en disco es un JSON de dos kilobytes más el binario
extraído en la caché versionada, que se poda sola (`.orphaned_at`, recogida a los 14 días).
Los tres extremos están **medidos**: ni un fichero `.git`, 2717 bytes de catálogo plano, y el
rechazo por digest con el esperado y el obtenido dentro del mensaje.

**El pico en disco es `2 × B`, no `B`:** medido, el `update` deja las dos versiones en la caché
—`…/flipchart/1.0.0` y `…/flipchart/2.0.0`, los dos binarios enteros— hasta que la poda pase.
Con un universal de ~84 MB eso son ~168 MB entre la actualización y la recogida.

Topes medidos, y hay que respetarlos porque no todos tienen válvula:

| | |
|---|---|
| Alta del catálogo | **10 s**, **5 MiB** |
| Descarga del archive | **120 s sin válvula**, **256 MiB** |
| Redirecciones | hasta 5, con política anti-SSRF revalidada en cada salto |
| Caudal desde el CDN de GitHub | 25,7–27,9 MB/s (42 MB ≈ 1,6 s: margen de ~75×) |

No hay ninguna variable de entorno de timeout para `archive`. La política de URL exige
`https://` y prohíbe loopback, link-local y hosts de metadatos de nube — así que **no hay atajo
local para probar esto: hay que alojar de verdad.**

### 10.2 La forma del zip, y el versionado

Dentro del zip: **`.claude-plugin/plugin.json`, `.mcp.json`, el Lanzador y el binario.**

- **Se empaqueta con Info-ZIP** (`zip`), que produce `version made by == 3` (Unix) con los
  modos `0755` intactos. **El `zip` de la CI es parte del contrato**: el host lee los atributos
  externos y hace `chmod(mode & 0o777)` cuando hay algún bit de ejecución, pero eso depende de
  quién empaquete. Nunca desde el Finder — mete `__MACOSX/` y `.DS_Store`.
- **El `sha256` se declara siempre en la entrada, y el generador lo verifica.** Medido: el
  esquema del host lo trata como **opcional**, y una entrada sin él **instala igual y sin
  comprobar nada**, sin aviso. Olvidarlo no rompe nada visible: sólo desarma en silencio la
  única defensa de integridad del vehículo.
- **`version` se declara**, y **la que manda es la de `plugin.json`, dentro del zip**, no la de
  la entrada del catálogo. Se declara y no se deja al digest porque la UI de `/plugin` hace
  `manifest.version ?? "unknown"`, y en la única pantalla donde el usuario juzga si se fía de
  un binario nativo sin notarizar la versión pondría `unknown`, siempre.
- **El `marketplace.json` se genera desde el tag del release y nunca se edita a mano.**
  `version`, `url` y `sha256` salen los tres del mismo sitio, así que olvidar el bump deja de
  ser posible. **Esto es requisito de corrección, no comodidad:** `/plugin update` descarga el
  zip entero *antes* de comparar identidades, así que un arreglo publicado sin subir la versión
  **se baja, se tira y no avisa** (`already at the latest version`).

De propina, medido: el asset de release **redirige fuera de origen** (302 a
`release-assets.githubusercontent.com`) y ahí el host deja caer las cabeceras heredadas del
catálogo — **un asset privado o autenticado es imposible por esta vía**. `raw.githubusercontent.com`
responde 200 sin redirección.

### 10.3 La CI del release

1. Compilar arm64 y x86_64.
2. `lipo` para el universal binary.
3. **Re-firmar ad-hoc** (`codesign -s -`) y verificar que la firma sobrevive al `lipo`: en
   Apple Silicon todo ejecutable necesita al menos firma ad-hoc para correr, Rust la genera al
   compilar, pero el universal se fabrica después. **Sin notarización** (ver §11.2).
4. Empaquetar con Info-ZIP.
5. Calcular el `sha256`.
6. Generar el `marketplace.json` desde el tag, subir el zip como asset del release y commitear
   el JSON en `main`.

**Nunca documentar «bájate el zip a mano»**: quien descarga con un navegador o Mail se lleva
`com.apple.quarantine`, y ese es el caso que Gatekeeper mata.

### 10.4 El arranque, y por qué el Lanzador no puede fallar

Lo que el host hace y lo que se ve cuando falla, medido con Claude Code 2.1.228:

- **30 000 ms duros** para el handshake, y **el plugin no puede ampliarlos** — ni por
  `settings.json` ni con `timeout` / `startupTimeout` / `initializationTimeout` en el
  `.mcp.json`. Sólo el usuario, con `MCP_TIMEOUT`. Corta a los 30,0 s y manda `SIGTERM` 2 s
  después.
- **Un arranque fallido veta el servidor 15 minutos**: se apunta en
  `~/.claude/mcp-needs-auth-cache.json` (TTL 900 000 ms) y **las sesiones siguientes ni lanzan
  el proceso**. Aplica a **cualquier** fallo, no sólo al timeout — un `command` que sale con
  error en 64 ms deja la misma marca. **No lo cura reinstalar el plugin**; lo único que borra
  la entrada es una conexión con éxito, que es justo lo que el veto impide. Y es un precio
  **exclusivo de los servidores stdio de plugin**.
- **Lo que ve el usuario son dos palabras:** `✘ failed` dentro de `/mcp`, y nada en la
  bienvenida. El stderr del `command` se captura pero va al log de depuración: **el mensaje de
  error que escribamos no llega**. El único texto con causa está en `claude mcp list`.
- **El arranque no bloquea el primer turno.** Con el lanzador tardando 10 s —muy por debajo del
  plazo— el turno corrió sin la herramienta. No hay que tardar 30 s para quedarse sin pizarra:
  basta tardar más que el usuario en escribir.

De ahí la restricción de primer nivel:

> **El Lanzador nunca falla.** Responde al handshake **siempre**, en milisegundos, haya binario
> o no, haya red o no, y sale con 0.

El motivo no es el timeout, que es una probabilidad: es el veto, que es un modo de fallo **que
el producto no puede reparar** — en la práctica *"espera un cuarto de hora o borra un fichero
de `~/.claude`"*.

**Cero código de red en el MVP.** No hay descarga ni clon: cuando el binario no sirve, el
Lanzador **lo cuenta y no intenta arreglarlo**. Peor caso aceptado: *reinstala el plugin*.

### 10.5 El Lanzador y el Servidor de aviso

El `command` del `.mcp.json` es un script de shell, en **bash 3.2 pelado**. Lo que hace:

1. `chmod +x` sobre el binario de su propio directorio (`${CLAUDE_PLUGIN_ROOT}`). Es
   **respaldo**, no mecanismo: el host preserva `0755` si el zip viene de Info-ZIP, pero nadie
   lo promete en su esquema.
2. `exec` del binario.
3. Si no puede, **no muere: se queda hablando él** como **Servidor de aviso**.

**El Servidor de aviso** es la cara del Lanzador cuando no hay Proceso de la pizarra al que
cederle el sitio. Es el mismo fichero: entra en un bucle leyendo stdin y contestando JSON-RPC a
mano —el transporte stdio de MCP es JSON por líneas, sin framing `Content-Length`— y atiende
`initialize`, la notificación `initialized`, `tools/list` y `tools/call`.

**Anuncia exactamente una herramienta, sin argumentos**, cuya *descripción* lleva el mensaje:
la pizarra no está operativa, esto es lo que se ha encontrado —binario ausente, sin permiso de
ejecución, o de otra arquitectura— y hay que reinstalar el plugin. El argumento es de
comportamiento: si anunciara las dos herramientas de verdad, el agente **intentaría dibujar y
fallaría**, y el usuario descubriría el problema en medio de otra cosa y con un turno gastado.
Con una herramienta que no se llama, el modelo sabe desde el primer momento que la pizarra no
está disponible y **no la ofrece**. Un canal roto que se anuncia es mejor que uno que se
descubre; y no anunciar nada es silencio absoluto, que es el único fallo que este producto no
puede permitirse.

**Por qué bash 3.2 y no Python, Perl o `jq`:**

- **`python3` no existe como dependencia**: `/usr/bin/python3` es un shim de las Command Line
  Tools, así que en una máquina sin Xcode no hay Python.
- `perl` y `ruby` están, pero Apple lleva años avisando de que los runtimes de scripting salen
  del sistema.
- `jq` **sí es de Apple** y está firmado, pero es reciente, y apoyarnos en él **le pondría suelo
  a macOS desde nuestro lado** — y ese suelo lo pone `eframe`/`winit`, no nosotros.
- Y la razón buena: sin `jq` hay que sacar el `id` del mensaje emparejando texto, lo cual es
  frágil **salvo que se haga imposible por construcción**. El único mensaje que puede traer un
  `"id"` anidado es un `tools/call`, y **con una sola herramienta sin argumentos** ningún
  mensaje que llegue a parsearse contiene jamás un `"id"` anidado: el primer `"id":` de la línea
  es siempre el de JSON-RPC. **La fragilidad no se mitiga, se elimina** — y por eso las dos
  decisiones (una herramienta, sin `jq`) encajan.

**Su caso protagonista, medido por ausencia:** el esquema de una entrada de marketplace **no
tiene ningún campo de plataforma** —ni `os`, ni `platform`, ni `arch`, ni `requires`—. Nada
impide que alguien en Linux haga `/plugin install flipchart`, se le extraiga un Mach-O y `exec`
devuelva `ENOEXEC`. Para ese usuario el Servidor de aviso no es un modo degradado: **es el
único mensaje que va a recibir en su vida.** Detrás van la cuarentena, un `chmod` que falle
sobre un sistema de ficheros de sólo lectura, una extracción a medias por disco lleno y el
borrado manual.

### 10.6 Instalación, actualización y desinstalación

**Un solo paso declarado**, más la línea recomendada:

```
/plugin marketplace add <url-al-marketplace.json>
/plugin install flipchart@<marketplace>
```

*(el nombre para el `install` es el del campo `name` del manifiesto, no el del repo)*

y después, del README: **pegar la línea del §8.2 en el `CLAUDE.md`.**

Mecánica verificada: `${CLAUDE_PLUGIN_DATA}` **está disponible desde el `.mcp.json`** —llega
expandida como argumento y como entorno, y el directorio se crea vacío antes de arrancar—
aunque el MVP no lo usa; el plugin se copia a
`~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/` con marcadores `.in_use/<pid>`, así
que **el host ya versiona por directorio y cuenta referencias por proceso**; y
`/plugin uninstall` **borra los datos del plugin** (`--keep-data` existe para evitarlo), así que
**el README no necesita ninguna línea de `rm -rf`**. Para desactivar sin desinstalar, `/plugin`.

### 10.7 Requisitos honestos

- **macOS** (Intel o Apple Silicon). Umbral: **macOS 12 o superior, provisional, a confirmar
  con el primer build** — lo fija lo que exijan `eframe`/`winit`, y no se puede saber sin
  construir. Preferimos eso a inventarnos un número que suene bien.
- Una versión de Claude Code con soporte de plugins.
- **No hay Node, no hay Python, no hay navegador, no hay toolchain de Rust.**

**Linux y Windows no se declaran imposibles: se declaran no probados y no prometidos.**
`objc2-app-kit` es lo que consigue Dock y foco, y ahí *aparecer una vez, robar el foco y nunca
más* hay que resolverlo de cero. Linux añade que **puede no haber pantalla** (SSH, contenedor,
WSL). Vuelven cuando el MVP exista.

---

## 11. Lo que la construcción tiene que resolver

### 11.1 Cuatro preguntas abiertas de implementación

Ninguna bloquea empezar. Se contestan mejor con código o con el dibujo delante que en una
conversación.

1. **¿Se puede tener Dock sin robar el teclado?** Subir de `Accessory` a `Regular` da *Dock y
   foco* como un paquete, pero son dos llamadas distintas: se puede mostrar la ventana sin
   activar la app (**`orderFrontRegardless`** en vez de `makeKeyAndOrderFront`). Si funciona,
   **desaparece el precio que acepta §6.2 sin pagar nada**; si no, la ventana aparece *detrás*
   del terminal y el agente dice que ha dibujado mientras el usuario no ve nada, que es peor.
2. ~~**Estructura de packages del repo.**~~ **CERRADA el 2026-09-03: un crate y un script de
   bash.** Un solo crate binario `flipchart` con módulos internos — `flipchart` (el estado y las
   Vistas), `server` (las dos herramientas de `rmcp`), `diagram` (la tubería de mmdr) y `mac`
   (App Nap y la política de activación) —, más el `launcher.sh` de §10.5. Ni workspace ni un
   crate por capa: Servidor y Visor comparten proceso, así que un límite de crate entre ellos
   sería una frontera sin nadie al otro lado. El paquete lleva además un `target` de librería,
   que es por donde entran los tests de integración; lo que se entrega sigue siendo **un solo
   fichero**.
3. **Cómo se testea un renderer visual** sin morir en snapshots. Hay una herramienta ya
   verificada: **`write_layout_dump` es público**, así que la PositionedScene se vuelca a JSON
   sin `--dumpLayout` y sin proceso hijo — es el instrumento para medir regresiones de layout.
4. **Dónde está el límite de tamaño de Vista y qué se dice al llegar.** En SVG el fallo a
   escala es **desorden**, no relaciones falsas, así que es calidad y no honestidad — por eso no
   se resolvió de tapadillo dentro del Límite honesto. Hoy **no hay barrera de tamaño en
   `show`**.

### 11.2 Tres riesgos con nombre, y su plan B

**(1) La cuarentena del zip. CERRADA, y a favor.** Medida el 2026-09-03 contra un release
alojado de verdad (`docs/research/15-la-cuarentena-medida-y-el-veredicto-del-vehiculo.md`).

Era lo único que podía cambiar el vehículo de instalación y no sólo el código: si el host
marcase `com.apple.quarantine` sobre lo que extrae del zip, **Gatekeeper mataría un binario
firmado ad-hoc y sin notarizar**. No lo marca. El fichero extraído llega **sin cuarentena**, con
modo **`100755`**, y **corre** (`rc=0`) —arrancado por el host a través del Lanzador, y antes de
su `chmod +x`—, igual en la extracción que en la copia versionada de la caché.

> **El plan B no se compra.** Ni notarizar (99 $/año contra un ataque que no ocurre) ni volver al
> binario dentro del repo del marketplace clonado, cuya factura de `B × (N + 2)` permanente sigue
> siendo peor.

Dos lecturas del log que no hay que confundir con lo contrario de lo que dicen: **`spctl -a`
responde `rejected`** —siempre lo hará sobre un ad-hoc sin notarizar— y el binario **corre
igual**, porque sin cuarentena la ejecución no consulta a Gatekeeper; y **`com.apple.provenance`
sí aparece** sobre todo lo instalado, pero no es la cuarentena y no impide nada.

Las otras cuatro comprobaciones del vehículo, contestadas en la misma corrida: `source: "url"`
por `https://raw.githubusercontent.com` **sin un solo fichero `.git`** en el
`CLAUDE_CONFIG_DIR`; el digest cambiado a mano **rechaza en primer plano** con el esperado y el
obtenido en el mensaje; el modo real es `100755`, así que el `chmod +x` del Lanzador es respaldo
y no mecanismo; y el `update` con `version` declarada dentro del zip **trae la versión nueva**
(`updated from 1.0.0 to 2.0.0`).

**(2) La regla del nodo rastreable. CERRADA, y en contra.** Medida el 2026-09-04 contra el
banco de 63 con `flipchart check`
(`docs/research/16-el-nodo-rastreable-medido.md`, arnés en el prototipo 23).

**Doce falsos positivos sobre los 42 casos correctos**, nueve de ellos de esta regla, y entre
ellos todo `stateDiagram-v2` que use `[*]` más `journey`, `quadrantChart`, `packet-beta` y
`treemap-beta` enteros: mmdr fabrica ids sintéticos legítimos (`__start_root__`,
`<familia>_<n>`) y el careo contra el fuente no los distingue de un apócrifo. Perdonarlos por
la forma del id **libera `radar-beta`**, que es lo que la regla mejor mataba, así que la
acotación pide justo la lista de familias que la regla presumía no necesitar. Compra lo
prometido —caza 5 de los 6 inventos, mata `radar-beta` sin lista— y no alcanza para pagarlo.
De rebote, la regla de la asimetría se lleva los otros cuatro falsos positivos, y uno es un
defecto plano: `class X` a secas **es** una declaración y `declares_itself` no lo sabe.

> **Se ejecuta el plan B:** vuelve a estar abierto **qué se hace con los seis inventos**, y
> nada más. El §9 del informe deja los datos para elegir; §4.1 queda marcado.

Lo que se sospechaba, y por qué había que medirlo antes:

Es la pata central del Límite honesto y **está decidida sobre un mecanismo leído, no ejecutado**.
Si mmdr fabrica algún `id` sintético legítimo —un `subgraph` sin id declarado, algo interno de
`classDiagram`, un participante implícito—, la regla convierte Mermaid válido en rechazo.

**La asimetría del riesgo es lo que obliga a medirla primero: un falso positivo es peor que la
enfermedad que cura.** Un invento hace que el usuario vea de más una vez; un falso positivo hace
que la pizarra **no dibuje nunca un tipo de diagrama entero** — y el agente no insiste: se pasa a
prosa y no lo dice.

Cómo, sin arnés nuevo: el **prototipo 21**
(`docs/research/prototipos/21-lo-que-mmdr-traga`) compila, corre la tubería entera y trae **63
casos con el parser de Mermaid 11.12 sobre jsdom como juez de validez**. Hay que ver tres cosas:
**falsos positivos entre los 42 casos correctos** (el número que decide si la regla vive); **que
caza los seis inventos**, incluido el nodo `flowchart` de la cabecera sin dirección; y falsos
positivos fuera del banco (`subgraph` sin id, ids con caracteres que el tokenizado parta
distinto, y `classDiagram` con relaciones a clases no declaradas, donde esta regla se solapa con
la de la asimetría y hay que ver cuál salta primero).

> **Plan B:** si la regla se cae, lo que vuelve a estar abierto es **qué se hace con los seis
> inventos**, no el reparto entero de §4.

**(3) El HTML en las etiquetas. Medido y sin desenlace.**

Aparece en **15 de 17** diagramas espontáneos, así que no es un caso raro: es lo normal. Y no lo
tapa nada de §3.2, porque **viaja dentro del texto de la etiqueta**, no en un campo:

| Constructo | Qué hace mmdr | Qué acaba viendo el usuario |
|---|---|---|
| `<br/>` | **lo interpreta** — parte la etiqueta en dos `<text>` | exactamente lo que el agente quería |
| `<b>…</b>` | **lo escapa** | `<b>recolocacion</b>` literal dentro de la caja |

Ninguno de los dos enciende el aviso ni el rechazo. Y el `<b>` produce **basura visible que nadie
puede corregir**: el agente es ciego y el usuario no lee el fuente.

> **No hay plan B porque no hay decisión que revertir.** Las tres salidas siguen sobre la mesa
> —rechazo, aviso o convivencia— y se decide **con el dibujo delante**. Dato para la balanza: la
> descripción de las herramientas cubre hoy un fallo que no ocurre (0/17) e ignora éste, que
> ocurre en 15/17.

### 11.3 La checklist del primer día

1. ~~**La cuarentena del zip** (riesgo 1)~~. **Hecho el 2026-09-03**: no hay cuarentena y el
   vehículo del §10.1 se queda como está. El empaquetado puede empezar.
2. ~~**La regla del nodo rastreable contra el banco de 63 casos** (riesgo 2)~~. **Hecho el
   2026-09-04**: 12 falsos positivos sobre los 42 correctos y la regla se cae. El Límite
   honesto **no está dado por bueno**: antes de empaquetar hay que decidir qué se hace con
   los seis inventos (§4.1, y `docs/research/16-el-nodo-rastreable-medido.md` §9).
3. **El tamaño del universal binary.** Referencia: el binario suelto de mmdr son 6,9 MB sin
   runtime, y falta `eframe` + `winit` + `resvg` + `rmcp` encima, en dos arquitecturas. Con el
   tope de 256 MiB y el caudal medido hay margen de sobra, pero conviene tener el número real
   antes de escribir el `marketplace.json`.

### 11.4 Límites conocidos, escritos para que no se descubran

- **No hay tope de tiempo.** Un `compute_layout` que tarde un minuto **no se puede abortar**: es
  una llamada en el mismo hilo, y un hilo de Rust no se mata. Cortarlo pediría un proceso hijo,
  que está descartado. **Un diagrama patológico puede colgar el turno.**
- **No hay barrera de tamaño en `show`** (§11.1, pregunta 4).
- **Los grupos se entregan tal como mmdr los dibuje** (§3.5).
- **Tras un `/clear` la ventana enseña el diagrama anterior** (§7).
- **Volver atrás en el rotafolio es lineal y a ciegas** (§6.1).
- **La primera ventana roba el foco**, una vez por sesión (§6.2, y §11.1 pregunta 1).
- **Sin la línea del `CLAUDE.md`, la pizarra no se usa jamás** por iniciativa del agente (§8.1).
- **La prohibición de píxeles es limpiada-y-avisada, no imposible** (§3.2).
- **Se pierden sin aviso** las pistas de longitud de arista, `cssClass`/`link` de `classDiagram`,
  cinco fugas de estructura en familias no probadas y dos deformaciones (§4.5).
- **El nombre `flipchart` es provisional**, y el del repo (`ai-render`) no dice nada. Renombrar
  es trivial y no es una decisión del MVP.

---

## 12. Fuera del MVP

Nada de esto vuelve al avanzar la construcción: vuelve, si vuelve, como esfuerzo nuevo.

**Del producto:** cuentas, cloud, colaboración, persistencia, base de datos, historial,
exportación, marketplace público, múltiples renderers simultáneos, IA propia para generar
layouts, sincronización entre máquinas. Y **edición del diagrama por el usuario** — el usuario
observa.

**De la superficie:** el **navegador como superficie de entrega y con él todo HTTP** (ventana de
navegador, servidor local, puerto, token de sesión, SSE, visor en HTML/JS); el **puente a MCP
Apps** (`ui://` + `postMessage`), quemado a sabiendas porque una ventana nativa no entra en un
iframe sandboxeado —se pierde el visor gratis en Claude Desktop, Claude web, Cursor y VS Code
Copilot, y se acepta porque el host protagonista es Claude Code, que no renderiza MCP Apps—;
**la integración embebida en Claude Code**, si resulta que hoy no existe el mecanismo; **el
camino de la terminal entero** (el ruteo de termaid es irrastreable a 3 nodos con herencia y
pierde el 28 % de las relaciones a 19, y la patología es de celdas de carácter, imposible en
SVG); y **pintar el diagrama en la conversación** (ASCII como tool result), que no es cuestión de
gusto: no hay canal del servidor MCP a la pantalla —`stdout` es JSON-RPC y `/dev/tty` pelea con
el TUI—, así que el dibujo entraría en el contexto del modelo, ~393 tokens por render.

**De la arquitectura:** **el Layout Engine y la Drawing Surface como piezas sustituibles detrás
de una interfaz.** Era el corazón arquitectónico del handoff y sobrevivió a tres giros; muere por
**falta de aspirante**. Siguen siendo dos *etapas* y el glosario las mantiene como términos; lo
que se cierra es tratarlas como **intercambiables**. Con ello, **cambiar de motor de Mermaid** —y
en concreto [merman](https://github.com/Latias94/merman), que existe, es MIT/Apache-2.0, declara
paridad con `mermaid@11.17.2` y la usa Zed— queda fuera **por decisión, no por ignorancia**: el
MVP tiene un motor, y medir un segundo abre una puerta que no necesita. Vuelve si mmdr se
abandona o si el MVP existe y los grupos siguen doliendo. Y **arreglar los grupos de mmdr
nosotros** (sus issues #140 y #136) nos volvería mantenedores de un motor de layout.

**De la distribución:** **todo camino de instalación que no sea el plugin** —paquete npm,
registro manual en Cursor, VS Code Copilot o Claude Desktop—; **`brew` como vía principal**, que
vuelve cuando el MVP exista; **`experimental.binaries`**, el aprovisionamiento de binarios del
host, que hace *exactamente* lo que flipchart necesita (ficheros pinneados por sha256 traídos a
`bin/` en instalación, digest verificado, modo `0755`, caché compartida) y **no está disponible**:
además de un feature gate, sale por la puerta si el marketplace no está en el `Set` de nombres
reservados a Anthropic. Sólo vuelve si Anthropic lo abre.

**Del ciclo de vida:** **atar la muerte de la pizarra al final de la conversación** (hook
`SessionEnd`) — ver §7.

**Del alcance del producto:** **que flipchart escriba las instrucciones de proyecto del usuario.**
Lo que sí se hace es *recomendar* la línea desde la documentación de instalación (§8.2): es un
consejo, no un canal que el plugin controle.
