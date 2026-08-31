# Superficies de dibujo candidatas

Type: research
Status: resolved

## Question

Dado que el usuario **no edita** el diagrama, ¿qué debería pintar la escena ya
posicionada, y qué aporta y qué cuesta cada candidato?

Cubre §26.2, §26.3, §26.4 y §26.12 del handoff.

Candidatos de partida: **tldraw**, **React Flow**, **Excalidraw**, **SVG propio
en React**, y lo que aparezca (Cytoscape, Sigma, mermaid como renderer, ...).

Para cada uno:

- **Licencia y condiciones de distribución.** Crítico en tldraw: hay que
  averiguar exactamente qué exige su licencia hoy — watermark, licencia
  comercial, qué se puede quitar y bajo qué condiciones. Un proyecto que se
  publica en npm no puede descubrir esto tarde.
- **Peso real** del bundle y coste de arranque.
- **Qué se usaría y qué habría que apagar.** tldraw trae editor, toolbar,
  historial, selección, persistencia local: en un visor de solo lectura todo eso
  es superficie que hay que desactivar. ¿Cuánto trabajo es?
- **Qué se obtiene gratis**: zoom/pan, flechas con re-enrutado, hit-testing,
  accesibilidad, exportación a imagen, calidad tipográfica.
- **Capacidad de pintar lo que necesitamos**: nodos con compartimentos de texto,
  contenedores anidados con etiqueta, aristas con estilos distintos por `kind`.
- **Actualización incremental**: ¿se puede mutar la escena sin repintarla entera?

Ojo al sesgo del handoff: §22.9 daba tldraw por decidido. Aquí **no** lo está —
mide, y si tldraw gana, que gane con argumentos.

## Context

Hallazgos: [research/03-superficies-de-dibujo.md](../research/03-superficies-de-dibujo.md)

## Answer

**tldraw queda descartado por licencia, no por gusto. La recomendación es React
Flow (`@xyflow/react`, MIT, ~60 KB gzip + 2,5 KB de CSS).**

### tldraw: por qué se cae, y por qué revoca §22.9 del handoff

Comprobado contra la licencia vigente y contra el código publicado en npm:

- La **"tldraw license" es propietaria**, no open source. Prohíbe usar el
  software en *"Production Environments"*, definidos como cualquier despliegue
  *"used to provide functionality to end users, customers, or the public"*. Un
  paquete npm público que cualquiera arranca con `npx` es exactamente eso.
- Su propia documentación dice que en un proyecto open source *"you **and your
  downstream users** will require their own trial, commercial, or hobby
  license"*. No es solo nuestro problema: se lo trasladaríamos a cada usuario.
- El estado `unlicensed` **pinta un botón "Get a license for production"** sobre
  el canvas — y lo renderiza `TldrawEditor`, no la capa de UI, así que `hideUi`
  no lo quita, la licencia prohíbe taparlo por CSS, y **aparece también en
  `localhost`**. La clase se llama, literalmente, `tl-watermark_SEE-LICENSE`.
- Peso: **523,9 KB gzip** de JS, 16 dependencias en runtime, y fuentes e iconos
  servidos desde `cdn.tldraw.com` — lo que además choca con la restricción de
  página autocontenida.

**Consecuencia para el mapa: §22.9 del handoff ("tldraw se utiliza inicialmente
como renderer") y el paquete `renderer-tldraw` de §19 quedan revocados.** La
decisión de partida 3 —bajar tldraw a candidato en lugar de heredarlo como
decidido— es lo que ha permitido descubrirlo antes de escribir código.

### La decisión real: React Flow contra SVG propio

Fuera tldraw, los demás se caen solos: **Excalidraw** pinta en canvas
(accesibilidad nula), no anida contenedores con etiqueta y arrastra un fallback
a `esm.sh` inextirpable; **Cytoscape** no sabe expresar un nodo con
compartimentos y su export a SVG es GPL-3.0; **JointJS** retiró sus formas UML
en la v4.

Queda React Flow contra SVG propio — y **son la misma arquitectura**: nodos como
componentes React en DOM, aristas en SVG. La pregunta no es "librería o a mano",
sino si vale ~62 KB comprarla hecha.

**Lo que se compra:** la cámara (zoom, pan, `fitView`, pinch, límites) y sobre
todo **la accesibilidad** — Tab entre nodos y aristas, roles ARIA, live regions,
`autoPanOnNodeFocus` — que es justo lo que a mano no se escribe nunca.
**Lo que NO se compra:** compartimentos, marcadores UML, trazado de aristas y
estética. Todo eso lo escribimos igual. El riesgo honesto es acabar pagando
62 KB por una cámara.

**Se mitiga a coste cero**, y esto es condición de la recomendación:

- Los **componentes de nodo son nuestros y agnósticos**: reciben un `VisualNode`
  y devuelven JSX, sin depender de props de React Flow más allá de `data`.
- Las **aristas se pintan con un edge custom que dibuja el `path` que ya calculó
  el motor de layout**, en lugar de `getBezierPath`. Elimina la única carencia
  real de React Flow (no enruta evitando obstáculos) y desacopla el trazado.
- `nodeTypes`/`edgeTypes` fuera del componente y nodos en `React.memo` — es lo
  que hace que el update incremental sea de verdad incremental.

Con eso, SVG propio deja de ser una alternativa y pasa a ser **plan B de media
tarde**: la mitad del trabajo ya estaría escrita.

### La decisión que hay que tomar el día 1

**`<tspan>` o HTML para el texto de los nodos.** HTML (lo que impone React Flow)
da tipografía, elipsis y accesibilidad gratis, pero el export a PNG pasa por
`html-to-image` con las fuentes inlineadas en base64. `<tspan>` da export
fiable, pero los saltos de línea los calculamos nosotros — trivial para una
clase UML, donde es una línea por miembro.

**Como el export a imagen está fuera del MVP (§23), HTML gana sin discusión**, y
con él React Flow. Queda escrito para que no se reabra.

Tabla comparativa completa, licencias verificadas tarball a tarball, y el detalle
de los descartados: [research/03-superficies-de-dibujo.md](../research/03-superficies-de-dibujo.md)
