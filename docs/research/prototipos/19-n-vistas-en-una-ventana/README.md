# Maqueta: N vistas en una ventana

Prototipo del 2026-09-02 para
[Cómo se ven N vistas en una ventana](https://github.com/javierponferradalopez/ai-render/issues/19).

El ticket no pregunta si se puede, pregunta **cómo debería verse**, así que esto
no demuestra nada: pone las respuestas candidatas como interruptores para poder
mirarlas en vez de discutirlas.

## Qué es

Una ventana `egui` con el camino real del producto dentro:

```
fixture .mmd  --mmdr 0.3.1-->  SVG  --resvg-->  textura  --egui-->  ventana
```

El SVG es el que produce el motor que
[El stack de rendering](https://github.com/javierponferradalopez/ai-render/issues/8)
fijó, y lo rasteriza `resvg` a la escala del momento, como decidió
[Transporte y ciclo de vida](https://github.com/javierponferradalopez/ai-render/issues/13).
Lo que se mira, por tanto, es el dibujo de verdad, no un rectángulo de mentira.

Las cinco Vistas son de tamaños deliberadamente dispares, porque esa es una de
las preguntas del ticket:

| Vista | fixture | natural |
|---|---|---|
| `actual` | classDiagram de 6 clases | 497 × 913 |
| `propuesto` | el mismo refactor, ya movido | 523 × 759 |
| `flujo` | flowchart LR con subgraphs | 1192 × 344 |
| `pequeno` | 3 nodos | 266 × 286 |
| `grande` | 20 nodos | 787 × 1164 |

Y `x-actual-v2` no es una Vista: es el recambio con el que el botón
**show sobre «actual»** simula un `show` sobre un id que ya existe. Es más grande
que el original a propósito, que es lo que hace saltar el dibujo.

## Los interruptores

- **disposición** — pestañas · columnas · rejilla · apiladas · mezcla
- **encaje** — natural (1:1) · encoger nunca agrandar · cada una a su hueco · escala común
- **zoom** — de la ventana entera · por vista (rueda encima de cada una)
- **orden** — creación · alfabético · último show
- **nombres** — el `id` de la Vista visible o no
- **vistas vivas** — apagar Vistas simula una Pizarra más pequeña
- **al reemplazar** — quieta · salta a ella · quieta + marca

## Cómo se corre

```sh
./genera-vistas.sh                      # baja mmdr =0.3.1 (sha256 verificado) y genera los SVG
cargo run                               # la ventana, para jugar
cargo run -- --captura capturas         # regenera las 10 capturas y sale
```

Requiere `~/.cargo/bin` en el `PATH`. `vistas/`, `mmdr` y `target/` no se
comitean; `capturas/` sí, que es lo que se mira desde el issue.

## Lo que se ve

`capturas/` tiene las cinco disposiciones × dos escenarios: la Pizarra de cinco
Vistas y la del caso protagonista, `actual` + `propuesto`. Todas a 1400×950.

| | 2 vistas | 5 vistas |
|---|---|---|
| columnas | [✓ legible](capturas/2-vistas-columnas.png) | [✗ 20 %](capturas/5-vistas-columnas.png) |
| rejilla | [2-vistas-rejilla](capturas/2-vistas-rejilla.png) | [✗ 27–41 %](capturas/5-vistas-rejilla.png) |
| apiladas | [✓ 100 %, una por pantalla](capturas/2-vistas-apiladas.png) | [5-vistas-apiladas](capturas/5-vistas-apiladas.png) |
| pestañas | [2-vistas-pestanas](capturas/2-vistas-pestanas.png) | [5-vistas-pestanas](capturas/5-vistas-pestanas.png) |
| mezcla | [2-vistas-mezcla](capturas/2-vistas-mezcla.png) | [5-vistas-mezcla](capturas/5-vistas-mezcla.png) |

Lo que la maqueta enseñó nada más encenderse, y que no estaba en el ticket:

1. **La disposición buena depende de N, no del gusto.** Columnas con dos Vistas
   las deja al 73 % y al 88 %, las dos legibles y comparables de un vistazo;
   con cinco, la misma disposición manda `flujo` al 20 % y `grande` al 31 %. La
   rejilla con cinco no salva a ninguna: 27–41 %.
2. **«Cada una a su hueco» agranda, y agrandar miente.** En la pila, donde no hay
   altura que limite, estira `actual` al 278 %; y en la rejilla pone `pequeno`
   al 128 % **al lado** de `grande` al 27 %, con lo que el diagrama de tres nodos
   se ve cinco veces mayor que el de veinte. Por eso la maqueta abre en *encoger,
   nunca agrandar*, que es un cuarto encaje que el ticket no listaba.
3. **A 1:1 se lee todo y no cabe casi nada.** `actual` mide 913 px de alto: en una
   ventana de 950 entra una Vista por pantalla, y sobran 900 px de ancho a la
   derecha. Los diagramas de clases salen altos y estrechos, así que la pila
   desperdicia justo la dimensión que sobra.
