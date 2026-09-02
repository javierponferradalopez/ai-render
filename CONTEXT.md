# Pizarra efímera para agentes

Un canal visual temporal para que un agente de IA se explique. El agente expresa
significado —qué hay y cómo se relaciona— y el sistema decide cómo se ve; nunca
al revés.

## Language

### El canal

**Pizarra** — `flipchart` en código:
El canal visual temporal en su conjunto. Existe para que el usuario entienda lo
que el agente está diciendo, no para guardar lo que el agente dibuja. El nombre
inglés es provisional y su metáfora cojea en un punto: un rotafolio enseña una
hoja cada vez, y la Pizarra enseña N Vistas conviviendo.
_Avoid_: canvas (en español), lienzo, board, tablero; y como identificadores,
`canvas`, `board` o `whiteboard`

**Efímero**:
Que muere con la sesión MCP, no con la disciplina del agente. Un artefacto
efímero no se guarda, no se exporta y no sobrevive a la sesión MCP que lo
motivó. Una sesión MCP **no** es una conversación: `/clear` acaba la conversación
y deja viva la sesión, así que la pizarra le sobrevive.
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
Lo que expone las herramientas al agente y es **dueño del estado** de la pizarra.
La verdad vive aquí. No es un proceso propio: comparte proceso con el Visor y
vive en un hilo distinto del suyo.
_Avoid_: backend, host

**Visor**:
Lo que recibe una escena y la pinta, en su propia ventana. Es tonto por diseño:
no guarda nada, y cerrar su ventana no pierde nada porque el estado no es suyo.
No se reinicia: su ventana se oculta y se vuelve a mostrar dentro del mismo
proceso, que sólo muere con la sesión MCP.
_Avoid_: frontend, cliente, viewer, app, página

**Superficie de entrega**:
Por dónde llega el visor a los ojos del usuario. Distinta de la Drawing Surface,
que es con qué se pinta dentro.
_Avoid_: superficie a secas

**Lanzador**:
Lo que el host invoca para que exista el Proceso de la pizarra. **No lo trae**: el
ejecutable ya está en la máquina cuando el lanzador corre. Pero sí lo deja utilizable
—puede llegar sin permiso de ejecución, y dárselo es trabajo suyo—, porque nadie
promete en qué estado aparece. Y nunca falla, porque su fallo no se ve como un error
sino como una pizarra que deja de existir sin decir por qué; cuando no puede cederle
el sitio, se queda hablando él como Servidor de aviso.
_Avoid_: instalador, wrapper, shim, script de arranque

**Servidor de aviso**:
La cara del Lanzador cuando no hay Proceso de la pizarra al que ceder el sitio.
Habla lo justo para decir que la pizarra no está operativa y qué se ha encontrado;
no dibuja, no guarda nada y no intenta arreglarse. Existe porque el silencio es el
único fallo que el producto no puede permitirse: no hay nadie más que pueda
contárselo al usuario.
_Avoid_: stub, fallback, modo degradado, servidor a secas

**Proceso de la pizarra**:
El único proceso que hay, y que es a la vez Servidor MCP y Visor. Lo lanza el
host como hijo y le habla por stdio. Reparte los dos papeles entre hilos: el
principal dibuja, el secundario sirve. Cuál manda no es simétrico — el hilo que
sirve es el que sabe qué hora es y el que decide cuándo sale el proceso, porque
el que dibuja se congela cuando el sistema tapa la ventana.
_Avoid_: demonio, daemon, servidor a secas
