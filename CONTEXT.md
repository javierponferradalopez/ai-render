# Pizarra efímera para agentes

Un canal visual temporal para que un agente de IA se explique. El agente expresa
significado —qué hay y cómo se relaciona— y el sistema decide cómo se ve; nunca
al revés.

## Language

### El canal

**Pizarra**:
El canal visual temporal en su conjunto. Existe para que el usuario entienda lo
que el agente está diciendo, no para guardar lo que el agente dibuja.
_Avoid_: canvas (en español), lienzo, board, tablero

**Efímero**:
Que muere con la sesión MCP, no con la disciplina del agente. Un artefacto
efímero no se guarda, no se exporta y no sobrevive a la conversación que lo
motivó.
_Avoid_: temporal, volátil

**Vista**:
Una de las N representaciones con nombre que conviven en la pizarra a la vez —
"actual" junto a "propuesto", o un diagrama de clases junto a un flujo. Se
identifica por su `id`, y volver a mostrarla sobre ese `id` la reemplaza.
_Avoid_: diagrama, escena, tab, pestaña

### Las capas

**Visual Protocol**:
Lo que el agente emite: significado puro —nodos, relaciones, contención—. Tiene
prohibido expresar HTML, SVG, CSS, coordenadas, tamaños, colores o formas
concretas. Es la frontera que el agente nunca cruza hacia abajo.
_Avoid_: formato, payload, DSL

**VisualDocument**:
El estado semántico completo de una vista, expresado en el Visual Protocol. Es
lo que el agente describe y lo que el sistema conserva como verdad.
_Avoid_: modelo, escena, grafo

**Layout Engine**:
La pieza que decide dónde va cada cosa. Toma un VisualDocument y produce una
PositionedScene. El agente no participa en esta decisión.
_Avoid_: motor de posicionamiento, autolayout

**PositionedScene**:
Un VisualDocument con geometría ya resuelta: lo que queda cuando el Layout
Engine ha hecho su trabajo y antes de que se pinte un solo píxel.
_Avoid_: layout, escena posicionada

**Drawing Surface**:
Con qué se pinta finalmente dentro del visor. De qué está hecha es invisible para
todo lo que hay aguas arriba, y no contradice la prohibición del Visual Protocol:
lo prohibido es que el **agente** produzca HTML o caracteres de dibujo, no que
existan.
_Avoid_: renderer, canvas, pintor

**Renderer**:
Término contaminado: mezcla el Layout Engine con la Drawing Surface, que son
piezas de primera clase separadas y sustituibles por separado. Nombrar la que
toca.
_Avoid_: usar la palabra a secas

**Límite honesto**:
El tamaño máximo de vista que el visor dibuja sin mentir. Pasado ese punto la
pizarra no dibuja peor: se para y lo dice. Existe porque la Drawing Surface
elegida produce lecturas falsas por encima de cierto número de nodos, y dibujar
una relación que no existe es peor que no dibujar nada.
_Avoid_: límite a secas, truncado, degradación

### Las piezas vivas

**Servidor MCP**:
El proceso local que expone las herramientas al agente y es **dueño del estado**
de la pizarra. La verdad vive aquí.
_Avoid_: backend, host

**Visor**:
Lo que recibe una escena y la pinta, en su propia ventana. Es tonto por diseño:
no guarda nada, y reiniciarlo no pierde nada porque el estado no es suyo.
_Avoid_: frontend, cliente, viewer, app, página

**Superficie de entrega**:
Por dónde llega el visor a los ojos del usuario. Distinta de la Drawing Surface,
que es con qué se pinta dentro.
_Avoid_: superficie a secas
