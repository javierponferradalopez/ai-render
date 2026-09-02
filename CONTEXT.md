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
El subconjunto de Mermaid que el agente tiene permitido escribir: significado
—nodos, relaciones, contención— con todos los ids declarados y sin nada que pida
píxeles. Es la frontera que el agente nunca cruza hacia abajo.
_Avoid_: formato, payload, DSL, y "Mermaid" a secas — el idioma entero no es el
protocolo

**VisualDocument**:
El estado semántico completo de una Vista, escrito en el Visual Protocol y
conservado tal cual. Es la verdad de la Vista, no una copia de algo anterior.
_Avoid_: modelo, escena, grafo, fuente

**Layout Engine**:
La pieza que decide dónde va cada cosa: toma un VisualDocument y produce una
PositionedScene. Ni el agente ni el usuario participan en esa decisión, y no es
sustituible — viene con el idioma en la misma pieza que lo entiende.
_Avoid_: motor de posicionamiento, autolayout

**PositionedScene**:
Un VisualDocument con geometría ya resuelta: lo que queda cuando el Layout
Engine ha hecho su trabajo y antes de que se pinte un solo píxel. Vive dentro del
Servidor MCP y no cruza al Visor.
_Avoid_: layout, escena posicionada

**Drawing Surface**:
Con qué se pinta finalmente dentro del visor. De qué está hecha es invisible para
todo lo que hay aguas arriba, y no contradice la prohibición del Visual Protocol:
lo prohibido es que el **agente** produzca HTML o caracteres de dibujo, no que
existan.
_Avoid_: renderer, canvas, pintor

**Renderer**:
Término contaminado: mezcla el Layout Engine con la Drawing Surface, que son
etapas distintas aunque hoy lleguen en la misma pieza. Nombrar la que toca.
_Avoid_: usar la palabra a secas

**Límite honesto**:
La frontera que la pizarra no cruza dibujando: pasado ese punto no dibuja peor,
se para y lo dice. Cubre el tamaño de Vista que ya no se lee, y también el
significado que no se sostiene — dibujar una relación que no existe es peor que
no dibujar nada.
_Avoid_: límite a secas, truncado, degradación

**Nodo fantasma**:
El nodo que el idioma inventa cuando una relación nombra un id que nadie declaró.
Se dibuja vacío al lado del que se quiso nombrar, así que no se lee como el error
que es sino como algo de lo que se sabe menos. Es la forma que toma la mentira
cuando el agente se equivoca escribiendo, y el Límite honesto existe para
rechazarlo.
_Avoid_: nodo implícito, auto-creación, typo

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
