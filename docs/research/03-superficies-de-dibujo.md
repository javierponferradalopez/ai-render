# Superficies de dibujo candidatas

> Investigación para el ticket `03-superficies-de-dibujo`, esfuerzo
> `ephemeral-visual-mcp`. Fecha: 2026-08-31.
> Cubre §26.2, §26.3, §26.4 y §26.12 del handoff.
>
> **Convención**: `[V]` = verificado contra fuente primaria (licencia, código
> publicado en npm, docs oficiales, API de bundlephobia). `[I]` = inferido o
> razonado a partir de lo verificado. `[?]` = no confirmado.

## 0. El problema que hay que resolver, acotado

Antes de comparar, conviene fijar qué se le pide realmente a la superficie,
porque cambia radicalmente el ganador:

1. La escena llega **ya posicionada**. El motor de layout (ticket 02) calcula
   `x`, `y`, `w`, `h` de cada nodo y los puntos de cada arista. La superficie
   **no tiene que hacer layout**.
2. El usuario **no edita**. No hay herramientas, ni selección semántica, ni
   deshacer, ni portapapeles, ni creación de shapes por gesto.
3. **No hay persistencia ni colaboración**. Ni IndexedDB, ni sync, ni CRDT, ni
   presencia, ni multi-página.
4. El paquete se **publica en npm** y lo instala cualquiera con `npx`. Corre en
   la máquina del usuario final, en `http://localhost`.
5. Debe funcionar **offline** y sin telemetría: el handoff (§15) vende
   privacidad del código fuente como ventaja.

Lo que sí hace falta de la superficie es un conjunto sorprendentemente pequeño:
**cámara (zoom/pan + fit), hit-testing para hover/tooltip, render nítido de
texto, y mutación incremental barata**. Todo lo demás —herramientas, historial,
bindings, colaboración— es superficie muerta.

Esto invierte el criterio habitual: en este ticket, **"trae mucho" es un
defecto**, no una virtud, salvo que lo que traiga sea exactamente cámara +
texto + hit-testing.

---

## 1. tldraw

### 1.1 Licencia — es el punto que decide

Versión actual en npm: **`tldraw@5.3.2`**, publicada el 2026-08-31, campo
`license: "SEE LICENSE IN LICENSE.md"` `[V]`.

El texto vigente (https://github.com/tldraw/tldraw/blob/main/LICENSE.md,
descargado íntegro) dice, literalmente `[V]`:

> "Production Environment" means any production deployment of the Software that
> operates on servers, cloud platforms, web applications, **or where the
> software is used to provide functionality to end users, customers, or the
> public**. Production Environment excludes internal development.
>
> "Development Environment" means any internal hosting or deployment of the
> Software for development, testing, or staging purposes, operated by your
> organization **and not accessible to end users, customers, or the public**.

Y en las condiciones `[V]`:

> - Not to use the Software in Production Environments.
> - Not to disable, change, or interfere with the Software's License Key enforcement.
> - **Not to make the Software available under a license that supersedes or negates the effect of this License.**
> - Not to distribute the Software or modifications of the Software as a standalone product, but only as part of another application.
> - To include a verbatim copy of this License in any distribution of the Software.

La propia documentación de tldraw responde a nuestro caso exacto, sin
ambigüedad (https://tldraw.dev/community/license) `[V]`:

> "While the tldraw SDK is source available, it is not permissively licensed."
>
> "If you wish to include tldraw in an open source project, you may do so but
> the SDK itself must remain under its original license. **This means that you
> and your downstream users will require their own trial, commercial, or hobby
> license in order to use the SDK in production.**"

Traducción a nuestro proyecto: si publicamos `ephemeral-visual-mcp` en npm con
tldraw dentro, **cada persona que lo instale necesita su propia licencia de
tldraw**. No podemos licenciar nuestro paquete como MIT sin más: la cláusula
"not to make the Software available under a license that supersedes or negates
the effect of this License" lo impide para la parte de tldraw, y el consumidor
hereda la restricción.

**Precios** `[V para la existencia de los tiers, I para la cifra]`:
- *Trial*: 100 días gratis, sin watermark, requiere formulario y clave
  (https://tldraw.dev/sdk-features/license-key).
- *Commercial*: acuerdo anual. La página de precios (https://tldraw.dev/pricing)
  no publica cifra ("value-based pricing"); la cobertura de prensa del cambio de
  la 4.0 cita **6.000 USD/año por equipo**
  (https://biggo.com/news/202509190115_tldraw_SDK_4.0_Licensing_Debate). `[I]`
  — no verificado contra tldraw directamente, que evita publicar el número.
- *Hobby*: gratis, **solo no comercial**, sujeto a aprobación por formulario, y
  **muestra un watermark "made with tldraw"** en el canvas
  (https://tldraw.dev/get-a-license/hobby). `[V]`
- *Startup*: descuento para empresas nuevas, también por solicitud. `[V]`

Historial, para contexto: tldraw fue Apache-2.0 hasta la v2 beta (dic 2023),
pasó a licencia dual, y la **4.0 (sep 2025)** endureció a "solo desarrollo sin
clave; producción requiere clave" (https://tldraw.dev/blog/tldraw-sdk-4-0)
`[V]`. **La trayectoria de la licencia ha sido monótonamente restrictiva
durante tres años.** Eso es un dato para la decisión, no una anécdota. `[I]`

### 1.2 El watermark aparece TAMBIÉN en desarrollo

Este es el hallazgo que más cambia la evaluación, y no está en la documentación
comercial. Verificado leyendo el código publicado en npm
(`@tldraw/editor@5.3.2`, `dist-esm/lib/license/`):

**`LicenseManager.mjs`** — detección de entorno `[V]`:

```js
getIsDevelopment() {
  const protocol = window.location.protocol
  const hostname = window.location.hostname
  if (hostname.toLowerCase().endsWith('.localhost')) {
    return process.env.NODE_ENV !== 'production'
  }
  return protocol === 'http:'
    || (protocol === 'https:' && this.isLoopbackHost(hostname))
    || process.env.NODE_ENV !== 'production'
}
```

Nuestro visor en `http://localhost:PORT` **cae en "development"** aunque el
build sea de producción, porque basta `protocol === 'http:'`. Bien: no pide
clave y no hace la llamada de tracking (`maybeTrack` devuelve pronto si
`isDevelopment`). `[V]`

Pero en `getLicenseState(...)`, sin clave y en desarrollo, el estado resultante
es `"unlicensed"` `[V]`. Y en **`Watermark.mjs`** `[V]`:

```js
if (!['licensed-with-watermark', 'unlicensed'].includes(licenseManagerState)) return null
```

...es decir, **`"unlicensed"` SÍ pinta watermark**. Y no es un logo discreto:
es el componente `UnlicensedWatermark`, un botón con el texto literal
**"Get a license for production"**, `title` = *"The tldraw SDK requires a
license key to work in production. You can get a free 100-day trial license at
tldraw.dev/pricing."*, que al pulsarlo abre `tldraw.dev/pricing`. Se ancla
`position: absolute; bottom; right; z-index: var(--tl-layer-watermark)`. `[V]`

Peor aún para nosotros: el watermark **lo renderiza `TldrawEditor`, no la capa
de UI** (`TldrawEditor.mjs:401` → `jsx(Watermark, {})` dentro del `Layout`, al
mismo nivel que el `Canvas`) `[V]`. Es decir: **`hideUi` no lo quita**, y usar
el editor "pelado" sin la UI tampoco. Solo desaparece con una clave comercial
o de trial válida.

Y quitarlo por CSS está explícitamente vedado. El propio código inyecta este
comentario en la hoja de estilos `[V]`:

```
/* ------------------- SEE LICENSE -------------------
The tldraw watermark is part of tldraw's license. It is shown for unlicensed
or "licensed-with-watermark" users. By using this library, you agree to
preserve the watermark's behavior, keeping it visible, unobscured, and
available to user-interaction.
*/
```

La clase se llama, literalmente, `tl-watermark_SEE-LICENSE`. `[V]`

**Consecuencia práctica**: en el MVP tal y como está descrito en el handoff
(§25: *"el usuario debe sentir que el agente tiene una pizarra temporal"*),
cada visualización que abra el agente llevará pegada abajo a la derecha una
llamada a la acción comercial de tldraw. Es incompatible con el producto que se
quiere. `[I, pero la premisa está verificada]`

### 1.3 Llamadas a red por defecto

`tldraw@5.3.2` **no empaqueta ninguna fuente**: `find` sobre el tarball devuelve
**0 ficheros `.woff*`** `[V]`. Los assets por defecto apuntan a
`https://cdn.tldraw.com` `[V]`:

```js
// dist-esm/lib/utils/static-assets/assetUrls.mjs
tldraw_sans: `${getDefaultCdnBaseUrl()}/fonts/IBMPlexSans-Medium.woff2`,
tldraw_draw: `${getDefaultCdnBaseUrl()}/fonts/Shantell_Sans-Informal_Regular.woff2`,
// dist-esm/lib/ui/assetUrls.mjs
icons:        `${getDefaultCdnBaseUrl()}/icons/icon/0_merged.svg#${name}`,
translations: `${getDefaultCdnBaseUrl()}/translations/${lang.locale}.json`,
```

Para un producto local-first y offline hay que **auto-hospedar** todo eso vía el
paquete `@tldraw/assets` (que existe, misma versión y misma licencia
propietaria `[V]`) y pasar `assetUrls`. Es trabajo conocido y resoluble, pero es
trabajo, y engorda el paquete npm que publicamos.

### 1.4 Peso

| Métrica | Valor | Fuente |
|---|---|---|
| `tldraw@5.3.2` bundle principal | **523,9 KB gzip** / 1,76 MB min | bundlephobia API `[V]` |
| `tldraw.css` | 20,3 KB gzip / 101 KB raw | medido sobre el tarball `[V]` |
| Tarball npm desempaquetado | 14,7 MB, 1.869 ficheros | `npm view` `[V]` |
| Dependencias directas | 16, incluyendo **ProseMirror completo** (vía `@tiptap/*`, ~410 KB solo prosemirror-*), `radix-ui` (9 primitivas), `idb`, `lz-string` | `npm view` + bundlephobia `[V]` |

Media KB de JS comprimido para pintar cajas y flechas que no se pueden editar.
La mayor parte es el editor de texto enriquecido (Tiptap/ProseMirror), los
menús de Radix y el motor de persistencia — cosas que en un visor de solo
lectura no se usan nunca. Con tree-shaking agresivo y `TldrawEditor` sin UI se
podría recortar algo, pero ProseMirror entra por el `TextShapeUtil` y Radix por
los overlays; **no es realista bajar de ~300 KB gzip**. `[I]`

### 1.5 Qué habría que apagar

| Cosa | Cómo | Coste |
|---|---|---|
| Edición | `editor.updateInstanceState({ isReadonly: true })` — bloquea create/delete/update/group/transform/estilos/portapapeles `[V]` | Trivial |
| Toolbar y menús | `hideUi` en `<Tldraw>`, o usar `<TldrawEditor>` directamente `[V]` | Trivial |
| Persistencia | No pasar `persistenceKey`; el store queda en memoria `[V]` | Trivial |
| Selección / indicadores | Custom `ShapeUtil` con indicador vacío, o CSS | Bajo |
| Assets desde CDN | `assetUrls` + `@tldraw/assets` auto-hospedado `[V]` | Medio |
| **Watermark** | **Imposible sin licencia de pago** `[V]` | **Bloqueante** |

Nota: en readonly *"only the select tool, hand tool, and laser pointer remain
visible"* `[V]` — pero eso da igual si además usamos `hideUi`.

### 1.6 Qué se obtiene gratis

Es donde tldraw brilla, y hay que reconocerlo:

- **Cámara de primera**: zoom/pan con inercia, `zoomToFit`, `zoomToSelection`,
  camera constraints, todo bien pulido. `[V]`
- **Store reactivo con señales** (`@tldraw/state`): cada shape es un record y
  los componentes se re-renderizan por shape, no por escena. `editor.run(fn)`
  agrupa cambios en una sola notificación `[V]`. Esto es, con diferencia, lo
  mejor que aporta para nuestro caso de `visual.update` incremental.
- **Bindings**: las flechas se re-enrutan solas cuando se mueve el shape
  vinculado (`onAfterChangeToShape`), y se pueden definir bindings propios con
  `BindingUtil` `[V]`. Útil, aunque en nuestro caso el layout ya calcula las
  rutas.
- **Hit-testing geométrico** vía `getGeometry()` en cada `ShapeUtil` `[V]`.
- **Frames**: contenedor con `props.name` (etiqueta), color, recorte de hijos,
  anidables `[V]`. Encaja bastante bien con "contenedor anidado con etiqueta".
- **Exportación** a SVG/PNG (`exportAs`, `exportToBlob`), permitida incluso en
  readonly porque no muta el documento `[V]`.
- **Custom shapes** con `ShapeUtil` (`component()`, `getGeometry()`,
  `getIndicatorPath()`), renderizando HTML o SVG arbitrario dentro de
  `HTMLContainer` `[V]`.
- v5 añadió sistema de temas, "display values", y overlays renderizados a un
  único canvas HTML con indexado espacial (mejor rendimiento con muchos shapes)
  `[V]`.

### 1.7 Capacidad de pintar lo que necesitamos

Aquí aparece el segundo problema, más sutil que la licencia:

- **Nodo con compartimentos** (clase con nombre + atributos + métodos): las
  shapes por defecto (`geo`, `text`) no lo hacen. Hay que escribir un
  `ShapeUtil` propio cuyo `component()` devuelva nuestro HTML/SVG. `[V/I]`
- **Contenedor anidado con etiqueta**: `frame` sirve, o custom shape. `[V]`
- **Aristas por `kind` semántico** (discontinua, punta de flecha abierta, rombo
  de composición UML): la `arrow` de tldraw tiene su propio estilo "a mano
  alzada" con un conjunto cerrado de puntas. Para rombos UML o dashes por
  semántica hay que **escribir también un `ShapeUtil` de arista**. `[I]`

Es decir: **para pintar diagramas técnicos precisos acabas escribiendo tú todo
el SVG dentro de custom shapes.** Lo que aporta tldraw entonces no es "dibujo",
es "cámara + store + hit-testing" — con 523 KB gzip, una licencia propietaria y
un watermark encima. La estética *hand-drawn* que hace atractivo a tldraw es
justamente lo primero que hay que apagar para un diagrama de arquitectura, y
apagarla implica no usar sus shapes.

### 1.8 Actualización incremental

**Excelente, y es su mejor argumento.** `editor.createShapes` /
`updateShapes` / `deleteShapes` mutan records individuales en un store reactivo
basado en señales; los observadores reciben una sola notificación por lote y
solo se re-renderizan los shapes afectados `[V]`. `editor.run(() => {...})`
agrupa. Mapea casi 1:1 con `VisualPatch` (`add`/`remove`/`update`).

### 1.9 Veredicto tldraw

Técnicamente es la superficie más capaz de la lista. **Y es inviable para este
proyecto por licencia.** Los tres hechos verificados que lo cierran:

1. Los usuarios de nuestro paquete npm necesitarían **su propia licencia** —
   documentado por tldraw para exactamente el caso "open source project".
2. El **watermark comercial se muestra sin clave**, también en localhost, lo
   renderiza el editor (no la UI), y su eliminación está prohibida por licencia.
3. No podemos publicar nuestro paquete bajo una licencia permisiva sin chocar
   con "not to make the Software available under a license that supersedes or
   negates the effect of this License".

Apoyarse en que `protocol === 'http:'` activa el modo desarrollo sería
construir el producto sobre una **laguna de implementación, no sobre un
permiso**. La definición contractual de "Production Environment" ("used to
provide functionality to end users... the public") nos incluye de lleno, y
tldraw ha endurecido su licencia tres veces en tres años. Es riesgo existencial
para un paquete público, y de los que se descubren tarde.

### 1.10 "¿Y si fijamos una versión antigua?" — no funciona

Comprobación hecha: `tldraw@3.15.0` **no congela los términos**. El fichero
`LICENSE.md` que viaja dentro de ese tarball de npm contiene, literalmente y
como única línea `[V]`:

```
This code is licensed under the [tldraw license](https://github.com/tldraw/tldraw/blob/main/LICENSE.md)
```

Es decir, **un enlace a un fichero móvil en `main`**, que hoy contiene el texto
restrictivo de la 4.x/5.x. Y la página que documentaba los términos de la 3.x
(`https://tldraw.dev/legal/tldraw-sdk-3-x-license`, que aún aparece en
resultados de búsqueda) **devuelve 404** `[V]`. No hay un texto histórico
archivado al que agarrarse. Fijar versión no es una salida.

---

## 2. React Flow — `@xyflow/react`

Versión analizada: **12.11.5** (última en npm, agosto 2026).

### 2.1 Licencia

**MIT limpio, sin adendas.** `package.json` publicado declara `"license": "MIT"`
`[V]`, y el `LICENSE` del tarball es el texto MIT estándar íntegro
(`Copyright (c) 2019-2025 webkid GmbH`), idéntico al del repo
(https://github.com/xyflow/xyflow/blob/main/LICENSE) `[V]`.

**La atribución no es una obligación legal.** El componente `<Attribution>`
pinta un enlace "React Flow" en una esquina. En el bundle publicado
(`dist/esm/index.js:141-144`) `[V]`:

```js
if (proOptions?.hideAttribution) { return null }
return jsx(Panel, { className: "react-flow__attribution",
  "data-message": `Please only hide this attribution when you are subscribed to React Flow Pro: ${link}`, ... })
```

Y el tipo `ProOptions` documenta *"**please** support our work by subscribing"*
`[V]`. El lenguaje es de petición, no de condición: `proOptions={{
hideAttribution: true }}` es una prop pública y funcional, sin gating técnico ni
contractual. La página https://reactflow.dev/learn/troubleshooting/remove-attribution
pide no ocultarla sin suscripción, pero **eso no está respaldado por ninguna
cláusula del LICENSE** `[V]`.

Confirmado por el mantenedor (moklick) en
https://github.com/xyflow/xyflow/discussions/3397 `[V]`:

> "You don't need a subscription to use React Flow within a commercial product.
> It's MIT licensed so there is no need to pay anything."

**React Flow Pro** (https://reactflow.dev/pro) es una suscripción de
soporte/contenido — Pro Examples & Templates, issues priorizados, soporte por
email — **no un paquete de código distinto** ni features de pago. Tiers:
$169/$289/mes y enterprise `[V]`. La "Pro License" (https://xyflow.com/pro-license)
cubre exclusivamente ese contenido, no la librería `[V]`.

> **Cero riesgo de distribución.** Podemos publicar en npm como OSS sin pagar
> nada y sin obligar a nadie a licenciar nada. Recomendación pragmática `[I]`:
> dejar la atribución visible por defecto (es un enlace de texto pequeño) y
> exponerla como opción; el coste visual es mínimo y evita fricción.

### 2.2 Peso

| Métrica | Valor |
|---|---|
| `@xyflow/react@12.11.5` | **59,8 KB gzip** / 187,2 KB min `[V]` |
| `dist/base.css` (mínimo, sin tema) | **2,5 KB gzip** / 13,6 KB raw `[V]` |
| `dist/style.css` (con tema) | 3,1 KB gzip / 18,6 KB raw `[V]` |
| Tarball desempaquetado | 1,21 MB, 516 ficheros `[V]` |
| Dependencias directas | **3** (`zustand`, `classcat`, `@xyflow/system`) `[V]` |

Los `d3-*` (~25 KB gzip de `d3-zoom` + `d3-drag` + `d3-transition` + amigos)
entran vía `@xyflow/system` y **no son tree-shakeables**: pagamos `d3-drag`
aunque no arrastremos nada `[V]`.

**~62 KB gzip totales frente a ~544 KB de tldraw: 8,8× menos.** `[V]`

### 2.3 Qué habría que apagar

Trivial — es un objeto de props, ~10 líneas `[V]`:

```tsx
<ReactFlow nodes={nodes} edges={edges} fitView
  nodesDraggable={false} nodesConnectable={false}
  elementsSelectable={false} edgesReconnectable={false}
  connectOnClick={false} zoomOnDoubleClick={false}
  /* panOnDrag, zoomOnScroll, zoomOnPinch: se dejan, son el visor */
/>
```

**`Controls`, `MiniMap` y `Background` no se pintan por defecto**: son
componentes que renderizas tú como hijos. Si no los montas, no existen `[V]`.
**El único residuo de UI es la atribución.** `[V]`

Trampa a documentar `[V]`: `disableKeyboardA11y` debe quedarse en `false`. El
movimiento por flechas ya desaparece solo al poner `nodesDraggable={false}`,
así que ponerlo a `true` solo destruye navegación útil por Tab.

### 2.4 Qué se obtiene gratis

- **Pan/zoom** (d3-zoom) y **`fitView` / `fitViewOptions`** declarativos `[V]`.
- **Hit-testing gratis**: los nodos son DOM, lo hace el navegador. Las aristas
  (SVG) tienen `interactionWidth`, un área invisible alrededor del path para
  hacerlas clicables `[V]`.
- **Accesibilidad seria** — y esto es diferencial: navegación por Tab entre
  nodos y aristas, Enter/Space, Escape; `role="group"` por nodo con override vía
  prop `ariaRole`; `domAttributes` para inyectar `aria-label`/`aria-roledescription`;
  **`ariaLabelConfig`** para localizar todos los mensajes internos; ARIA live
  regions; `autoPanOnNodeFocus`. Los docs afirman alineación WCAG 2.1 AA
  (https://reactflow.dev/learn/advanced-use/accessibility) `[V]`. En el bundle
  se ve `role="application"` en el wrapper y un componente `A11yDescriptions`
  `[V]`.
- **Exportación a imagen**: recipe oficial MIT (no Pro) con `html-to-image` +
  `getNodesBounds` + `getViewportForBounds`, ambos exportados por el paquete
  `[V]`. ⚠️ Los docs obligan a fijar `html-to-image@1.11.11`; versiones
  posteriores no exportan bien `[V]`.
- **Tipografía: los nodos son DOM/HTML, no canvas** `[V]`. Texto real,
  seleccionable, con webfonts, `text-overflow`, y legible por lectores de
  pantalla. Para diagramas de clase densos en texto esto es exactamente lo que
  se quiere.

### 2.5 Capacidad de pintar lo que necesitamos

- **(a) Nodos con compartimentos** ✅ Un custom node es un componente React
  arbitrario. Nombre / atributos / métodos son tres `<div>` con CSS. Es el caso
  de uso natural de la librería, sin fricción `[V]`. Requisito: definir
  `nodeTypes` **fuera** del componente, los docs lo exigen para evitar
  re-renders `[V]`.
- **(b) Contenedores anidados con etiqueta** ✅ Subflows nativos `[V]`:
  `parentId` en el hijo hace su `position` **relativa al padre** — lo que casa
  exactamente con un motor de layout jerárquico externo (ELK produce
  coordenadas así); `extent: 'parent'` confina; existe un tipo `group`
  integrado, y cualquier custom node puede ser padre (→ contenedor **con
  header/etiqueta** propio). Anidamiento multinivel soportado.
  ⚠️ Requisito: *"parent nodes appear before their children in the `nodes`
  array"* — nuestro serializador debe emitir en orden topológico `[V]`.
- **(c) Aristas por `kind` semántico** ✅ Por arista: `type`, `style` (→
  `strokeDasharray` para discontinuas), `markerStart`, `markerEnd`, `label`,
  `labelStyle`, `data`, `zIndex`. `MarkerType` trae `arrow` y `arrowclosed`.
  **Y marcadores SVG propios están explícitamente soportados y documentados en
  los tipos** (`dist/esm/types/edges.d.ts`) `[V]`:

  > *"The id of the SVG marker to use at the end of the edge. This should be
  > defined in a `<defs>` element... Use the format `"url(#markerId)"`."*

  Es decir: **rombo relleno de composición y rombo hueco de agregación UML se
  definen como `<marker>` en un `<defs>` y se referencian**. Herencia (triángulo
  hueco + línea sólida), realización/dependencia (discontinua): todo cubierto.
- **(d) Enrutado**: trae `default` (bezier), `smoothstep`, `step`, `straight`,
  `simplebezier`, con las funciones de path exportadas (`getBezierPath`, etc.)
  `[V]`. **No hay re-enrutado con evitación de obstáculos** y los mantenedores
  han indicado que probablemente no lo añadirán
  (https://github.com/xyflow/xyflow/issues/4766) `[V]`. En nuestra arquitectura
  esto es irrelevante: el motor de layout (ticket 02) ya emite los puntos de
  cada arista, y un edge custom que solo pinta el path recibido es diez líneas
  `[I]`.

### 2.6 Actualización incremental

**Sí, y bien.** `updateNode(id, patch)`, **`updateNodeData(id, dataPatch)`**,
`updateEdge`, `updateEdgeData`, `addNodes`, `addEdges`, `setNodes`, `setEdges`
`[V]`. El estado vive en un **Zustand interno**, fuera del árbol React;
`useReactFlow()` lee sin suscribirse (*"won't cause your component to re-render
when state changes"*) `[V]`. Con `nodeTypes` estático y nodos envueltos en
`React.memo` —los docs lo exigen— un `updateNodeData` puntual re-renderiza solo
ese nodo `[V/I]`.

Mapea limpiamente sobre `VisualPatch`: `add` → `addNodes/addEdges`, `update` →
`updateNodeData`, `remove` → `setNodes(filter)`.

### 2.7 Riesgos y letra pequeña

- **CSS obligatorio** `[V]`: *"You must import the css stylesheet for React Flow
  to work"*. Como publicamos un paquete, o lo empaquetamos nosotros o lo
  documentamos. `base.css` (2,5 KB gzip, sin tema) es probablemente lo que
  queremos, porque estilamos todo nosotros `[I]`.
- **El contenedor necesita `width`/`height` explícitos** `[V]`. Causa clásica de
  "no se ve nada".
- **Rendimiento**: nodos = DOM, el coste escala con elementos DOM y con el CSS.
  Los docs no publican límite numérico pero recomiendan *"simplify CSS styles"*
  (evitar sombras y gradientes). Existe `onlyRenderVisibleElements` (default
  `false`) con la advertencia de que *"also adds an overhead"* `[V]`. Cientos de
  nodos con texto rico habrá que medirlos `[I]`.
- **Peer deps** `react`/`react-dom` `>=17`, con `@types/*` marcados como
  opcionales `[V]`. Rango amplio, sin fricción para un paquete OSS.
- **SSR/SSG soportado desde v12** si se pasan `width`/`height` por nodo `[V]` —
  viable para nosotros porque el layout externo ya los conoce, aunque para un
  visor local no hace falta `[I]`.

---

## 3. Excalidraw — `@excalidraw/excalidraw`

Versión analizada: **0.18.1** (última en npm, agosto 2026). Salvo indicación
contraria, todo lo de esta sección está verificado sobre el **tarball publicado
en npm** (bundle minificado, `.d.ts` publicados, binarios `.woff2`), no sobre la
documentación ni sobre `master`.

### 3.1 Licencia — MIT limpio, pero con deuda de cumplimiento en las fuentes

**El código es MIT** `[V]`: `package.json` declara `"license": "MIT"` y el
`LICENSE` del repo es el texto MIT estándar (`Copyright (c) 2020 Excalidraw`).
Un solo `LICENSE` para todo el monorepo — `excalidraw-app/`,
`packages/excalidraw/` y `packages/element/` no tienen licencia propia `[V]`.
**Sin obligación de atribución, sin watermark, sin licencia comercial.**
Excalidraw Plus es un SaaS aparte y no afecta al paquete npm.

Ahora la letra pequeña, que **sí es un bloqueante blando** si publicamos el
paquete y reempaquetamos sus assets:

| Fuente | Licencia | Nota |
|---|---|---|
| Excalifont, Virgil, Assistant, Lilita One, Nunito, Xiaolai | OFL-1.1 `[V]` | Virgil tiene *Reserved Font Name* |
| Comic Shanns | MIT `[V]` | |
| Cascadia Code | OFL-1.1 upstream, pero el `nameID 13` embebido abre con EULA de Microsoft `[V]` | Ambiguo |
| **Liberation Sans** | ⚠️ **GPLv2 + font exception** `[V]` | Es la **v1.05** (`nameID 5 = "Version 1.05"`); solo Liberation ≥2.00.0 es OFL. **La única fuente no permisiva del paquete** |

Y dos hechos verificados que nos obligan a hacer trabajo:

1. **El tarball de npm no contiene ningún fichero de licencia** — solo
   `README.md`, `package.json` y `dist/` `[V]`.
2. **El proceso de subsetting borró los registros de licencia de los binarios**:
   parseando las tablas `name` de los `.woff2` publicados, Excalifont y Xiaolai
   **no conservan `nameID 13` ni `14`** en ningún subset; Excalifont solo lleva
   `Copyright (c) 2024 by Excalidraw. All rights reserved.` `[V]`

OFL §2 exige que cada copia redistribuida incluya el aviso y la licencia. **Si
reempaquetamos `dist/prod/fonts` heredamos ese incumplimiento** y tendríamos que
añadir nosotros los textos OFL-1.1 y MIT, y excluir Liberation Sans. Es
resoluble en una tarde, pero hay que saberlo antes de publicar.

### 3.2 Peso

Los 46,8 MB desempaquetados son artefacto de empaquetado (`dist/dev` 27 MB +
`dist/types` 2,2 MB + `dist/prod` 17 MB, de los cuales **13 MB son fuentes**).
Lo que realmente llega al navegador, medido con `gzip` sobre el grafo de
imports estáticos reales de `dist/prod/index.js`:

| Pieza | raw | gzip |
|---|---|---|
| `index.js` | 502 KB | **154,7 KB** |
| `chunk-K2UTITRG.js` (import estático) | 439 KB | **151,8 KB** |
| resto de chunks estáticos | 25 KB | 10,4 KB |
| `index.css` | 145 KB | **22,9 KB** |
| **Total en el camino crítico** | ~1,1 MB | **≈339,8 KB** |

`[V]` — corroborado por bundlephobia (352,7 KB gzip para `main`) y, casi al byte, por
el presupuesto del propio proyecto en `.size-limit.json`: `"limit": "340 kB"`
`[V]`.

**`chunk-EIO257PC.js` (1,82 MB raw / 743,1 KB gzip) NO está en el camino
crítico** `[V]`: es **harfbuzz compilado a WASM** (23 símbolos `hb_`),
referenciado solo por `subset-worker.chunk.js` con `import()` dinámico dentro de
un Web Worker, y usado únicamente para subsetear fuentes al exportar SVG.
`exportToSvg({ skipInliningFonts: true })` lo evita del todo.

#### 🔴 Fallback a CDN — el punto que choca con "local-first"

En `chunk-K2UTITRG.js` `[V]`:

```js
P(jn, "ASSETS_FALLBACK_URL", `https://esm.sh/${PKG_NAME}@${PKG_VERSION}/dist/prod/`)
```

y en la resolución de URLs de fuente `[V]`:

```js
static createUrls(t){
  …
  if (typeof window.EXCALIDRAW_ASSET_PATH === "string") { … r.push(new URL(n, o)) }
  r.push(new URL(n, jn.ASSETS_FALLBACK_URL));   // ← SIEMPRE se añade
  return r
}
```

Es decir: **sin `EXCALIDRAW_ASSET_PATH` las fuentes se bajan de `esm.sh`**, y
**aun configurándolo, la URL de esm.sh sigue en la lista** como último recurso.
Offline eso produce un `fetch` fallido y un `console.error`, no un fallo duro,
pero **hay intento de red**. Con CSP estricta hay que declarar `connect-src
'self'` y asumir el ruido.

**Matiz importante a favor**: la carga es perezosa y selectiva
(`getUniqueFamilies(elements)` + `getCharsPerFamily(elements)`), así que solo se
descargan las familias que la escena usa y solo los subsets unicode necesarios
`[V]`. De los 13 MB, **12,7 MB son Xiaolai (CJK, 209 ficheros)**; todo lo demás
junto son **440 KB**, y Excalifont sola son **65 KB en 7 subsets** `[V]`. Para un
visor latino basta con copiar `fonts/Excalifont`. La fuente de UI (Assistant) va
por `@font-face` con URL **relativa** en `index.css`, así que cualquier bundler
la reescribe y funciona offline sin configuración `[V]`.

### 3.3 Modo solo lectura — no queda limpio

`viewModeEnabled?: boolean` existe como prop de primer nivel `[V]`, y el
contenedor raíz recibe la clase `.excalidraw--view-mode`. Extrayendo los
condicionales de render del bundle `[V]`:

| Elemento | En view mode |
|---|---|
| Barra de herramientas, botón Library, panel de propiedades, Stats, undo/redo | ❌ ocultos (`!o.viewModeEnabled && …`) |
| **Controles de zoom** | ✅ **siempre visibles**, sin condición |
| **Botón de ayuda "?"** | ✅ **siempre visible**, sin condición |
| **Menú hamburguesa** | ✅ **siempre visible** |
| Footer | ✅ siempre renderizado |

**Trampa verificada**: `LayerUI` renderiza siempre un `<MainMenu __fallback>`
con items por defecto (LoadScene, SaveToActiveFile, Export, Help, ClearCanvas,
Socials, ToggleTheme…). **No renderizar `<MainMenu>` no lo quita — te deja el
menú por defecto** `[V]`.

**`UIOptions` no sirve para esto** `[V]`: su forma completa es
`{dockedSidebarBreakpoint, canvasActions:{…}, tools:{image}, welcomeScreen}`.
Vacía *items del menú*, no oculta el menú, ni el zoom, ni el "?".

El issue https://github.com/excalidraw/excalidraw/issues/7683 sigue abierto:
*"Currently there is no native way to opt out of the predefined UI"* `[V]`.
**Hace falta CSS** sobre clases internas (`.layer-ui__wrapper__footer`,
`.help-icon`, `.zoom-actions`, `.App-menu_top__left`, `.scroll-back-to-content`).
Estimación: **medio día**, con riesgo de rotura entre versiones porque son
clases sin contrato de estabilidad `[I]`.

`renderTopRightUI(isMobile, appState)` **sí sigue invocándose** en view mode —
es el punto de extensión limpio `[V]`.

### 3.4 Qué se obtiene gratis

- Zoom/pan, hit-testing, gestos táctiles, cursor `grab`, culling por viewport.
- **Retina correcto**: el canvas se dimensiona a `width * devicePixelRatio`,
  acotado a `[1,2,3]` `[V]`. Texto con `fillText` nativo (no paths), así que la
  nitidez es la del navegador — **mejor de lo que sugiere "es canvas"** `[V]`.
- **Dos capas de canvas** (`static` + `interactive`), que aíslan repintado de
  contenido y de interacción `[V]`.
- **Exportación sin montar el componente** — funciones puras sobre `elements`
  `[V]`: `exportToCanvas`, `exportToBlob`, `exportToSvg`, `exportToClipboard`.
- **`convertToExcalidrawElements(skeletons, { regenerateIds: false })`** `[V]`:
  rellena `id`, `seed`, `version`, `versionNonce`, `updated`, `index`, `angle`,
  `groupIds`, `boundElements`, `roundness` y estilos por defecto; cablea
  `label` ↔ `containerId`, `start`/`end` de las flechas, y `children` de frames.
  Imprescindible si se elige esta vía (generar `seed` mal cambia el dibujo).
- **`customData?: Record<string, any>` en todos los elementos** `[V]` — sitio
  natural para colgar nuestros IDs semánticos y hacer hit-testing inverso.
- **`setCustomTextMetricsProvider`** exportado en la raíz `[V]` — permite
  inyectar nuestro propio medidor de texto.
- **Repertorio de puntas de flecha, el mejor de todos los candidatos** `[V]`:
  ```ts
  type Arrowhead = "arrow" | "bar" | "dot" | "circle" | "circle_outline"
    | "triangle" | "triangle_outline" | "diamond" | "diamond_outline"
    | "crowfoot_one" | "crowfoot_many" | "crowfoot_one_or_many"
  ```
  Composición (`diamond`), agregación (`diamond_outline`), herencia
  (`triangle_outline`) y cardinalidades ER de pata de gallo, de fábrica.
  Combinables con `strokeStyle: "solid" | "dashed" | "dotted"`.
- **Elbow arrows**: `ExcalidrawElbowArrowElement` con `elbowed: true` y
  `fixedSegments` `[V]`.

### 3.5 Dónde falla para nuestro caso

1. **Un contenedor = UN texto.** Verificado en la implementación: la búsqueda es
   `boundElements?.find(a => a.type === "text")` — un `find`, no un `filter`—, y
   si ya hay uno, el nuevo se desvincula. El helper público es
   `getBoundTextElementId(container) → string | null`, singular `[V]`.
   → **Una clase UML con compartimentos hay que componerla a mano**: rectángulo
   exterior + N textos posicionados absolutamente + N−1 `line` como separadores,
   agrupados por `groupIds`. **~5-8 primitivas por nodo**, y **se pierde el
   auto-wrap y el auto-alto** que da el binding de texto: la medición pasa a ser
   responsabilidad nuestra `[V/I]`.
2. **Los frames no anidan.** `ExcalidrawFrameElement` tiene `name: string | null`
   ✅, pero `frameId` es de un solo nivel: un frame no puede apuntar a otro
   frame `[V]`. Lo que sí anida es `groupIds` (*"Ordered from deepest to
   shallowest"*) — pero **los grupos no tienen etiqueta** `[V]`. "Contenedor
   anidado con etiqueta" hay que componerlo: rectángulo + texto de cabecera +
   `groupIds` compartidos.
3. **Los bindings NO re-enrutan con `updateScene`.** Este es el matiz que más
   cambia la evaluación. El cuerpo completo de `updateScene`, extraído del
   bundle `[V]`, es:
   ```js
   "updateScene", Fe(t => { let r = Jo(t.elements ?? []);
     if (t.captureUpdate && …) { …store bookkeeping… }
     t.appState && this.setState(t.appState);
     t.elements && this.scene.replaceAllElements(r);
     t.collaborators && this.setState({ collaborators: t.collaborators })
   })
   ```
   **No hay ninguna llamada a recálculo de bindings.** `updateBoundElements` vive
   en el camino interactivo de arrastre y, aunque aparece en los `.d.ts`, **no
   es importable en runtime**: el mapa `exports` define `"./*": { "types": … }`,
   solo tipos, sin `default` `[V]`. Para un visor da igual —el layout ya calcula
   las rutas— pero significa que **`startBinding`/`endBinding` es decorativo en
   nuestro caso**: el re-enrutado no viene gratis.
4. **Accesibilidad floja.** Es un `<canvas>`: el contenido del diagrama **no está
   en el árbol de accesibilidad**. Los botones de UI llevan `aria-label`, pero
   nodos y aristas no son navegables por lector de pantalla. Habría que mantener
   un DOM espejo aparte `[V/I]`.
5. **Estética hand-drawn como identidad** `[I]`. Se puede poner `roughness: 0`,
   pero la fuente por defecto y todo el sistema visual están diseñados para el
   boceto. Para arquitectura técnica es la estética equivocada.
6. **Sin SSR**, `react`/`react-dom` `^17 || ^18 || ^19`, 31 deps transitivas, CSS
   obligatorio, el contenedor necesita dimensiones no nulas, y el CSS global
   define clases genéricas (`.Island`, `.App-menu`) sin aislar `[V]`.

### 3.6 Actualización incremental — mejor de lo que parece

Corrección importante: **`updateScene({elements})` es reemplazo total**
(`this.scene.replaceAllElements`) `[V]`, **pero no es la única vía**. La raíz
exporta **`mutateElement(element, updates, informMutation?, options?)`** `[V]`,
mutación in-place que bumpea `version`/`versionNonce`/`updated` por nosotros (el
tipo `ElementUpdate` los excluye precisamente por eso) y, con
`informMutation = true`, dispara el repintado sin tocar el array.

Y el repintado ya es barato: el canvas estático está memoizado con un
comparador por `sceneNonce`/`selectionNonce`/referencias `[V]`, solo dibuja
`visibleElements`, y hay caché por elemento invalidada por `versionNonce`.

`seed` `[V]` — *"Random integer used to seed roughjs shape generation so the
shape doesn't differ across renders"*: **hay que mantenerlo estable** o el trazo
cambia visualmente en cada repintado.

`captureUpdate` `[V]`: para un visor, siempre `CaptureUpdateAction.NEVER` (*"for
updates which should never be recorded, such as remote updates or scene
initialization"*), evitando que el Store acumule increments.

⚠️ Aviso de versión: en `master` la firma de `mutateElement` cambió (recibe un
`elementsMap`) y se añadió `excalidrawAPI.mutateElement`. **Nada de eso está en
0.18.1** `[V]`.

### 3.7 Veredicto Excalidraw

Licencia del código perfecta, la mejor simbología de flechas del grupo, retina y
tipografía mejores de lo que sugiere "es canvas", y una vía de mutación
incremental real vía `mutateElement`. Pero **es un editor de dibujo a mano
alzada, no un renderer de diagramas estructurados**: un contenedor solo admite
un texto (los compartimentos se componen a mano, y la medición es nuestra), los
frames no anidan, los grupos no tienen etiqueta, el contenido no es accesible, y
el modo lectura necesita medio día de CSS sobre clases internas. Súmale 340 KB
gzip, un fallback a `esm.sh` que no se puede quitar, y deuda de cumplimiento
OFL/GPL en las fuentes si reempaquetamos assets. **Cabe, pero se pelea con
nosotros en casi todo lo que el producto necesita.**

---

## 4. SVG propio en React

La opción de "no usar superficie": un componente que recibe la escena ya
posicionada y emite `<svg>` con `<g transform>`, `<rect>`, `<text>`, `<path>`.
Tiene dos variantes que conviene distinguir, porque deciden cosas distintas:

- **A** — todo SVG, texto en `<tspan>`.
- **A2** — **nodos en HTML posicionados en absoluto + aristas en SVG debajo**.
  (Nótese desde ya: **A2 es exactamente la arquitectura de React Flow**, sin la
  cámara.)

### 4.1 Licencia y peso

Sin licencia ajena. La única dependencia real sería la cámara:

| Opción de zoom/pan | gzip | Licencia | Deps | Nota |
|---|---|---|---|---|
| Implementación propia (`wheel` + pointer events) | **0 KB** | — | 0 | ~40-50 líneas |
| `panzoom` (anvaka) 9.4.4 | 6,2 KB `[V]` | MIT | 3 | Imperativo |
| `svg-pan-zoom` 3.6.2 | 7,9 KB `[V]` | BSD-2 | 0 | Trae `fit()`+`center()`, pero **muta imperativamente un `<g>` viewport → colisiona con la reconciliación de React** |
| `react-zoom-pan-pinch` 4.0.4 | 12,8 KB `[V]` | MIT | **0** | API React nativa, publicado 2026-08-03 |
| `d3-zoom` 3.0.0 | 15,5 KB `[V]` | ISC | 5 | Estándar de facto; te da el `transform`, lo aplicas tú |

**0–16 KB gzip frente a 62 KB (React Flow), 340 KB (Excalidraw) o 544 KB
(tldraw).**

`fitView` es aritmética, no librería:
`k = min(W/bbox.w, H/bbox.h) * margen; x = W/2 − k·bbox.cx; y = H/2 − k·bbox.cy`.
Seis líneas.

### 4.2 Qué hay que construir

| Pieza | Coste | Nota |
|---|---|---|
| Zoom/pan | Bajo | 0–16 KB + ~50 líneas |
| `fitView` | Trivial | Aritmética de bounding box |
| Hit-testing hover/tooltip | **Gratis** | Los elementos son nodos DOM con eventos |
| Compartimentos de clase | ~60-100 líneas de JSX | **Ninguna librería del estudio lo da hecho** salvo mermaid |
| Contenedores anidados con etiqueta | Trivial | `<g>` + `<rect>` + `<text>` |
| Aristas por `kind` | ~40 líneas de `<defs>` | `<marker>` propios + `stroke-dasharray` |
| Texto multilínea | **Medio — el punto doloroso** | Ver abajo |
| Exportación a imagen | Medio | SVG nativo (`XMLSerializer`); PNG con matices |
| Accesibilidad | Medio | WAI-ARIA Graphics Module, ver 4.4 |

### 4.3 Texto: el punto que decide la arquitectura

**Verificado y actualizado a 2026**: el `inline-size` de SVG2 —que daría wrapping
nativo— **sigue sin implementarse en navegadores**. Chromium lo tiene abierto
desde 2014 (https://issues.chromium.org/issues/40362375); Firefox soporta la
propiedad pero el wrap no funciona. El repaso de junio de 2026
(https://patrickbrosset.com/articles/2026-06-22-whats-missing-from-svg/) lo
lista como la carencia principal de SVG: *"real text support for charts and
diagrams: wrapping, overflow handling, sizing, alignment"* sigue exigiendo
line-breaking manual `[V]`.

| Vía | A favor | En contra |
|---|---|---|
| `<tspan>` + saltos calculados | **Export a PNG 100 % fiable**; cero dependencias; control total | Los saltos los calculas tú; sin `text-overflow: ellipsis` nativo |
| `<foreignObject>` + HTML | CSS real: wrap, ellipsis, flex | *"Inconsistent positioning and rendering across browsers"* `[V]`; historial largo de bugs en WebKit; complica el export |
| **A2: HTML absoluto fuera del SVG** | Todo lo bueno del HTML sin los bugs de `foreignObject` | El export a PNG pasa por `html-to-image` |

**Y aquí está el matiz que cambia el cálculo `[I]`: una clase UML no necesita
wrapping.** Cada atributo y cada método es **una línea**, con elipsis si
desborda. Eso reduce el problema a "un `<tspan>` por miembro con `dy` fijo" — el
caso trivial. El wrapping solo haría falta para texto libre (notas).

**Medición**: `ctx.measureText()` sobre un canvas offscreen con la misma `font`
shorthand — rápido y correcto. **No** usar `getComputedTextLength()`/`getBBox()`,
que fuerzan reflow síncrono; medir cientos de nodos así es el cuello de botella
clásico (es justo lo que hace mermaid, §5.3) `[V/I]`. Obligatorio esperar
`await document.fonts.ready` antes de medir, o las cajas salen dimensionadas con
la fuente de reserva `[V]`.

Sinergia con el ticket 02: **el motor de layout ya tiene que medir el texto**
para dimensionar los nodos. Si esa medición viaja en la escena (líneas ya
partidas + anchos), el visor solo pinta y el problema desaparece. Es una
decisión de contrato entre el ticket 02 y este `[I]`.

### 4.4 Exportación a imagen: qué es problema y qué no

**Lo que NO es problema** — desmontando dos mitos que circulan `[V]`:
- El *tainting* del canvas al pintar un SVG same-origin **no ocurre** en
  navegadores modernos: el bug que se cita
  (https://bugzilla.mozilla.org/show_bug.cgi?id=1413978) está **RESOLVED
  INVALID**, era error del reportante.
- Firefox no dibujando SVG sin `width`/`height` intrínsecos
  (https://bugzilla.mozilla.org/show_bug.cgi?id=700533) está **RESOLVED FIXED en
  Firefox 120**.

**Lo que SÍ es problema** `[V]`:
1. **Las fuentes y el CSS externos no se cargan.** Un SVG usado como `<img>`
   corre en *secure static mode* (https://www.w3.org/TR/SVG2/embedded.html,
   https://svgwg.org/specs/svg-native/): **toda carga de recurso externo está
   prohibida**. Hay que inlinear un `<style>` dentro del SVG serializado y
   embeber `@font-face` con la WOFF2 en `data:` URI. Si no, el PNG sale con la
   fuente del sistema y métricas distintas a lo que se ve en pantalla.
2. Hay que poner `width`/`height` **absolutos** en el SVG serializado.
3. `foreignObject` en WebKit tiene historial de bugs. Matiz importante:
   `html-to-image` (MIT) se basa precisamente en `foreignObject` + canvas y
   declara Chrome 49+/Firefox 45+/**Safari 16+** — **funciona, pero exige
   inlinear fuentes e imágenes en base64**, y DOMs grandes revientan el límite
   de tamaño de la data URI `[V]`.

> **Conclusión operativa**: si el export a PNG es requisito, `<tspan>`; si manda
> la riqueza de texto y la accesibilidad, A2 + `html-to-image`. **`foreignObject`
> es el peor de los dos mundos.** Y esta decisión hay que tomarla el día 1,
> porque condiciona toda la arquitectura de texto.

### 4.5 Accesibilidad

Existe el **WAI-ARIA Graphics Module 1.0** (W3C Recommendation, 2018)
(https://www.w3.org/TR/graphics-aria-1.0/) con `graphics-document` (diagramas
técnicos: exactamente nuestro caso), `graphics-object` (subcomponente con
semántica) y `graphics-symbol` `[V]`. Patrón:
`<svg role="graphics-document" aria-labelledby>` + `<title>` + `<desc>`, y cada
nodo `<g role="graphics-object" aria-label tabIndex={0}>`.

**Aviso**: el soporte de navegador para los roles ARIA en SVG, y en particular
para el Graphics Module, es inconsistente `[V]`. El plan robusto es **A2**
(nodos como HTML real → semántica y foco de teclado sin depender del módulo
gráfico) `[I]`. Y ARIA no aporta comportamiento: el orden de foco, las flechas y
`Escape` los cableamos nosotros.

### 4.6 Rendimiento `[I, con cálculo]`

Una clase con 10 miembros ≈ 15-20 elementos SVG. Para **300 nodos + 400
aristas**: ~6.000-8.000 nodos DOM. Manejable con tres reglas:

1. **El zoom no debe re-renderizar React.** Aplicar el transform a un único
   `<g>` (o al `viewBox`), fuera del árbol memoizado. Así pan/zoom es coste GPU,
   no de reconciliación.
2. `React.memo` por nodo, comparando por identidad de la entrada de escena.
3. Por encima de ~2.000 nodos visibles: culling por viewport antes de plantearse
   canvas.

### 4.7 Veredicto SVG propio

Es la opción con menos riesgo estructural y más control, y la que mejor encaja
con "el renderer decide todo". Su coste no es el dibujo —trivial cuando las
coordenadas ya vienen dadas— sino **la cámara, la exportación y la
accesibilidad**. Y esas tres son exactamente lo que React Flow regala por 62 KB,
sobre la misma arquitectura A2.

---

## 5. Otros candidatos, y por qué se descartan

Licencias y versiones verificadas con `npm view`; pesos con la API de
bundlephobia y con medición local `[V]`.

### 5.1 Cytoscape.js — `cytoscape@3.34.2`, MIT, 137 KB gzip

Muy vivo (publicado 2026-08-25, ~15,5 M descargas/semana, **cero dependencias
runtime**). Ponerlo en solo lectura es trivial `[V]`:
`boxSelectionEnabled:false, autoungrabify:true, autounselectify:true,
autolock:true`, y `layout: { name: 'preset' }` respeta posiciones dadas.

Y sus **aristas son de las mejores del estudio** `[V]`: `line-style:
solid|dotted|dashed`, `line-dash-pattern`, arrow shapes `triangle, vee, tee,
square, circle, diamond, chevron, triangle-tee, circle-triangle,
triangle-cross…` más `arrow-fill: filled|hollow`. Todo el vocabulario UML sale:
herencia = `triangle` + `hollow`; realización = idem + `dashed`; composición =
`diamond` + `filled`; agregación = `diamond` + `hollow`.

**Pero tiene dos bloqueantes duros, ambos verificados en sus docs:**

1. **Un nodo = una etiqueta, de un solo estilo.** Las únicas propiedades de
   texto son `label`, `source-label`, `target-label`. Hay `text-wrap`,
   `text-max-width`, `line-height`… pero **`color`, `font-weight` y
   `font-style` aplican a la etiqueta completa** `[V]`. Una clase con nombre en
   negrita, separadores, métodos abstractos en cursiva y visibilidad `+ - # ~`
   **no es expresable**. La salida sería `cytoscape-node-html-label`, cuya
   última release es de **enero de 2021** `[V]` — y que nos devuelve a la
   arquitectura A2 pagando 137 KB de canvas debajo.
2. **Los compound nodes NO tienen dimensiones propias.** Cita literal de los
   docs `[V]`: *"a compound parent node does not have independent dimensions
   (position and size), as those values are automatically inferred by the
   positions and dimensions of the descendant nodes"*. **Esto es incompatible
   con nuestra premisa**: nuestro motor de layout calcula el tamaño de cada
   contenedor y Cytoscape lo sobrescribiría con el bbox de los hijos.

Añádase: **accesibilidad nula** (canvas: sin lector de pantalla, sin texto
seleccionable, sin `Ctrl+F`), y una **mina de licencia**: el export a SVG solo
existe vía `cytoscape-svg@0.4.0`, que es **GPL-3.0** `[V]` — contaminaría un
paquete que publicamos permisivo. `react-cytoscapejs` está en **v2.0.0 de
septiembre de 2022** `[V]`.

**Descartado.**

### 5.2 Sigma.js — `sigma@3.0.3`, MIT, 26 KB gzip

WebGL, *"aimed at visualizing graphs of thousands of nodes and edges"* `[V]`.
Su vocabulario es `circle`/`square`/`image`/`piechart` con **una etiqueta por
nodo**. Sin compartimentos, sin contenedores etiquetados, sin marcadores UML.
Resuelve el problema **opuesto** al nuestro: decenas de miles de nodos con cero
detalle interno; nosotros tenemos cientos de nodos con mucho detalle textual.
**Descartado.**

### 5.3 mermaid como renderer — `mermaid@11.17.2`, MIT, ~198-250 KB gzip

**Corrección a la intuición inicial: mermaid v11 SÍ acepta posiciones propias.**
Tiene un sistema de layout enchufable público `[V]`
(`rendering-util/render.ts`: `interface LayoutAlgorithm`,
`registerLayoutLoaders`), y desde **11.17.0 (2026-08-19)** exporta
`createCommonLayoutRenderer`, `defaultMeasureLayout` y `paintLayoutData` — el
hook `runLayoutCore` es donde se escriben `node.x/y` y `edge.points`. Es lo que
hace `@mermaid-js/layout-elk` internamente. Y `classDiagram` ya usa el pipeline
unificado, registrando `markers: ['aggregation','extension','composition',
'dependency','lollipop']`.

Aun así se descarta, por cuatro razones verificadas:

1. **Mide los nodos él mismo.** `insertMeasuredNode()` hace `insertNode()` +
   `getBBox()` y **sobrescribe `node.width/height`** `[V]`. Nuestros tamaños no
   se respetan. El patrón viable sería de dos pasadas, ejecutando **nuestro
   layout dentro del navegador**, dentro de `runLayoutCore` — lo que rompe la
   arquitectura del esfuerzo.
2. **Cero actualización incremental.** La única API es `render(id, text) →
   {svg: string}`: cada cambio es re-parse + re-medición DOM de cada nodo +
   re-serialización + DOMPurify del SVG entero `[V]`. Incompatible con el §11
   del handoff.
3. **Peso y ausencia de tree-shaking**: bundlephobia da 178,5 KB gzip; el cierre
   estático real es **~198 KB que se pagan siempre** (~250 KB con
   class+flowchart+dagre), porque `diagram-orchestration.ts` importa
   estáticamente los ~35 detectores y el `package.json` no declara
   `sideEffects:false`. El build `.tiny` **no se publica en npm** `[V]`.
4. **`htmlLabels: true` por defecto** → `<foreignObject>` → complica el export a
   PNG (§4.4). Hay que fijar `htmlLabels:false` + `useMaxWidth:false` desde el
   principio, porque cambiarlo después altera todas las métricas `[V]`.

A favor, y por eso merece quedarse como **fallback de exportación**: sintaxis
`classDiagram` completa y correcta, offline verificado (cero `fetch`, cero
`@font-face`, cero CDN), y a11y decente (`role="graphics-document document"`,
`accTitle:`/`accDescr:`) `[V]`.

Nota para el ticket 02: `@mermaid-js/layout-elk@0.2.3` pesa **~511 KB gzip**;
`elkjs@0.12.0` por sí solo pesa **433,4 KB gzip** y es
**`EPL-2.0 OR GPL-3.0-or-later`** `[V]`. Si ELK acaba en el navegador, hay que
elegir EPL-2.0 explícitamente y documentarlo — y su peso supera al de cualquier
superficie de esta lista, lo que es un argumento más para que la superficie sea
barata.

### 5.4 JointJS — plan C real, no descarte rápido

`jointjs@3.7.7` está **DEPRECATED en npm** en favor de **`@joint/core@4.3.2`**,
que es **MPL-2.0** `[V]` (no hubo cambio de licencia: la 3.x ya era MPL-2.0).
MPL-2.0 es copyleft **por fichero**: podemos publicar nuestro visor permisivo
dependiendo de él; solo si modificamos ficheros suyos debemos publicarlos MPL.

A favor `[V]`: **136-140 KB gzip con cero dependencias runtime** (la v4 eliminó
jQuery/lodash/Backbone/dagre/graphlib), **es SVG**, read-only con
`paper.interactive = false`, incremental de verdad (una `CellView` por celda), y
**`@joint/react@4.3.5` es oficial, MPL-2.0 y publicado el 2026-08-24**, con
`<Paper renderElement={...}>` para pintar cada nodo en JSX.

En contra `[V]`: **`joint.shapes.uml.Class` ya no existe en la v4** — `shapes.uml`,
`shapes.erd`, `shapes.fsa` y compañía se retiraron del paquete y quedan como
custom shapes en las demos. Y lo que lo haría atractivo de verdad (routers
ortogonales buenos, anchors, connectors) está en gran parte en **JointJS+, que
es comercial**.

> **Cuándo lo elegiríamos**: si necesitáramos su maquinaria de enrutado
> ortogonal. Como ELK ya nos da los *bend points*, serían 140 KB por dibujar
> `<rect>` y `<path>`. **Plan C.**

### 5.5 Descartes rápidos

| Candidato | Licencia | gzip | Por qué no |
|---|---|---|---|
| **`@antv/g6@5.1.1`** | MIT `[V]` | **390 KB** `[V]` | `index.ts` empieza con `import './preset'` → registra todo; sin `exports` map ni `sideEffects:false` → **no tree-shakeable**. `base-node.ts` solo modela `label`/`halo`/`icon`/`badge`/`port`: **sin compartimentos**. Canvas → a11y nula. Docs primarias en chino |
| **`konva@10.3.2` + `react-konva`** | MIT `[V]` | 54+40 = 94 KB `[V]` | Hit-testing excelente (hit canvas con color-ID, O(1)) y `Konva.Text` con `wrap`+`ellipsis` decente. Pero **a11y nula**, texto no seleccionable ni buscable, **sin export SVG**, zoom/pan a mano, y el repintado es **por capa completa**. Solo tras medir que SVG se cae |
| **`pixi.js@8.20.1`** | MIT `[V]` | 252 KB, 10 deps `[V]` | Motor de juegos. **Rasteriza el texto a textura** → borroso al zoom salvo `resolution` alta (memoria) o atlas MSDF. Nuestro contenido es texto denso a zoom libre. Descarte contundente |
| **`@svgdotjs/svg.js@3.2.8`** | MIT `[V]` | 29 KB `[V]` | **Imperativa por diseño**. En React implica reimplementar a mano la reconciliación que React ya da gratis para `<g>`/`<rect>`/`<text>`/`<path>`. **Pagas 29 KB para perder JSX** |
| **`graphology@0.26.0`** | MIT `[V]` | 13 KB `[V]` | **No es un renderer**, es la estructura de datos. Sensato solo si necesitáramos consultas de grafo; para un visor, un `Map<id, Node>` basta |
| **`@hpcc-js/wasm-graphviz@1.28.0`** | Apache-2.0 el wrapper, **EPL-1.0 el Graphviz de dentro del wasm** `[V]` | **619 KB** `[V]` | Acepta posiciones (`nop`/`nop2` ≡ `neato -n1/-n2`, con `inputscale=72`). Pero: 619 KB por un layout que ya tenemos; salida = blob de SVG plano que hay que parsear y recablear (sin componentes, sin incremental); y **el campo `license` de npm no refleja la EPL-1.0 de dentro** |
| **`d3-graphviz@5.6.0`** | BSD-3 `[V]` | **633 KB** `[V]` | Último publish ago-2024; depende del monolito antiguo `@hpcc-js/wasm ^2.x`. Peor que el anterior en todo |
| **`reaflow@5.4.1`** | Apache-2.0 `[V]` | 799 KB, **17 deps** `[V]` | Arrastra `reablocks` (un design system entero) y `react-use-gesture` (deprecado). **Ejecuta su propio ELK dentro** → duplicaría el motor |
| **`beautiful-react-diagrams@0.5.1`** | MIT | — | Publicado en **noviembre de 2020**. Abandonado |
| **yFiles** | Comercial | — | Licencia cerrada y cara. Descartado por lo mismo que tldraw, sin sus ventajas |

---

## 6. Tabla comparativa

| | **tldraw 5.3.2** | **React Flow 12.11.5** | **Excalidraw 0.18.1** | **SVG propio (A / A2)** | **Cytoscape 3.34** | **@joint/core 4.3.2** | **mermaid 11.17** |
|---|---|---|---|---|---|---|---|
| **Licencia** | Propietaria "tldraw license" `[V]` | **MIT** `[V]` | **MIT** `[V]` | — | MIT (export SVG = **GPL-3.0**) `[V]` | **MPL-2.0** `[V]` | MIT `[V]` |
| **¿Podemos publicar en npm?** | **No sin que cada usuario licencie** `[V]` | Sí | Sí | Sí | Sí (ojo al export) | Sí (copyleft por fichero) | Sí |
| **Watermark / atribución** | **Watermark irremovible, también en localhost** `[V]` | Enlace de texto, ocultable legalmente `[V]` | No | No | No | No | No |
| **JS gzip** | **523,9 KB** `[V]` | **59,8 KB** `[V]` | ≈316,9 KB `[V]` | **0–15,5 KB** `[V]` | 137,0 KB `[V]` | 136-140 KB `[V]` | ~198-250 KB `[V]` |
| **CSS gzip** | 20,3 KB `[V]` | 2,5 KB (`base.css`) `[V]` | 22,9 KB `[V]` | 0 | 0 | 0 | 0 |
| **Deps runtime** | 16 (ProseMirror, Radix, idb) `[V]` | **3** `[V]` | 31 `[V]` | 0 | **0** `[V]` | **0** `[V]` | 22 `[V]` |
| **Assets externos** | **Fuentes/iconos desde `cdn.tldraw.com`; 0 woff en el paquete** `[V]` | Ninguno | Fuentes en el paquete, pero **fallback a `esm.sh` inextirpable** `[V]` | Ninguno | Ninguno | Ninguno | Ninguno `[V]` |
| **Superficie de render** | DOM+SVG+canvas | **DOM (nodos) + SVG (aristas)** `[V]` | **canvas** | **SVG (A) / DOM+SVG (A2)** | canvas | SVG `[V]` | SVG (string) |
| **Trabajo para solo lectura** | Trivial salvo watermark = imposible | **Trivial (~10 props)** `[V]` | ~½ día de CSS sobre clases internas `[V]` | N/A | 4 flags `[V]` | 1 prop `[V]` | N/A (ya lo es) |
| **Zoom/pan + fit gratis** | Excelente | **Sí (`fitView`)** `[V]` | Sí | **No** (0–16 KB + ~50 líneas) | Sí | No | No |
| **Hit-testing** | Geométrico `[V]` | **Gratis (DOM)** `[V]` | Canvas, interno | **Gratis (DOM)** | Canvas, interno | Gratis (DOM) | Gratis (DOM) |
| **Accesibilidad** | Media | **Alta: Tab, roles, `ariaLabelConfig`, live regions** `[V]` | **Nula (canvas)** `[V]` | Media (A) / **Alta (A2)** | **Nula (canvas)** `[V]` | Media | Media `[V]` |
| **Export a imagen** | `exportAs` nativo `[V]` | Recipe MIT + `html-to-image@1.11.11` `[V]` | Nativo `[V]` | SVG nativo; PNG con matices (§4.4) | **PNG nativo; SVG = GPL-3** `[V]` | A mano | Ya es SVG |
| **Nodo con compartimentos** | Custom `ShapeUtil` (SVG a mano) | **Componente React, trivial** `[V]` | ~5-8 primitivas a mano `[V]` | **A mano, fácil** | **NO expresable** `[V]` | A mano (`shapes.uml` retirado en v4) `[V]` | **Sí, nativo** |
| **Contenedor anidado con etiqueta** | `frame` con `name`, anidable `[V]` | **Subflows: `parentId` + `extent`, multinivel** `[V]` | `frame` con `name` pero **no anida**; `groupIds` anida **sin etiqueta** `[V]` | **Libre** | **El tamaño lo impone el bbox de los hijos** `[V]` | `embed()` | Namespaces |
| **Aristas por `kind` semántico** | Custom `ShapeUtil` | `style` + **markers SVG propios** `[V]` | **12 arrowheads, incl. rombos y crowfoot** `[V]` | `<marker>` propios, ~40 líneas | **Excelente, nativo** `[V]` | **Excelente (routers)** | Excelente, nativo |
| **Re-enrutado de flechas** | Sí (bindings) `[V]` | No `[V]` — lo da el layout | **No con `updateScene`** `[V]` | No — lo da el layout | Parcial | Sí (routers, los buenos en JointJS+) | Interno |
| **Actualización incremental** | **Excelente (store por señales)** `[V]` | **Buena (`updateNodeData`, Zustand)** `[V]` | Media: `updateScene` reemplaza, pero hay `mutateElement` `[V]` | **Trivial (React.memo)** | `cy.batch()` | Por `CellView` `[V]` | **Ninguna** `[V]` |
| **Impone su propio layout** | No | **No** | No | No | Sí (compounds) | No | **Sí (mide y sobrescribe w/h)** `[V]` |

---

## 7. Recomendación

### 7.1 tldraw queda descartado, y hay que decirlo en el mapa

El §22.9 del handoff ("tldraw se utiliza inicialmente como renderer") y el
paquete `renderer-tldraw` del §19 **deben revocarse**. No es una preferencia
estética: el producto descrito —un paquete npm público que cualquiera instala
con `npx` y que abre una pizarra limpia— **no se puede construir sobre tldraw
sin obligar a cada usuario a conseguir una licencia y sin un botón "Get a
license for production" pegado al canvas**. Ambos hechos verificados contra la
licencia vigente y el código publicado en npm.

Que el chequeo `protocol === 'http:'` nos deje colar por "development" no es un
permiso, es una laguna. Construir encima de ella es aceptar que un cambio de
tres líneas en `LicenseManager.mjs` puede romper el paquete de todos los
usuarios, en cualquier release menor, de forma retroactiva.

### 7.2 La decisión real: **React Flow vs. SVG propio**

Una vez fuera tldraw, la comparación honesta se reduce a dos, y **son la misma
arquitectura**. React Flow renderiza **nodos como componentes React en DOM y
aristas en SVG** — que es literalmente la variante A2 de la §4. La pregunta no
es "librería o a mano", sino **si vale 62 KB comprarla ya construida**.

Lo que se compra por esos 62 KB:

| Cosa | ¿La escribiríamos igual? |
|---|---|
| Cámara (zoom/pan/`fitView`, inercia, pinch, límites, resize) | Sí, y peor. 0-16 KB de librería + ~50 líneas + los casos raros |
| Accesibilidad (Tab entre nodos y aristas, roles ARIA, `ariaLabelConfig` localizable, live regions, `autoPanOnNodeFocus`) | **No la escribiríamos.** Es el trabajo que siempre se pospone |
| Virtualización (`onlyRenderVisibleElements`) | Probablemente no, hasta que doliera |
| Subflows (`parentId` + `extent:'parent'`, multinivel) | Sí, pero mapea 1:1 con la salida de ELK |
| `interactionWidth` en aristas | Detalle que se olvida y luego se echa de menos |

Y **lo que NO se compra**: compartimentos, marcadores UML, trazado de aristas y
estética. Todo eso lo escribimos igual, en ambos escenarios. Ese es el
contraargumento legítimo a favor de SVG propio, y hay que decirlo sin adornos:
**podríamos acabar pagando 62 KB por una cámara y por accesibilidad.**

**Recomiendo React Flow**, por tres razones:

1. La accesibilidad y la cámara son precisamente lo que peor se hace a mano y lo
   que menos se revisa después. Comprarlas por 62 KB, cuando la alternativa
   descartada pesaba 544 KB, es barato.
2. **La marcha atrás es casi gratis** (§7.3): los componentes de nodo son
   idénticos en ambos mundos.
3. `updateNodeData` sobre un Zustand externo al árbol React mapea limpiamente
   sobre `VisualPatch`, sin store intermedio que mantener.

### 7.3 Cómo mitigar el riesgo de haber elegido mal

El §3 del handoff ya lo exige y aquí es la póliza: **la interfaz
`VisualRenderer` no se toca**, y el adapter (`renderer-reactflow`) traduce
`VisualDocument → { nodes, edges }`. Con tres condiciones concretas:

- **Los componentes de nodo son nuestros y agnósticos**: reciben un `VisualNode`
  y devuelven JSX; no dependen de props de React Flow más allá de `data`. Si
  React Flow estorba, se reutilizan tal cual en un renderer propio y solo hay
  que reescribir la cámara.
- **Las aristas se pintan con un edge custom que dibuja el `path` ya calculado**
  por el motor de layout, en lugar de `getBezierPath`. Elimina la única carencia
  funcional de React Flow (no hay enrutado con evitación de obstáculos) y
  desacopla el trazado del renderer.
- **`nodeTypes`/`edgeTypes` definidos fuera del componente** y nodos en
  `React.memo`: requisito documentado, y además es lo que hace que el update
  incremental sea realmente incremental.

Con eso, "SVG propio" deja de ser una alternativa y pasa a ser el **plan B a
media tarde**: la mitad del trabajo ya estaría escrita.

### 7.4 La decisión que hay que tomar el día 1: `<tspan>` o HTML

Es independiente de la superficie y condiciona todo lo demás (§4.3, §4.4):

- **HTML** (lo que impone React Flow) → tipografía y elipsis del navegador
  gratis, accesibilidad real, **pero el export a PNG pasa por `html-to-image`**,
  que se apoya en `foreignObject` y **exige inlinear las fuentes en base64**.
- **`<tspan>`** → export a PNG 100 % fiable, pero los saltos de línea los
  calculamos nosotros. Para una clase UML esto es trivial: **una línea por
  miembro**, con elipsis si desborda; el wrapping solo haría falta en notas de
  texto libre.

Si el export a imagen no es requisito del MVP —y según el §23 del handoff
*no lo es*— **HTML gana sin discusión**, y con él React Flow. Conviene dejarlo
escrito para que no se reabra.

### 7.5 Prueba decisiva antes de comprometerse (ticket 11)

**2-3 horas por rama.** Montar el nodo más difícil que vayamos a tener —una
clase con 8 atributos y 6 métodos, dentro de un contenedor anidado con etiqueta,
con una arista de herencia (triángulo hueco) y otra de dependencia
(discontinua)— en **React Flow** y en **SVG propio**. Eso decide con datos, no
con argumentos.

Además, verificar en el prototipo:

- **Medición de nodos**: React Flow los mide en el DOM con `ResizeObserver` y
  guarda `measured`. Si nuestro layout ya fijó tamaños, hay que pasarlos como
  `width`/`height` en el nodo para evitar doble medición y un salto visual en el
  primer frame `[I — no verificado en ejecución]`.
- **Orden padre→hijo** en el array `nodes`: requisito documentado, obliga al
  serializador a emitir en orden topológico `[V]`.
- **Rendimiento** con ~200 nodos con compartimentos; los docs avisan de que
  sombras y gradientes en el CSS son el primer cuello de botella `[V/I]`.
- **`await document.fonts.ready`** antes de medir texto, en cualquiera de las dos
  ramas `[V]`.
- **Empaquetado del CSS**: `@xyflow/react/dist/base.css` (2,5 KB gzip, sin tema)
  debería bastar.

### 7.6 Nota para el ticket 02 (motores de layout)

Dos hallazgos de este estudio le afectan directamente:

- **`elkjs@0.12.0` pesa 433,4 KB gzip y es `EPL-2.0 OR GPL-3.0-or-later`** `[V]`.
  Si acaba corriendo en el navegador, pesa más que cualquier superficie de esta
  lista y hay que elegir EPL-2.0 explícitamente y documentarlo. `@dagrejs/dagre`
  son 15,8 KB gzip y MIT `[V]`.
- Si la **medición de texto** se hace en el motor de layout y viaja en la escena
  (líneas ya partidas + anchos), el visor se simplifica mucho en cualquiera de
  las dos ramas.

---

## Respuesta corta

**tldraw está descartado por licencia, no por gusto.** Su licencia vigente prohíbe
el uso en "Production Environments" —definidos como cualquier despliegue *"used to
provide functionality to end users... or the public"*— y su propia documentación
dice que en un proyecto open source *"you **and your downstream users** will require
their own trial, commercial, or hobby license"*. Además, leyendo el código publicado
(`@tldraw/editor@5.3.2`), el estado `unlicensed` **pinta un botón "Get a license for
production"** que renderiza `TldrawEditor`, no la capa de UI: `hideUi` no lo quita,
borrarlo por CSS lo prohíbe la licencia, y sale también en `http://localhost`.

**Recomiendo React Flow (`@xyflow/react`, MIT, 60 KB gzip + 2,5 KB de CSS base).**
Excalidraw y Cytoscape se caen por lo mismo: no saben pintar un nodo con
compartimentos (un contenedor = un texto) y no respetan nuestros contenedores.
La decisión real es React Flow **contra SVG propio**, y son la misma arquitectura
—nodos en DOM, aristas en SVG—, así que la pregunta es si vale 62 KB comprarla
hecha. Digo que sí: lo que se compra es la cámara y **la accesibilidad**, que es
justo lo que a mano no se escribe nunca.

**Principal riesgo:** todo lo visual —compartimentos, marcadores UML, trazado de
aristas— lo escribimos igual, así que podríamos acabar pagando 62 KB por una cámara.
Se mitiga a coste cero manteniendo los componentes de nodo agnósticos de React Flow
y pintando las aristas con un edge custom que solo dibuja el `path` que ya calcula el
layout: si estorba, migrar a SVG propio es reescribir la cámara, no el diagrama.
Confírmese con el prototipo de 2-3 h por rama del §7.5.
