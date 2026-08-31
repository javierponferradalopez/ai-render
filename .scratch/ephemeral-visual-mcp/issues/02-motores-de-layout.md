> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/3 — copia congelada, no la edites.

# Motores de layout para grafos dirigidos

Type: research
Status: resolved

## Question

¿Qué motor de layout puede convertir un grafo dirigido semántico en coordenadas,
con la calidad suficiente para que un diagrama de clases o de arquitectura se
entienda de un vistazo?

Candidatos de partida: **dagre**, **elkjs**, **graphviz** (vía WASM), y lo que
aparezca. Para cada uno, los criterios que importan aquí:

- **Grupos anidados / contención.** Es el requisito duro: representar carpetas y
  capas exige que el motor sepa colocar nodos *dentro* de contenedores y enrutar
  aristas que los cruzan. Varios motores populares lo hacen mal o no lo hacen.
- **Nodos de tamaño variable.** Una clase con ocho métodos es una caja alta; el
  motor tiene que aceptar dimensiones dadas por nosotros, no imponer las suyas.
- **Calidad de enrutado de aristas** y cruces: ¿ortogonal, spline, recta?
- **Estabilidad ante cambios incrementales.** Si se añade un nodo, ¿el resto se
  queda donde estaba o salta todo el diagrama? Esto importa mucho: un layout que
  se reordena entero en cada `visual.update` rompe la sensación de continuidad.
- **Peso del bundle, entorno de ejecución** (¿corre en Node, en el navegador, en
  ambos?) y **latencia** con grafos de ~10 y ~50 nodos.
- **Licencia** y salud del proyecto (mantenimiento, issues abiertos, releases).

Entregar una comparativa corta con una recomendación razonada, no un catálogo.

## Context

Hallazgos: [research/02-motores-de-layout.md](../research/02-motores-de-layout.md)

## Answer

**Graphviz vía WASM (`@hpcc-js/wasm-graphviz`), con elkjs como plan B declarado
y dagre descartado.** Medido, no leído: los tres motores ejecutados contra una
escena sintética y contra el caso protagonista real
(`src/{domain/{model,services},infra/{db,http},ui}`, 10 clases, 12 aristas, tres
niveles de anidamiento), con los renders inspeccionados visualmente.

### Por qué Graphviz

- **Compacidad, que es lo que decide si un diagrama se lee de un vistazo.** Misma
  escena de 14 nodos: Graphviz **0,66 M px² con proporción 1,03** (casi
  cuadrado, entra en pantalla), elkjs 1,17 M, dagre **4,43 M en cinta vertical**.
  La longitud total de aristas es un tercio de la de elkjs.
- **7-20× más rápido**: 7 ms a 50 nodos, 23 ms a 200. Un `visual.update` cuesta
  menos que un frame.
- **Contención impecable** a tres niveles, y **aristas contra un cluster** vía
  `compound=true` + `lhead`/`ltail` — que es lo que necesita `kind: contains`.
- Respeta nuestros tamaños con error de 0,01 px (hay que dividir por 72; trabaja
  en pulgadas).
- Un solo fichero con el WASM dentro, sin fetch en tiempo de ejecución — encaja
  con la restricción de "página autocontenida".

### Por qué se cae dagre

Dos fallos duros reproducidos: **una arista que toque un nodo-grupo lanza
excepción** (`TypeError ... setting 'rank'`, issue #238, vivo en 3.1.1) — eso
mata `kind: contains` — y **excepción con 200 nodos compound**. Además es el más
lento a 50 nodos y el que peor compacta. Nota de riesgo: el paquete `dagre` de
npm está congelado en 2019 y sigue sirviendo 2,8 M descargas/semana; el vivo es
`@dagrejs/dagre`.

### El riesgo, y es el que importa

**Graphviz `dot` no acepta posiciones sembradas: no se puede hacer layout
incremental estable, y no tiene arreglo.** Fue el peor de los tres al añadir un
nodo (194 px de desplazamiento mediano en la escena realista, frente a 53-56 px).
**elkjs es el único con antídoto**: `INTERACTIVE` + `elk.position` baja la
mediana de 509 px a **12 px**.

Atenuante medido: en la escena de arquitectura realista los tres motores movieron
nodos **sin reordenar nada** (0/45 inversiones en dagre y elkjs). El escenario
catastrófico de "salta todo el diagrama" solo apareció en el grafo sintético
denso. Puede que en uso real no se dé nunca — pero eso hay que verlo, no
suponerlo.

**Consecuencia obligatoria:** el motor de layout va detrás de una interfaz
sustituible, y la continuidad entre updates se mide en
[¿Se lee bien un refactor real?](./11-prototipo-refactor-real.md) antes de dar
Graphviz por bueno.

### Dos avisos

- **Licencia: elkjs y Graphviz están bajo la misma, EPL-2.0.** El wrapper de
  HPCC es Apache-2.0 pero Graphviz no. Elegir Graphviz para escapar del EPL de
  elkjs no funciona. Solo dagre es MIT de arriba abajo — y dagre está descartado.
- **Las aristas de elkjs vuelven en coordenadas relativas a `container`**, no
  absolutas. Es la causa número uno de "las flechas no llegan a las cajas". Si
  se acaba en elkjs, documentarlo en el adaptador.

### Y una decisión que este ticket destapa

**¿Dónde corre el layout — en el servidor MCP (Node) o en el visor (navegador)?**
Si corre en el servidor, los 455-621 KB de elkjs o Graphviz no cuestan nada y el
criterio de peso desaparece de la tabla; si corre en el visor autocontenido, pasa
a ser decisivo. Va a
[El stack de rendering](./07-stack-de-rendering.md), y hay que responderla
**antes** que la del motor.

Método, tablas completas y candidatos descartados (d3-dag, Cytoscape/fcose,
WebCola, d3-hierarchy): [research/02-motores-de-layout.md](../research/02-motores-de-layout.md)
