> **MIGRADO A GITHUB (2026-08-31).** Ticket vivo: https://github.com/javierponferradalopez/ai-render/issues/12 — copia congelada, no la edites.

# ¿Se lee bien un refactor real?

Type: prototype
Status: open
Blocked by: 07

## Question

Con el stack de rendering elegido, ¿un diagrama de clases de un refactor de
verdad se entiende de un vistazo — o hemos construido algo bonito e inútil?

Es la prueba de fuego del caso de uso protagonista. Coge un refactor real y
concreto (por ejemplo: sacar `PrismaRepository` de la capa de aplicación a
infraestructura detrás de un puerto), y produce el artefacto:

- El **estado actual** como diagrama de clases con carpetas como grupos
  anidados, clases con sus métodos relevantes, y aristas con `kind` distintos.
- El **estado propuesto** como segunda vista al lado.
- Un **patch incremental** entre ambos, para ver si el diagrama cambia de forma
  comprensible o si salta entero.

Lo que hay que mirar, con el humano delante:

- ¿Se entiende sin explicación hablada?
- ¿Cabe en pantalla a un tamaño legible?
- ¿Las aristas se cruzan hasta hacerse ilegibles?
- ¿La contención anidada se ve como contención?
- ¿Aporta algo sobre escribirlo en texto en la conversación?

Enlaza el prototipo como asset. Si la respuesta es que no se lee bien, este
ticket manda: reabre el stack de rendering o las primitivas.

## Context

[Motores de layout para grafos dirigidos](./02-motores-de-layout.md) dejó una
medición pendiente que este prototipo debe resolver: **la continuidad del layout
entre el estado actual y el propuesto**.

Graphviz —el motor recomendado— no puede hacer layout incremental estable, y fue
el peor de los tres al añadir un nodo (194 px de desplazamiento mediano). Pero en
la escena de arquitectura realista ninguno de los tres reordenó nada, así que el
problema puede ser teórico. Aquí es donde se sale de dudas, mirando el patch
incremental con ojos humanos: **¿el diagrama cambia de forma comprensible, o
salta entero y hay que releerlo?**

Si salta, el plan B es elkjs con `INTERACTIVE` + `elk.position`, que bajó el
desplazamiento mediano de 509 px a 12 px.

[Superficies de dibujo candidatas](./03-superficies-de-dibujo.md) propone además
una prueba concreta de **2-3 horas por rama**: montar el nodo más difícil que
vamos a tener — una clase con 8 atributos y 6 métodos, dentro de un contenedor
anidado con etiqueta, con una arista de herencia (triángulo hueco) y otra de
dependencia (discontinua) — en **React Flow** y en **SVG propio**. Decide con
datos en vez de con argumentos.

Verificar de paso: que pasar `width`/`height` ya calculados evita la doble
medición y el salto visual del primer frame; que el serializador emite los nodos
en orden padre→hijo (requisito de React Flow); el rendimiento con ~200 nodos con
compartimentos; y `await document.fonts.ready` antes de medir texto.
