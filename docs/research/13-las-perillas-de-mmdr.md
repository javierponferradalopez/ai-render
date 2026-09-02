# Las perillas de mmdr, giradas

Barrido disparado el 2026-09-02 por [¿Curan las perillas los grupos de mmdr?][25],
que [El stack de rendering][8] dejó como techo del esfuerzo: *girar perillas, no
arreglar motores ajenos ni cambiar de motor*.

**Curan, y la que cura es la que el ticket decía que no había probado nadie: la
dirección.** El sospechoso principal —`preferredAspectRatio`— resulta inocente y
además dañino, y la mitad del `FlowchartLayoutConfig` no está conectada al motor.

## Nota sobre el método

Todo lo que sigue está **compilado y ejecutado**, no leído: macOS 26.6.2 arm64,
`rustc` 1.98.0, crate `mermaid-rs-renderer` **`=0.3.1`** descargado de crates.io
(sha256 `6cb5b469…3f117`), con `default-features = false, features = ["png"]`.
No interviene el binario `mmdr` en ninguna medición.

Los casos son **los mismos dieciséis** que midieron research 07 y research 08,
regenerados con los scripts del [prototipo 12][p12] (`ast` sobre `termaid` 0.8.0 y
sobre el `asyncio` de Python 3.9.6), más `arch.mmd` del [prototipo 13][p13]. Sólo
cambian las perillas. Scripts, métricas y renders: [prototipo 20][p20].

Las métricas son las tres patologías de research 07 trasladadas a geometría
—perdidas, cruces, sueltas— más tres que este barrido necesitaba y que se leen del
mismo volcado de layout:

- **rodeos** — aristas cuya polilínea se sale del corredor de sus dos extremos por
  más de 40 px, que es la queja literal de research 08 §6;
- **desvío** — el peor cociente entre la longitud de la polilínea y la recta que
  une los centros de sus nodos;
- **vacío** — fracción de bandas horizontales de 16 px sin un solo nodo dentro, o
  sea la *banda vacía en el medio*.

## 1. La API por etapas compila

El *"leído, no ejecutado"* que [El stack de rendering][8] dejó abierto y que este
ticket heredaba **queda cerrado, y a favor**. En el crate publicado son públicos y
compilan tal cual los describía la decisión:

```rust
let mut parsed = parse_mermaid_strict(&source)?;   // ParseError tipado
parsed.graph.direction = Direction::LeftRight;      // el IR es nuestro
let layout = compute_layout(&parsed.graph, &theme, &cfg);
let svg = render_svg(&layout, &theme, &cfg);
```

Tres cosas más que el mapa no sabía y que abaratan el spec:

- **`LayoutConfig` y `Theme` son `Serialize + Deserialize`.** Los valores que
  flipchart fije se pueden escribir como datos y mezclar sobre el default, sin
  construir la struct campo a campo.
- **`layout_dump::write_layout_dump(path, &layout, &graph)` es público.** La
  `PositionedScene` se puede volcar a JSON sin `--dumpLayout` y sin proceso hijo,
  que es justo lo que #8 decidió no usar. No hace falta para el MVP —al Visor va el
  SVG—, pero es la herramienta con la que se mide cualquier regresión de layout.
- **El `Layout` no expone si un nodo fue declarado o inventado**, así que el agujero
  que [La API de las dos herramientas MCP][11] escribió para la regla de la
  asimetría **sigue abierto**: hay que resolverlo sobre el `Graph`, no sobre el
  `Layout`. Ver §7.

## 2. El "hoy" del ticket es `Theme::mermaid_default`

La primera medición no era del barrido sino de la calibración, y corrige un
supuesto: research 08 midió con el binario, cuyo `Config::default()` usa
`Theme::mermaid_default()` (fuente 16 px), mientras que `RenderOptions::default()`
—la puerta que usaría flipchart sin pensarlo— usa `Theme::modern()` (fuente 14 px).
No son el mismo dibujo.

| caso | research 08 (binario) | `mermaid_default` | `modern` |
|---|---|---|---|
| `arch.mmd` | 800×1100 | **804×1104** | 778×1077 |
| `termaid` 6 nodos | 574×780 | **575×780** | 556×716 |
| `asyncio` 3 nodos | 602×366 | **602×367** | 576×336 |

Reproducido. La API por etapas y el binario dan el mismo dibujo, y todo lo que
sigue se mide contra `mermaid_default` como estado de partida.

## 3. El barrido: 90 configuraciones, una perilla cada vez

Sobre `arch.mmd`, moviendo un solo eje desde el estado de partida. 5,4 s las
noventa. Resultado ordenado por lo que importa (rodeos, luego desvío):

| config | rodeos | desvío | vacío | tamaño | orden de las capas |
|---|---:|---:|---:|---|---|
| **`dir-LR`** | **0** | **1,00** | **0,28** | 2137×409 | API, Application, Domain, Infrastructure |
| `dir-RL` | 0 | 1,00 | 0,28 | 2137×409 | ídem |
| `pady-40` | 1 | 3,23 | 0,42 | 804×1454 | API, **Infrastructure**, Application, Domain |
| `theme-modern` | 1 | 3,61 | 0,57 | 778×1077 | ídem |
| **`hoy`** | 1 | 3,92 | 0,54 | 804×1104 | ídem |
| `dir-BT` | 1 | 3,92 | 0,58 | 804×1104 | Domain, Application, Infrastructure, API |
| `ar-1.6` | 1 | 7,16 | 0,54 | 1752×1104 | ídem |
| `ar-3.0` | 1 | **10,70** | 0,54 | 3296×1104 | ídem |
| `font-10` | **2** | 3,56 | 0,57 | 630×989 | ídem |

Sesenta y tantas de las noventa producen un layout **idéntico byte a byte** al de
hoy. Ver §6.

## 4. `preferredAspectRatio` es inocente, y encima daña

Era el sospechoso principal del ticket: *"mmdr saca 800×1100 y el dibujo bueno es
1540×790"*. **No reordena nada.** El ancho del contenido se queda donde estaba y
lo único que hace la perilla es estirar el lienzo hasta cuadrar la razón pedida,
inflando las cajas de los grupos con hueco vacío y alargando las aristas: el
desvío sube de 3,92× a 10,70× al pedir 3,0.

Se ve mejor en el sentido contrario, ya con LR aplicado
(`renders/arch-lr-aspect13.png`): pidiendo 1,3 sobre un dibujo de 2137×409, el
ancho **sigue siendo 2137** y lo que crece es el alto, hasta 1621. La perilla no
pliega; sólo añade aire.

| perilla | efecto sobre el trazado |
|---|---|
| `preferred_aspect_ratio` | ninguno; estira el lienzo y empeora el desvío |
| `node_spacing`, `rank_spacing` | ±60 px de lienzo, mismo trazado |
| `node_padding_x/y` | ídem |
| `max_label_width_chars` | nada por debajo de 22 (ningún label llega) |
| `Theme` (los cinco) | sólo color, salvo por el tamaño de fuente |
| `font_size` | cambia el tamaño de las cajas y con ello el lienzo; a 10 px **empeora** |

## 5. La dirección es la perilla

`arch.mmd` con `Direction::LeftRight` impuesta después del parse: **rodeos 1 → 0,
desvío 3,92 → 1,00, banda vacía 54 % → 28 %**, y las siete aristas trazadas de
izquierda a derecha sin una sola vuelta (`renders/arch-hoy.png` frente a
`renders/arch-lr.png`). Los tres criterios de éxito que fijó el ticket, cumplidos.

Y no es un apaño para el caso protagonista: **mmdr respeta la dirección también en
`classDiagram`**, así que se puede medir en los dieciséis casos de research 08.

| caso | n | hoy: rodeos / desvío | LR: rodeos / desvío |
|---|---:|---|---|
| `arch` | 8 | 1 / 3,92 | **0 / 1,00** |
| `termaid-n20` | 19 | 1 / 3,15 | **0 / 1,14** |
| `termaid-n17` | 17 | 1 / 1,90 | **0 / 1,12** |
| `termaid-n14` | 14 | 3 / 1,50 | **0 / 1,16** |
| `termaid-n12` | 12 | 0 / 1,37 | 0 / 1,17 |
| `termaid-n10` | 10 | 0 / 1,26 | 0 / 1,15 |
| `termaid-n08` | 8 | 0 / 1,17 | 0 / 1,18 |
| `termaid-n07` | 7 | 0 / 1,13 | **0 / 1,00** |
| `termaid-n05` | 5 | **0 / 1,00** | 1 / 1,09 |
| los otros siete | 3–6 | 0 / 1,00 | 0 / 1,00 |

**Gana en dieciséis de diecisiete y pierde en uno** (`termaid-n05`, que se lleva un
rodeo y 0,09 de desvío). Cero aristas perdidas y cero cajas sueltas en las dos
columnas: ninguna dirección miente, que era la condición previa.

### El precio: la hoja se encoge

LR endereza el trazado a cambio de tumbar el dibujo. `arch` queda en 2137×409, una
razón de **5,23:1**. En una hoja de 1200×800 —y con la regla *encoger nunca
agrandar* que fijó [Cómo se ven N vistas en una ventana][19]— eso baja el zoom del
**72 % al 56 %**. En los demás casos el precio va de 0 a −9 puntos, y en uno
(`asyncio-n07`) LR gana 7.

Es el peor número de la decisión y se acepta a sabiendas: un dibujo pequeño que se
sigue con la vista bate a uno grande en el que hay que seguir una arista con el
dedo.

## 6. La mitad del `FlowchartLayoutConfig` no está conectada

El hallazgo que no buscaba nadie. `FlowchartLayoutConfig` publica tres subestructuras
con veintitantos campos —`objective`, `routing`, `auto_spacing`— y **dos de ellas no
cambian un solo byte del layout**, ni en `arch.mmd` ni en los dos flowcharts grandes
del propio crate (`flowchart_mega_crosslane_subgraphs`, `flowchart_nested_clusters`,
decenas de nodos con subgrafos cruzados):

| perilla | ¿mueve algo? |
|---|---|
| `objective.enabled = false` | **no** |
| `objective.max_aspect_ratio` (1,2 … 20) | **no** |
| `objective.wrap_min_groups` (2 … 99) | **no** |
| `objective.wrap_main_gap_scale`, `wrap_cross_gap_scale` | **no** |
| `objective.edge_relax_passes`, `backedge_cross_weight` | **no** |
| `routing.enable_grid_router = false` | **no** |
| `routing.grid_cell` (4 … 64), `turn_penalty`, `occupancy_weight` | **no** |
| `routing.snap_ports_to_grid = false` | **no** |
| `order_passes` (1 … 32) | sí |
| `port_side_bias` (−1 … 1) | sí (mueve el fichero; no las métricas de `arch`) |
| `auto_spacing.enabled = false` | sí, sólo en los grandes |

`wrap_min_groups: 4` parecía el culpable perfecto —`arch.mmd` tiene exactamente
cuatro `subgraph`— y no hace nada. Que `enable_grid_router: false` tampoco cambie
nada dice que el router configurable **no llega al motor**. Son campos de
escaparate.

Lo que esto significa para el mapa: **el techo del esfuerzo que fijó #8 era más
bajo de lo que #8 creía.** No es que las perillas se hayan girado y no basten; es
que la mayoría no está enchufada, y la única que mueve el dibujo de verdad es una
que ni siquiera es de mmdr — la dirección, que sale del IR.

## 7. Lo que no cubre la imposición, y una fuga nueva

**Las otras familias.** Forzar `LeftRight` es **no-op** en `sequenceDiagram` y
`mindmap` (sus parsers fijan su propia dirección), y **cambia** `stateDiagram-v2`
(124×292 → 433×91) y `erDiagram` (326×196 → 398×187) sin romper ninguno. Como la
decisión 4 del mapa las declara *no probadas*, la imposición se limita a `flowchart`
y `classDiagram`, que es lo medido aquí.

**mmdr no distingue una dirección escrita de una ausente.** El parser inicializa
`graph.direction = Direction::TopDown` y la pisa si encuentra cabecera, así que
después del parse `flowchart TB` y `flowchart` son indistinguibles. Es el mismo
agujero que [La API de las dos herramientas MCP][11] escribió para la regla de la
asimetría, y se tapa igual: mirando el fuente antes de entregárselo a mmdr.

**Y una fuga que no es de este ticket:** un `flowchart` **sin dirección** hace que
mmdr **invente un nodo llamado `flowchart`**, con esa etiqueta, y lo dibuje.

```
flowchart                    ->  nodos: A[Uno], B[Dos], flowchart["flowchart"]
  A[Uno] --> B[Dos]
```

`parse_mermaid_strict` lo acepta con éxito, así que `render_strict` no lo tapa. Y
**la regla de la asimetría tampoco lo pilla**: el nodo inventado trae etiqueta, no
es un id desnudo. Es una caja que nadie escribió, dibujada sin aviso — exactamente
la mentira que el producto existe para no dibujar, entrando por una puerta que el
mapa no tenía vigilada. Va a [¿Qué traga mmdr sin dibujar?][27].

## 8. Lo que no se ha medido

- **Los grupos anidados.** `arch.mmd` tiene cuatro `subgraph` planos. La decisión
  de partida 5 quería `Group` anidable, y aquí no se ha probado.
- **Diagramas por encima de 20 nodos**, que siguen siendo la niebla *"qué pasa con
  diagramas grandes"* del mapa.
- **La calidad tipográfica** de cada tema: se ha medido lo que cambia el layout
  (el tamaño de fuente), no lo que cambia el gusto.

[25]: https://github.com/javierponferradalopez/ai-render/issues/25
[8]: https://github.com/javierponferradalopez/ai-render/issues/8
[11]: https://github.com/javierponferradalopez/ai-render/issues/11
[19]: https://github.com/javierponferradalopez/ai-render/issues/19
[27]: https://github.com/javierponferradalopez/ai-render/issues/27
[p12]: ./prototipos/12-limite-de-termaid/
[p13]: ./prototipos/13-mmdr-frente-a-termaid/
[p20]: ./prototipos/20-las-perillas-de-mmdr/
