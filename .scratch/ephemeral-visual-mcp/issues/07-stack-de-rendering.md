# El stack de rendering: layout engine + drawing surface

Type: grilling
Status: open
Blocked by: 02, 03, 04, 14

## Question

¿Qué motor de layout y qué superficie de dibujo se usan en el MVP?

Se deciden **juntos** aunque sean piezas separadas (decisión de partida 3),
porque la elección interactúa: algunos candidatos traen layout incorporado, otros
imponen restricciones sobre las dimensiones de nodo o el enrutado de aristas que
el motor tiene que respetar.

A decidir:

- El **Layout Engine** ganador, con la contención anidada como criterio duro.
- La **Drawing Surface** ganadora — incluido el veredicto sobre tldraw, que
  entra a este ticket como candidato y no como decisión heredada.
- **La frontera exacta entre las dos.** ¿Qué es un `PositionedScene`? ¿Qué
  decide el motor y qué decide la superficie? Aquí es donde se hace real el
  principio de §20 del handoff: nada de tldraw (o de quien gane) puede escaparse
  hacia arriba.
- **Estabilidad del layout entre updates**: ¿se acepta que el diagrama se
  recoloque al añadir un nodo, o es requisito que no salte?
- Las consecuencias de licencia, si las hay, para publicar en npm.

Al cerrar, revisa si esto invalida algo del mapa — en particular, si tldraw cae,
qué partes del handoff dejan de aplicar.

## Context

[Motores de layout para grafos dirigidos](./02-motores-de-layout.md) ya midió los
tres candidatos y recomienda **Graphviz WASM, con elkjs como plan B y dagre
descartado** (excepción al conectar una arista a un grupo — mata `kind:
contains`). Eso deja este ticket con menos trabajo del previsto en la mitad
"layout", pero le añade dos decisiones que no estaban:

- **¿Dónde corre el layout — servidor MCP en Node, o visor en el navegador?**
  Hay que responderla **antes** que la del motor: si corre en el servidor, los
  455-621 KB de elkjs o Graphviz no cuestan nada y el peso desaparece de la
  tabla; si corre en el visor, es decisivo. Interactúa con "el servidor es dueño
  del estado" (decisión de partida 6) y con la restricción de página
  autocontenida de [La superficie de entrega](./06-superficie-de-entrega.md).
- **¿Cuánto vale la estabilidad del layout entre updates?** Graphviz no puede
  ofrecerla (`dot` no siembra posiciones); elkjs sí, y de forma espectacular
  (mediana de 509 px a 12 px). Pero en la escena realista medida la diferencia
  casi desaparece. Decidir si es criterio eliminatorio o un "ya veremos",
  sabiendo que [el prototipo](./11-prototipo-refactor-real.md) lo va a medir.

Restricción heredada: **el motor de layout va detrás de una interfaz
sustituible**, precisamente porque el ganador tiene ese punto ciego.

Aviso de licencia: **elkjs y Graphviz son ambos EPL-2.0**. No hay opción
permisiva entre los dos.

[Superficies de dibujo candidatas](./03-superficies-de-dibujo.md) ya cerró la
mitad "drawing surface": **tldraw fuera por licencia**, recomendado **React
Flow** con SVG propio como plan B, y el texto de los nodos en **HTML** (el export
a imagen está fuera del MVP, así que `<tspan>` no compra nada).

Eso deja a este ticket una tensión que ninguno de los dos research resuelve solo:

- **Graphviz corre en Node y necesita las dimensiones de cada nodo *antes* de
  calcular el layout. React Flow mide el texto en el DOM, *después*.** O medimos
  el texto en el servidor con métricas de fuente, o el layout se mueve al
  navegador. Es la misma decisión de "dónde corre el layout", vista desde el otro
  lado — y ahora tiene consecuencias concretas: si la medición viaja dentro de la
  escena (líneas ya partidas y anchos calculados), el visor se simplifica mucho
  en cualquiera de las dos ramas.
