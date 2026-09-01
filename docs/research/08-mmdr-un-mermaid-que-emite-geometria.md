# mmdr: un Mermaid que emite geometría

Investigación disparada el 2026-09-01, al proponerse
[`mermaid-rs-renderer`][mmdr] (binario `mmdr`) como motor de render sin
navegador. No decide [El stack de rendering][8], pero **tumba la premisa sobre la
que ese ticket está planteado**: que elegir Mermaid obliga a renunciar al corte en
dos piezas.

## Nota sobre el método

Todo lo que sigue está **ejecutado**, no estimado, en un macOS 25.6 arm64.
Binario `mmdr` 0.3.1 del release `aarch64-apple-darwin`, con el sha256 verificado
contra el publicado (`562d0250…f3223`).

**Los casos son los mismos que tumbaron a termaid**, regenerados con los scripts
del [prototipo 12](./prototipos/12-limite-de-termaid/): clases reales extraídas
con `ast` de `termaid` 0.8.0 (composición y dependencia) y de `asyncio` de Python
3.9 (herencia pura), recortadas en subgrafos conexos por tamaño. Solo cambia el
renderer al que se le dan. Scripts y renders:
[prototipo 13](./prototipos/13-mmdr-frente-a-termaid/).

## 1. Qué es

Reimplementación de Mermaid **en Rust puro**, sin navegador y sin Node. MIT.
Creado el 22-01-2026, activo (último push el 23-08-2026), 1.683 estrellas, 96
forks, 545 commits, **un solo autor**. 62.665 líneas de Rust en 56 ficheros y
6.508 de tests de integración. 20 issues abiertos contra 46 cerrados.

El binario pesa **6,9 MB y no tiene runtime**: sus únicos enlaces dinámicos son
`libSystem` y `libiconv`. Hay releases precompiladas para cuatro plataformas, y
además brew, scoop, AUR y nix. No hay toolchain de Rust que instalar — es un
fichero.

Declara 23 tipos de diagrama. Verificados aquí: `classDiagram`, `flowchart` con
`subgraph`, `sequenceDiagram`, `stateDiagram-v2`, `erDiagram` y `mindmap`.

Su propio README avisa: *"under active early development. Visual output quality
is improving rapidly but may not yet match mermaid-cli in all cases."*

## 2. El hallazgo: `--dumpLayout`

[El stack de rendering][8] plantea su apartado 2 sobre una premisa:

> **Mermaid es un monolito**: parsea, hace layout y dibuja en un bloque. No se le
> inyectan posiciones […] Y el `PositionedScene` deja de existir como etapa.

Es cierto de Mermaid.js. **No es cierto de mmdr.** La bandera `--dumpLayout`
vuelca el layout calculado como JSON: cada nodo con `x`, `y`, `width`, `height` y
sus `label_lines` ya partidas, y cada arista con su polilínea de puntos.

```json
{ "kind": "Class", "direction": "TopDown", "width": 574.6, "height": 780.0,
  "nodes": [ { "id": "Graph", "shape": "Rectangle",
               "x": 135.1, "y": 184.0, "width": 181.5, "height": 187.2,
               "label_lines": ["Graph", "---", "-direction", "-nodes", "---",
                               "+add_node(node)", "+add_edge(edge)", "+get_roots()"] } ],
  "edges": [ { "from": "Graph", "to": "Direction", "directed": false,
               "points": [[241.9,371.2],[241.9,360.2],[287.1,360.2],[287.1,638.0],[256.3,638.0]] } ] }
```

Eso **es** la PositionedScene del glosario, servida como dato. Consecuencias
directas sobre las decisiones del mapa:

- **La decisión 3 no tiene que elegir entre Mermaid y el corte en dos piezas.**
  Existe una implementación de Mermaid que expone la etapa intermedia.
- **Graphviz WASM sigue sin sitio**, y eso no cambia: mmdr trae su propio layout y
  tampoco admite que se le inyecten posiciones. Lo que vuelve no es el motor
  sustituible; es la **PositionedScene como dato del servidor**.
- **Encaja con la decisión 6** (*el servidor MCP es dueño del estado, el visor es
  tonto*). Con Mermaid.js el layout se calcula dentro del visor, que es
  precisamente la pieza que el mapa declara desechable; con mmdr el servidor
  conoce la geometría sin depender de él.

## 3. Calidad de layout, sobre los casos que tumbaron a termaid

Las cuatro patologías del [research 07][r07], trasladadas a geometría. Dos de
ellas —paredes corrompidas y fragmentos huérfanos— **no pueden darse en SVG**:
son artefactos de dibujar sobre una rejilla de caracteres. Se sustituyen por su
equivalente honesto en píxeles, una arista que atraviesa una caja ajena.

### Fuente 1 — `termaid`: composición y dependencia

| Nodos | Aristas | Perdidas | Cruces | Sueltas | Lienzo | |
|---:|---:|---:|---:|---:|---|---|
| 3 | 2 | 0 | 0 | 0 | 248×604 | limpio |
| 4 | 4 | 0 | 1 | 0 | 402×604 | roce |
| 5 | 5 | 0 | 0 | 0 | 572×604 | limpio |
| 6 | 7 | 0 | 0 | 0 | 574×780 | limpio |
| 7 | 8 | 0 | 0 | 0 | 598×780 | limpio |
| 8 | 9 | 0 | 0 | 0 | 753×780 | limpio |
| 10 | 13 | 0 | 0 | 0 | 814×780 | limpio |
| 12 | 17 | 0 | 0 | 0 | 1058×780 | limpio |
| 14 | 20 | 0 | 0 | 0 | 1188×847 | limpio |
| 17 | 23 | 0 | 2 | 0 | 1481×861 | roce |
| 19 | 25 | 0 | 1 | 0 | 1574×890 | roce |

### Fuente 2 — `asyncio`: herencia pura

| Nodos | Aristas | Perdidas | Cruces | Sueltas | Lienzo | |
|---:|---:|---:|---:|---:|---|---|
| 3 | 2 | 0 | 0 | 0 | 602×366 | limpio |
| 4 | 3 | 0 | 0 | 0 | 859×366 | limpio |
| 5 | 4 | 0 | 0 | 0 | 1134×366 | limpio |
| 6 | 5 | 0 | 0 | 0 | 1136×563 | limpio |
| 7 | 6 | 0 | 0 | 0 | 1299×563 | limpio |

**Cero aristas perdidas y cero cajas sueltas en los dieciséis casos.** termaid
perdía hasta el 28 % de las relaciones y dejaba hasta 6 cajas sin nada tocando su
perímetro.

Los tres casos con "cruce" están **verificados a ojo** en n04 y n17
(`renders/termaid-n17.png`): son tangencias — una arista que pasa pegada al borde
o cruza la esquina de una caja vacía de texto. No parten el texto de un miembro ni
inventan una cadena de relaciones. El detector es deliberadamente estricto y sus
positivos hay que mirarlos antes de creerlos; el de n20 no se ha mirado.

El caso protagonista del proyecto —**3 clases con herencia, donde termaid dejaba
los dos `△` flotando sueltos en el centro sin bajar a ninguna subclase**— mmdr lo
dibuja con las dos flechas trazadas de subclase a superclase
(`renders/asyncio-n03.png`).

**El límite honesto deja de ser el problema que era.** No se ha encontrado el
punto de ruptura dentro del tamaño de un refactor; a 19 nodos y 25 aristas el
dibujo sigue siendo fiel. Lo que queda pendiente a escala es lo que el mapa ya
anota en *Not yet specified*: desorden, no mentira.

## 4. Velocidad

Mediana de 20 ejecuciones, proceso completo incluido:

| Caso | mmdr |
|---|---:|
| `classDiagram` 6 nodos | **3,3 ms** |
| `classDiagram` 17 nodos | **14,9 ms** |
| `flowchart` 8 nodos + 4 grupos | **62,1 ms** |
| Arranque en frío | 146 ms |

La referencia del README para mermaid-cli es ~1.900-2.000 ms por diagrama, con
Chromium detrás. El caso caro no es el número de nodos sino el layout de grupos:
un flowchart con cuatro `subgraph` cuesta cuatro veces más que diecisiete clases.

## 5. Determinismo y estabilidad incremental

Dos propiedades que nadie había medido y que una pizarra **en vivo** necesita:

- **Determinista.** Cinco renders del mismo fuente producen un único hash SHA-256.
- **Estable ante cambios incrementales.** Añadir una clase con su relación a un
  diagrama de 6 movió **1** de los 6 nodos preexistentes, sin cambiar el tamaño
  del lienzo. La pizarra no salta entera cuando el agente retoca.

Esto es exactamente el punto ciego que [Motores de layout][3] anotó de Graphviz
(*"no puede hacer layout incremental estable"*), y aquí sale bien sin haber sido
diseñado para ello.

## 6. Donde flojea: los grupos

El caso `arch.mmd` —cuatro capas como `subgraph`, ocho nodos, siete aristas
cruzándolas— es el peor render de la tanda (`renders/arch-subgraphs.png`):
coloca `Infrastructure` entre `API` y `Application`, y saca aristas que rodean
media figura para entrar por el lado contrario. Las siete relaciones están
trazadas y ninguna caja se corrompe, así que sigue sin mentir, pero está lejos de
la calidad de Mermaid.js con dagre.

Es coherente con sus issues abiertos: ruteo en U anchas (#136), ramas de decisión
que hacen S por encima de etiquetas hermanas (#123), aristas que rozan bordes
(#122) y un problema de generación de `subgraph` (#140).

**Y es el eje que importa**, porque la contención es la decisión de partida 5:
`Group` anidable como ciudadano de primera, que es lo que representa carpetas y
capas. mmdr las dibuja; las dibuja regular.

## 7. Lo que no resuelve

- **No devuelve la terminal al proyecto.** mmdr emite píxeles, no celdas de
  carácter: no es un sustituto de termaid, es un sustituto de mermaid-cli. Y la
  terminal del usuario de este proyecto es **Alacritty 0.17.0**, que no
  implementa ni sixel ni el protocolo gráfico de Kitty (verificado: cero
  apariciones de `sixel` en su binario). La decisión de partida 2 sigue en pie
  tal cual.
- **Si el visor es el navegador, el rival de mmdr no es mermaid-cli.** Es
  **Mermaid.js dentro del visor**, que no cuesta ni proceso, ni binario, ni
  descarga por plataforma, y cuya calidad de layout hoy es mejor. Los "100-1400×"
  del README se miden contra Chromium, y en un diseño que ya tiene una ventana de
  navegador abierta esa comparación **no aplica**. Lo que mmdr compra no es
  velocidad: es **geometría en el servidor** e independencia de la superficie.
- **No cierra [El stack de rendering][8]**, que sigue bloqueado por
  [¿Qué añade esto sobre Mermaid?][15].

## 8. Riesgos

- **Versión 0.3.1 y aviso del propio autor.** Un solo mantenedor. Adoptarlo es
  apostar por un proyecto joven en el punto exacto donde flojea (grupos).
- **El formato de `--dumpLayout` no tiene garantía de estabilidad.** Si se usa
  como fuente de la PositionedScene, hay que fijar la versión.
- **Entra Rust en el proyecto**, aunque como binario descargable y no como
  toolchain: es menos invasivo que el venv de Python que el giro a termaid llegó
  a aceptar, y no rompe la instalación como plugin (decisión 7) porque el binario
  se descarga por plataforma, no se compila.
- **No es Mermaid**, es una reimplementación. Lo que renderiza Mermaid.js y lo que
  renderiza mmdr pueden divergir en sintaxis de borde; `%%{init}%%` se parsea pero
  no se aplica (#137).

## Fuentes

- `mermaid-rs-renderer` 0.3.1 — <https://github.com/1jehuang/mermaid-rs-renderer>
- Binario `mmdr-aarch64-apple-darwin` del release v0.3.1, sha256 verificado
- Casos y extractor: [prototipo 12](./prototipos/12-limite-de-termaid/)
- Medición: [prototipo 13](./prototipos/13-mmdr-frente-a-termaid/)

[3]: https://github.com/javierponferradalopez/ai-render/issues/3
[8]: https://github.com/javierponferradalopez/ai-render/issues/8
[15]: https://github.com/javierponferradalopez/ai-render/issues/15
[r07]: ./07-limite-de-tamano-de-termaid.md
[mmdr]: https://github.com/1jehuang/mermaid-rs-renderer
