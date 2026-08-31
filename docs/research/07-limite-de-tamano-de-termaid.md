# El límite de tamaño de termaid, medido

Trabajo de [¿Se lee bien un refactor real?][12], parte 1. [research 05][r05]
verificó que el ruteo de aristas de termaid produce lecturas falsas, pero su
muestra eran 6 diagramas sintéticos: suficiente para detectar los fallos, no para
acotarlos. La decisión de partida 2 asume el giro a la terminal **a cambio de
medir N** — el número de nodos hasta el que la pizarra dibuja la verdad — y
ponerle un límite honesto al MVP.

**Resultado: no hay N utilizable.** Con herencia, el ruteo ya es irrastreable a
**3 nodos**. Con composición entre cajas adyacentes aguanta hasta **4**. Ninguna
configuración de termaid, y ningún layout externo, lo mueve de ahí.

## Método

Todo ejecutado, no estimado. `termaid` 0.8.0 en venv con Python 3.9.

**Los diagramas salen de código real, no de la imaginación del agente** — que es
el punto ciego que [research 05][r05] se marcó a sí misma. Un extractor con `ast`
saca clases, miembros y relaciones de dos paquetes Python instalados, y de ahí se
recortan subgrafos conexos por tamaño:

| Fuente | Clases | Aristas | Tipos de relación |
|---|---:|---:|---|
| `termaid` 0.8.0 | 76 | 54 | composición (atributos tipados) + dependencia (firmas) |
| `asyncio` (stdlib 3.9) | 97 | 39 | herencia pura |

Son dos topologías deliberadamente distintas: una densa y mixta, otra el árbol de
herencia que es el caso protagonista del proyecto.

**Los subgrafos se recortan por BFS** desde el nodo de mayor grado, prefiriendo
vecinos de grado alto — busca el caso denso, que es donde vive el código real. El
desempate es por nombre, así que el barrido es reproducible.

**Cuatro patologías, detectadas automáticamente** sobre el render Unicode, con el
grafo fuente como verdad:

1. **Aristas perdidas.** Calibrado en un caso de control: cada arista produce
   exactamente un marcador (`►◄▲▼△◆◇`). Menos marcadores que aristas significa
   relaciones que no se dibujan.
2. **Paredes corrompidas.** Se reconstruyen las cajas por sus esquinas y se
   verifica que sus bordes son bordes: cualquier `┼`, `┬`, `┴` o punta de flecha
   incrustada en una pared es una arista atravesándola.
3. **Fragmentos huérfanos.** Puntas de flecha sin línea que las alimente.
4. **Cajas sueltas.** Clases con relaciones en el fuente y **ningún** marcador
   tocando su perímetro: su relación es irrastreable.

**Y cada caso se mide en su mejor configuración, no en una fija.** Se prueban 32
combinaciones de `direction` (defecto/`LR`/`TB`), `--gap` (1/2/4/6) y
`--padding-y` (0/1), y se reporta la mejor. Esto importa: [research 05][r05] midió
con `--gap 1` fijo, y había que descartar que el problema fuera el parámetro.
No lo es — `--gap 1` gana en casi todos los casos.

## Resultado

Cada fila es la mejor de 32 configuraciones. "Limpio" = las cuatro patologías a
cero.

### Fuente 1 — `termaid`: composición y dependencia

| Nodos | Aristas | Perdidas | Paredes | Huérfanos | Sueltas | |
|---:|---:|---:|---:|---:|---:|---|
| 3 | 2 | 0 | 0 | 0 | 0 | limpio |
| 4 | 4 | 0 | 0 | 0 | 0 | limpio |
| 5 | 5 | 0 | 0 | 0 | 1 | **roto** |
| 6 | 7 | 0 | 2 | 0 | 1 | **roto** |
| 7 | 8 | 1 | 2 | 0 | 2 | **roto** |
| 8 | 9 | 1 | 2 | 0 | 3 | **roto** |
| 10 | 13 | 1 | 2 | 0 | 4 | **roto** |
| 12 | 17 | 5 | 6 | 1 | 4 | **roto** |
| 14 | 20 | 5 | 8 | 1 | 5 | **roto** |
| 17 | 23 | 6 | 8 | 0 | 5 | **roto** |
| 19 | 25 | 7 | 8 | 0 | 6 | **roto** |

### Fuente 2 — `asyncio`: herencia pura

| Nodos | Aristas | Perdidas | Paredes | Huérfanos | Sueltas | |
|---:|---:|---:|---:|---:|---:|---|
| 3 | 2 | 0 | 0 | 0 | 1 | **roto** |
| 4 | 3 | 1 | 0 | 0 | 2 | **roto** |
| 5 | 4 | 1 | 0 | 0 | 3 | **roto** |
| 6 | 5 | 1 | 0 | 0 | 3 | **roto** |
| 7 | 6 | 1 | 0 | 0 | 3 | **roto** |

La degradación no es un acantilado en un tamaño concreto: **empieza en el mínimo
y crece monótona**. A 19 nodos se pierde el 28 % de las relaciones.

## Las tres lecturas falsas, con el fuente al lado

### 3 nodos, 2 herencias — la herencia no se dibuja

El diagrama de clases más pequeño que tiene sentido, en su mejor configuración:

```
                 ┌───────────────────────────────────┐
                 │       AbstractChildWatcher        │
                 ├───────────────────────────────────┤
                 │ +add_child_handler(pid, callback) │
                 │ +remove_child_handler(pid)        │
                 │ +attach_loop(loop)                │
                 └───────────────────────────────────┘
                 ─────────────────△ △───────────────
  ┌────────────────────────────┐ ┌───────────────────────────────────┐
  │      BaseChildWatcher      │ │       MultiLoopChildWatcher       │
  ├────────────────────────────┤ ├───────────────────────────────────┤
  │ +close()                   │ │ +is_active()                      │
  │ +is_active()               │ │ +close()                          │
  │ -_do_waitpid(expected_pid) │ │ +add_child_handler(pid, callback) │
  └────────────────────────────┘ └───────────────────────────────────┘
```

Fuente: `AbstractChildWatcher <|-- BaseChildWatcher` y
`AbstractChildWatcher <|-- MultiLoopChildWatcher`.

Los dos `△` flotan **juntos en el centro** de una línea horizontal. Ninguna
vertical baja a ninguna subclase; ninguna caja tiene nada tocando su perímetro.
La herencia solo se infiere de que las cajas están debajo — es disposición
espacial, no trazado. En cuanto haya dos niveles, esa inferencia se rompe.

### 6 nodos — una arista atraviesa una caja ajena

La firma que [research 05][r05] documentó, reproducida sobre código real:

```
│ +is_vertical()   │─┼─-target──────────────┼◆│ -id          │
```

`Subgraph *-- Direction` cruza la caja `Edge` de lado a lado, parte el texto del
miembro `-target` y deja un `┼` en cada pared. Se lee
`Direction → Edge → Subgraph`. **Esa cadena no está en el fuente.**

### 5 nodos — dos marcadores inasignables

```
            ◆─◆────────────────────┐
  ┌──────────────────────┐         │
  │         Edge         │ ┌──────────────┐
  │          …           │ │   Subgraph   │
```

`◆─◆` bajo `Graph`: dos composiciones cuyo destino no se puede asignar. `Edge`
está justo debajo y `Subgraph` a la derecha, y no hay forma de decir cuál es
cuál. `Subgraph` queda sin ningún marcador en su perímetro.

## No es la anchura de las etiquetas, y no es el `gap`

Dos hipótesis alternativas, las dos descartadas midiendo:

**El `gap` no lo arregla.** De las 32 configuraciones, `--gap 1 --padding-y 0`
gana en 10 de los 11 casos de la fuente 1. Más espacio no le da al A\* un camino
mejor.

**La anchura desigual tampoco.** Una jerarquía sintética de base + 4 subclases se
dibuja **completa** en las cuatro variantes — nombres cortos uniformes, cortos
desiguales (5 vs 16 caracteres), realistas uniformes y realistas desiguales (26
caracteres). Lo que rompe el caso de `asyncio` no es lo anchas que son las cajas,
sino que cada clase tiene **miembros distintos**: la geometría irregular que
produce el código real y no produce un ejemplo inventado.

Es justo el sesgo del que [research 05][r05] advertía en su propia muestra.

## La salida 2, medida: el layout externo no lo salva

La decisión de partida 3 mantiene el Layout Engine separable, y
`Graph.grid_positions` acepta posiciones precalculadas — la vía por la que
[Motores de layout][3] dejó a Graphviz WASM en pie. Si un layout externo mitigara
el ruteo, el giro se salvaría.

**Primero, por lectura del código: no puede.** `grid_positions` solo alimenta
`_layer_order_from_grid` en `layout/grid.py:135` — sustituye la asignación de
capas por BFS. La colocación de coordenadas la siguen haciendo
`placement`/`coordinates`, y **el trazado lo sigue haciendo el A\* de
`routing/pathfinder.py`**. Un layout externo controla el orden de las capas, no
por dónde va la línea. Además `grid_positions` vive en el pipeline de
flowchart/architecture; el renderer de `classDiagram` es otro camino entero, con
su propio layout de 727 líneas.

**Segundo, midiéndolo en su techo.** En vez de Graphviz se inyecta el orden de
capas que **minimiza cruces** por búsqueda — un límite superior de lo que
cualquier motor externo podría aportar. Con 0-2 cruces:

| Nodos | Perdidas | Paredes | Sueltas | |
|---:|---:|---:|---:|---|
| 6 | 0 | 3 | 0 | mejora, sigue roto |
| 10 | 0 | 5 | 1 | mejora, sigue roto |
| 14 | 0 | 9 | 1 | mejora, sigue roto |
| 20 | 0 | 14 | 0 | mejora, sigue roto |

**Cambia la patología, no la cura.** Deja de perder aristas — eso sí lo arregla —
y a cambio apila puntas inasignables y corrompe más paredes, porque el layout
sale más compacto y las líneas cruzan más cajas. A 10 nodos, con cero cruces:

```
           ▼──────────┼───────────▼──────────┼──────────▼ ▼                     ▼
┌────────────────────┐│┌────────────────────┐│┌────────────────────┐ ┌──────────
│      Subgraph      │││     GraphNote      │││        Node        │ │        Edge
└─────────┬──────────┘│└────────────────────┘│└────────────────────┘ └──────────
          ▼ ▼─────────╯          ▼─▼─────────┼───────────▼───────────────────────
┌────────────────────┐ ┌────────────────────┐│┌────────────────────┐
│     Direction      │ │     EdgeStyle      │►│     ArrowType      │
└────────────────────┘ └────────────────────┘ └────────────────────┘
```

Cinco `▼` en una fila con `┼` cruzando entre ellos; `▼─▼` pegados. Y ese `│►│`
entre `EdgeStyle` y `ArrowType` es una **arista que no existe**: las reales son
`Edge → EdgeStyle`, `Edge → ArrowType` y `_FlowchartParser →` ambas.

## Veredicto

El ticket fijaba tres salidas. Es la **tercera**: *N es demasiado pequeño y nada
lo mitiga*.

- N = 4 en el mejor caso (composición entre cajas adyacentes), **N = 2 para
  herencia** — que es la relación protagonista de "entender un refactor".
- Ninguna de las 32 configuraciones lo mueve.
- El layout externo, medido en su techo teórico, tampoco.
- Y arreglar el ruteo está fuera de alcance por decisión explícita del mapa: es el
  A\* de otro proyecto, roto en sus dos renderers, y adoptarlo devora la única
  ventaja del giro.

Un diagrama de 3 clases no explica un refactor. **El giro a la terminal no se
sostiene sobre termaid**, y la decisión de partida 2 pierde su premisa: el ahorro
de construcción era real, pero se compra con un dibujo en el que no se pueden leer
las relaciones — que es exactamente la enfermedad por la que se descartó tldraw en
[research 04][r04], y ataca la promesa central del proyecto.

## Reproducir

Scripts en la rama de este prototipo: `extract.py` (grafo desde `ast`), `gen.py`
(subgrafos → Mermaid), `analyze.py` (las cuatro patologías), `sweep_all.py`
(32 configuraciones por caso), `external_layout.py` y `ext_sweep.py` (el techo del
layout externo), `width.py` y `fanout.py` (las hipótesis descartadas).

## Fuentes

- `termaid` 0.8.0 — <https://github.com/fasouto/termaid>
- Código leído: `layout/grid.py`, `graph/model.py`, `renderer/draw.py`,
  `routing/pathfinder.py`, `renderer/classdiagram.py`

[3]: https://github.com/javierponferradalopez/ai-render/issues/3
[12]: https://github.com/javierponferradalopez/ai-render/issues/12
[r04]: ./04-mcp-de-tldraw.md
[r05]: ./05-termaid-y-la-terminal.md
