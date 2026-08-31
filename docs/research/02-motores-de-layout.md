# Motores de layout para grafos dirigidos

Investigación para [issues/02-motores-de-layout.md](../issues/02-motores-de-layout.md).

> **Cómo leer este informe.** Casi todo lo que sigue está **verificado
> ejecutando los tres motores** contra escenas construidas para este caso de uso,
> no leído en su documentación. Los números son míos y reproducibles. Lo que es
> lectura de documentación o inferencia va marcado como *(inferido)*.

## Método

Banco de pruebas propio en Node v24.11.0 (macOS arm64), con las versiones que hay
hoy en npm:

| paquete | versión | fecha de publicación |
|---|---|---|
| `@dagrejs/dagre` | 3.1.1 | 2026-08-08 |
| `elkjs` | 0.12.0 | 2026-07-17 |
| `@hpcc-js/wasm-graphviz` | 1.28.0 (Graphviz 15.1.0) | 2026-07-24 |

Dos escenas:

- **Escena sintética** — 10/14/50/200 nodos de tamaño aleatorio repartidos en 6
  grupos (3 raíz + 3 anidados), con aristas encadenadas, saltos entre grupos y
  algunas hacia atrás. Sirve para estresar.
- **Escena realista** — el caso protagonista del ticket:
  `src/{domain/{model,services}, infra/{db,http}, ui}` con 10 clases de altura
  variable y 12 aristas `depends`/`extends`. **Tres niveles de anidamiento.**

Se midió: contención, anidamiento de cajas, solapes, área y proporción del
lienzo, cruces de aristas, latencia, y desplazamiento de los nodos al añadir
material. Todos los resultados se renderizaron a SVG y se inspeccionaron
visualmente — varias conclusiones de este informe sólo aparecen al mirar el
dibujo, no en las métricas.

---

## 1. dagre (`@dagrejs/dagre` 3.1.1)

**Lo primero: su fama de abandonado es falsa, pero apunta al paquete equivocado.**
Hay dos paquetes en npm y sólo uno está vivo *(verificado en el registro de npm)*:

- `dagre` — última versión **0.8.5, publicada en 2019-12-03**. Muerto. Sigue
  recibiendo 2,8 M de descargas semanales.
- `@dagrejs/dagre` — **3.1.1 el 2026-08-08**, con 2.0.4, 3.0.0 y 3.1.0 también en
  2026. Mantenido. 4,2 M descargas/semana. MIT.

Si se usa dagre, es el segundo. La confusión es el riesgo real aquí.

**Grupos anidados — funciona mejor de lo que dice su reputación, pero se rompe
por los bordes.** En la escena realista de tres niveles: **10/10 nodos dentro de
su grupo y 7/7 cajas hijas dentro de su padre**, sin solapes entre grupos
hermanos. Repetido con 14 y 30 nodos sintéticos: cero violaciones. Esto contradice
la creencia de que dagre "no sabe hacer clusters"; con dos y tres niveles, los
sabe hacer.

Pero hay dos fallos duros, ambos reproducidos:

1. **Una arista que toque un nodo-grupo lanza excepción.**
   `setEdge('b', 'grp')` → `TypeError: Cannot set properties of undefined
   (setting 'rank')`. Es el issue [#238](https://github.com/dagrejs/dagre/issues/238),
   abierto, y **sigue vivo en 3.1.1**. Para nuestro `kind: contains` o para
   "este módulo depende de esa capa entera" no hay camino: hay que expandir la
   arista a todos los hijos a mano.
2. **Revienta con grafos compound grandes.** Con 200 nodos y grupos:
   `Error: Not possible to find intersection inside of the rectangle`. No es
   lentitud, es una excepción. Con 50 nodos aguanta.

**Tamaños variables:** los toma tal cual de `width`/`height`. Sin problema.

**Enrutado:** puntos de polilínea con codos suaves, ni ortogonal ni spline real —
el consumidor decide cómo unirlos. No hay opción de enrutado ortogonal.

**Compacidad — su punto débil.** En la escena sintética de 14 nodos produjo un
lienzo de **1594×2780 px (4,43 M px², proporción 0,57)** frente a los 0,66 M px²
de Graphviz: **6,7 veces más área** y una cinta alta y estrecha, lo peor posible
para una pantalla. Es la manifestación visible del issue
[#117](https://github.com/dagrejs/dagre/issues/117) (los clusters anidados se
estiran en vertical). En la escena realista la diferencia se estrecha mucho
(1,41 M px², proporción 0,87) y el dibujo resultante es, de hecho, **el más
legible de los tres** para ese grafo.

**Latencia** (media de 10 pasadas, escena sintética con grupos): 10 nodos
**9,1 ms**, 50 nodos **145 ms**, 200 nodos **excepción**. Sorprendentemente lento:
es el más lento de los tres a 50 nodos, tres veces peor que elkjs y veinte veces
peor que Graphviz.

**Peso:** `dagre.min.js` **47 KB en bruto, 16 KB gzip**. Imbatible. Node y
navegador, sin WASM ni worker.

**Salud:** MIT. 5779 ★, **174 issues abiertas**, último push 2026-08-08.
`dagre-d3` (el renderer) sí está moribundo: último release en 2017, 217 issues
abiertas — irrelevante para nosotros, sólo necesitamos el motor.

---

## 2. elkjs 0.12.0

**Grupos anidados — el soporte más completo y el único diseñado para esto.** La
jerarquía es la estructura nativa (`children` recursivos), no un añadido.
Con `elk.hierarchyHandling: INCLUDE_CHILDREN` en la raíz:

- Escena realista de 3 niveles: **10/10 nodos contenidos, 7/7 cajas anidadas**,
  sin solapes.
- **Aristas contra un grupo: funcionan.** `sources:['b'], targets:['grp']`
  devuelve layout correcto sin excepción — a diferencia de dagre. Es lo que
  necesita `kind: contains`.

**Tamaños variables:** `width`/`height` por nodo, respetados literalmente.

**Enrutado — el mejor del lote, y el único con ortogonal de verdad.**
`elk.edgeRouting` acepta `ORTHOGONAL`, `POLYLINE` y `SPLINES`. Medido sobre 14
nodos / 19 aristas: **3 cruces** (dagre 4, Graphviz 8-9) y sólo **40 puntos de
inflexión** en modo ortogonal — el trazado más limpio. Devuelve `sections` con
`startPoint`, `bendPoints`, `endPoint` e `incomingShape`/`outgoingShape`, que es
exactamente la forma que quiere un `PositionedScene`.

**Estabilidad incremental — aquí está su ventaja decisiva, y es real.** ELK es el
único de los tres con modo interactivo. Activando a la vez:

```
elk.layered.cycleBreaking.strategy: INTERACTIVE
elk.layered.layering.strategy:      INTERACTIVE
elk.layered.crossingMinimization.semiInteractive: true
```

y sembrando cada nodo con `elk.position: (x,y)` del layout anterior, en la escena
sintética densa de 12 nodos al añadir uno:

| | desplazamiento máx. | mediana | pares reordenados en Y |
|---|---|---|---|
| elkjs por defecto | 914 px | 509 px | **34 / 66** |
| **elkjs + INTERACTIVE** | **156 px** | **12 px** | **2 / 66** |
| dagre | 751 px | 597 px | 0 / 66 |
| Graphviz | 699 px | 506 px | 32 / 66 |

La mediana cae de 509 px a **12 px** y el reordenado prácticamente desaparece.
Ningún otro motor ofrece nada equivalente. **Pero** — y esto también está medido —
en la **escena realista** el modo INTERACTIVE **no cambió nada** (156 px máx. con
y sin él, 0/45 reordenados en ambos casos): con grafos poco densos el orden ya
venía forzado por las aristas. La ventaja se cobra sólo cuando el grafo es denso
y ambiguo.

**Los tres motores son deterministas**: dos ejecuciones del mismo grafo dan
desplazamiento 0. Ninguno introduce aleatoriedad.

**Latencia:** 10 nodos **10,0 ms**, 50 nodos **46 ms**, 200 nodos **171 ms**. El
único que escala sin romperse, y **3× más rápido que dagre a 50 nodos**.

**Peso — el punto negro.** `elk.bundled.js` pesa **1571 KB en bruto, 455 KB
gzip**. Es Java transpilado con GWT y se nota. En Node da igual; en una página
autocontenida (que es lo que exige la decisión del ticket 01) son 455 KB de
descarga sólo para el layout.

**La trampa de integración que hay que documentar.** Las aristas jerárquicas
vuelven con un campo **`container`**, y sus coordenadas son **relativas a ese
contenedor, no a la raíz**. Verificado: la arista `UserRepo → User` vuelve como
`{sections:[{startPoint:{x:531,y:166}, …}], container:"src"}`. Al implementarlo
me equivoqué en esto y el resultado fueron aristas colgando en el aire y saliendo
del lienzo — hasta que sumé el offset del contenedor. dagre y Graphviz devuelven
coordenadas absolutas y no tienen esta clase de error. Es media hora de trabajo,
pero es media hora que sólo se paga con ELK.

**Calidad visual real — peor de lo que prometen sus métricas.** Inspeccionando el
render de la escena realista: elkjs saca las aristas entre jerarquías **por fuera
de las cajas de los contenedores**, rodeando `src/` por los laterales. Es
correcto según sus propias reglas y pésimo de leer. También es el más alto de los
tres. Las métricas de cruces le favorecen; el dibujo, no.

**Salud:** 2739 ★, 98 issues abiertas, 0.12.0 en 2026-07-17, con 0.11.x y 0.10.x
a lo largo de 2025-2026. **7,0 M descargas/semana**, el más usado de los tres.
Mantenimiento sano y con cadencia.

**Licencia: `EPL-2.0 OR GPL-3.0-or-later`** *(verificado en package.json y
LICENSE.md)*. EPL-2.0 es copyleft débil por fichero: consumir la librería sin
tocarla no contamina nuestro código; redistribuirla obliga a acompañarla de su
fuente, que es lo que ya hace npm. Para un servidor MCP local, no es un problema
— pero es la única de las tres opciones que no es permisiva *(inferido: no soy
abogado)*.

---

## 3. Graphviz vía WASM (`@hpcc-js/wasm-graphviz` 1.28.0 / Graphviz 15.1.0)

**Grupos anidados — impecables, y los mejor dibujados de los tres.** `subgraph
cluster_x { subgraph cluster_y { … } }` con tres niveles: **10/10 contenidos,
7/7 anidados**, sin solapes ni intrusos. Y con `compound=true`, **las aristas
contra un cluster funcionan** vía `lhead`/`ltail` — el equivalente a `contains`.

**Tamaños variables:** con `shape=box, fixedsize=true, width=<in>, height=<in>`
respeta las dimensiones que le damos con un **error máximo de 0,01 px** sobre
14 nodos. Hay que dividir por 72 (Graphviz trabaja en pulgadas); es la única
fricción.

**Compacidad — gana por goleada, y esto importa más de lo que parece.** Misma
escena de 14 nodos:

| motor | lienzo | área | proporción |
|---|---|---|---|
| dagre | 1594 × 2780 | 4,43 M px² | 0,57 |
| elkjs | 933 × 1256 | 1,17 M px² | 0,74 |
| **Graphviz** | **825 × 799** | **0,66 M px²** | **1,03** |

Proporción 1,03 es prácticamente un cuadrado: entra en una pantalla sin
scrollear ni encoger. Y la longitud total de aristas es **3833 px frente a los
12 585 de elkjs** — un tercio. En el render de la escena realista es visiblemente
el diagrama más legible: cajas grandes, jerarquía clara, poco aire desperdiciado.

**Enrutado:** `splines=spline|ortho|polyline|line`. `ortho` funciona y produce
trazados limpios. El precio: **8-9 cruces frente a los 3 de elkjs** — Graphviz
compacta a costa de cruzar más. En la práctica, sobre el dibujo, el diagrama
compacto con 9 cruces se lee mejor que el diagrama tres veces más alto con 3.

**Latencia — no hay color.** 10 nodos **1,4 ms**, 50 nodos **7,0 ms**, 200 nodos
**23 ms**. Entre **7× y 20× más rápido que los otros dos**. Carga en frío del
módulo WASM: **26 ms, una sola vez**. A escala de 50 nodos, un `visual.update`
completo cuesta menos que un frame.

**Estabilidad — su punto débil, y no tiene arreglo.** `dot` no acepta posiciones
sembradas (`pos` sólo lo honra `neato -n`, que es otro algoritmo y no hace
clusters jerárquicos). Medido: al añadir un nodo, **32/66 pares reordenados** en
la escena densa y **194 px de desplazamiento mediano** en la realista — el peor
de los tres en ambas. Es determinista (misma entrada → mismo dibujo, verificado),
pero *no es estable*: entrada parecida → dibujo distinto. La documentación oficial
lo admite: *"Layouts are not necessarily stable with respect to changes in the
input graph"*. Los paliativos que existen (`weight` en aristas, orden de
declaración, `rank=same`) son manuales y frágiles.

**Peso:** un único `index.js` de **801 KB en bruto, 621 KB gzip**, con el binario
WASM ya dentro — **no hay `.wasm` suelto que servir ni fetch en tiempo de
ejecución** (verificado: carga en Node sin red). Esto encaja bien con la
restricción de "página autocontenida" del ticket 01, aunque gzip comprime peor
que en elkjs porque el WASM ya viene denso.

**Salud:** `@hpcc-js/wasm-graphviz` 1.28.0 el 2026-07-24, repo con **1 sola issue
abierta** y push reciente. 101 k descargas/semana — el menos popular de los tres,
pero es un wrapper fino sobre Graphviz, que lleva treinta años en producción.
Alternativa equivalente: **`@viz-js/viz` 3.29.0** (2026-08-05, MIT, 4346 ★, 13
issues, 156 k desc/sem), más popular y con mejor API declarativa.

**Licencia — el matiz que anula su supuesta ventaja.** El wrapper de HPCC es
Apache-2.0, pero **Graphviz en sí es EPL-2.0** *(verificado en graphviz.org/license)*.
Es decir: **Graphviz y elkjs están bajo la misma licencia**. Elegir Graphviz para
evitar el EPL de elkjs no funciona. Sólo dagre es MIT de arriba abajo.

---

## 4. Otros candidatos, descartados brevemente

- **`dagre-d3`** — no es un motor, es el renderer D3 de dagre. Último release en
  2017, 217 issues. Irrelevante: nosotros renderizamos aparte.
- **`d3-hierarchy`** (ISC) — **último release 2022-04-02**. Sólo hace árboles y
  treemaps: no admite aristas arbitrarias ni ciclos. Nuestro grafo tiene
  `depends` cruzados. Descartado por incapacidad, no por mantenimiento.
- **`d3-dag`** 1.2.2 (MIT, 2026-07-05, 1516 ★) — Sugiyama para DAGs, activo y
  bien tipado. **Sin soporte de contención**: el requisito duro lo elimina.
  Además no admite ciclos, y `calls` puede tenerlos.
- **Cytoscape.js + `fcose`/`cose-bilkent`** (MIT, muy vivo: 3.34.2 en 2026-08-25,
  11 k ★) — soporta *compound nodes* de verdad, pero son layouts **dirigidos por
  fuerzas**: no jerárquicos, no deterministas por defecto, y arrastran todo el
  runtime de Cytoscape *(inferido: no lo ejecuté)*. Para un diagrama de clases,
  donde la dirección de la flecha *es* la información, un layout por fuerzas es
  la herramienta equivocada. Existe `cytoscape.js-elk`, que es elkjs con más capas
  encima.
- **WebCola** — layout con restricciones sobre simulación física. Mismo argumento
  que fcose, y menos mantenido *(inferido)*.
- **ELK más allá de `layered`** — merece una nota: elkjs trae también `mrtree`,
  `stress`, `radial`, `rectpacking` y `force` en el mismo bundle. Si algún día
  entra una segunda familia de diagrama, ya está pagada.

---

## Tabla comparativa

| criterio | dagre 3.1.1 | elkjs 0.12.0 | Graphviz WASM 15.1 |
|---|---|---|---|
| **Grupos anidados (3 niveles)** | ✅ 10/10 y 7/7 | ✅ 10/10 y 7/7 | ✅ 10/10 y 7/7 |
| **Arista contra un grupo** (`contains`) | ❌ **excepción** | ✅ | ✅ (`lhead`/`ltail`) |
| **Tamaños variables dados por nosotros** | ✅ directo | ✅ directo | ✅ (`fixedsize`, error 0,01 px) |
| **Enrutado ortogonal** | ❌ sólo polilínea | ✅ el mejor (3 cruces, 40 codos) | ✅ (`splines=ortho`, 9 cruces) |
| **Compacidad** (14 nodos) | ❌ 4,43 M px², ratio 0,57 | 1,17 M px², ratio 0,74 | ✅ **0,66 M px², ratio 1,03** |
| **Determinista** | ✅ | ✅ | ✅ |
| **Estable ante `+1 nodo`** | 597 px mediana, 0/66 reordenados | 509 px, 34/66 | ❌ 506 px, 32/66 |
| **Modo incremental explícito** | ❌ no existe | ✅ **INTERACTIVE + `elk.position`** → 12 px mediana | ❌ imposible con `dot` |
| **Latencia 10 nodos** | 9,1 ms | 10,0 ms | ✅ **1,4 ms** |
| **Latencia 50 nodos** | 145 ms | 46 ms | ✅ **7,0 ms** |
| **Latencia 200 nodos** | ❌ **excepción** | 171 ms | ✅ **23 ms** |
| **Peso (gzip)** | ✅ **16 KB** | ❌ 455 KB | 621 KB (WASM incluido) |
| **Entorno** | Node + navegador | Node + navegador (+worker) | Node + navegador (WASM) |
| **Licencia** | ✅ **MIT** | EPL-2.0 OR GPL-3.0+ | Apache-2.0 sobre **EPL-2.0** |
| **Último release** | 2026-08-08 | 2026-07-17 | 2026-07-24 |
| **Issues abiertas** | ❌ 174 | 98 | ✅ 1 (wrapper) |
| **Descargas/semana** | 4,2 M | 7,0 M | 101 k (+156 k `@viz-js/viz`) |

---

## Riesgos y cosas que descubrí por el camino

1. **El paquete `dagre` de npm no es dagre.** Está congelado en 2019 y sigue
   sirviendo 2,8 M descargas/semana. Hay que escribir `@dagrejs/dagre`.
2. **Las aristas de elkjs vienen en coordenadas relativas a `container`.** Es la
   causa número uno de "las flechas no llegan a las cajas". Documentarlo en el
   adaptador.
3. **elkjs y Graphviz comparten licencia (EPL-2.0).** No hay elección "más
   permisiva" entre ambos; sólo dagre escapa.
4. **La compacidad pesa más que los cruces.** Las métricas premian a elkjs
   (3 cruces); los ojos premian a Graphviz (un tercio de área, proporción
   cuadrada). Cualquier decisión tomada sólo con la tabla de cruces se equivoca.
5. **La estabilidad importa menos de lo que el ticket teme — en grafos
   realistas.** En la escena de arquitectura de verdad, los tres motores movieron
   los nodos entre 53 px y 194 px de mediana **sin reordenar nada** (dagre y
   elkjs: 0/45 inversiones). El escenario catastrófico de "salta todo el
   diagrama" sólo se dio en el grafo sintético denso. Conviene calibrar cuánto
   presupuesto merece este criterio.
6. **Dónde corre el layout cambia el peso de la balanza.** Si el layout corre en
   el servidor MCP en Node (que es coherente con "el servidor es dueño del
   estado" y con enviar un `PositionedScene` ya resuelto al visor), los 455-621 KB
   de elkjs o Graphviz **no cuestan nada** y el criterio de bundle desaparece de
   la tabla. Sólo si el layout se mueve al visor autocontenido del ticket 01 pasa
   a ser decisivo. **Esta es la decisión que hay que tomar antes que la del
   motor.**

---

## Respuesta corta

**Graphviz vía WASM (`@hpcc-js/wasm-graphviz`), con elkjs como plan B declarado.**
Es el único que gana en lo que de verdad decide si un diagrama "se entiende de un
vistazo": produce un lienzo de 0,66 M px² con proporción 1,03 donde elkjs da
1,17 M y dagre 4,43 M en cinta vertical — cabe en pantalla sin encoger. Encima es
7-20× más rápido (7 ms a 50 nodos), clava tres niveles de anidamiento, respeta
nuestros tamaños al 0,01 px y sabe enrutar contra un cluster con `lhead`. dagre
queda fuera por dos fallos duros verificados: excepción al tocar un nodo-grupo
—que mata `kind: contains`— y excepción con 200 nodos compound.

**El riesgo principal es la estabilidad incremental: `dot` no acepta posiciones
sembradas y no tiene arreglo.** Fue el peor de los tres en ambas pruebas (194 px
de desplazamiento mediano en la escena realista, frente a 53-56 px). elkjs es el
único con antídoto real —`INTERACTIVE` + `elk.position` bajó la mediana de 509 px
a 12 px—, así que **el adaptador de layout debe quedar detrás de una interfaz
sustituible**, y hay que medir la continuidad en el prototipo del ticket 11 antes
de darlo por bueno. Atenuante: en la escena de arquitectura realista los tres
movieron los nodos sin reordenar nada, con lo que el escenario catastrófico puede
no darse nunca en el uso real. Y ojo con la licencia: Graphviz es EPL-2.0, igual
que elkjs — el wrapper Apache-2.0 no cambia eso.
