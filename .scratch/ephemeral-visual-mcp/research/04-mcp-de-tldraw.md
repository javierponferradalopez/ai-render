# El MCP de tldraw: arquitectura y coste en tokens

Ticket: [04-mcp-de-tldraw.md](../issues/04-mcp-de-tldraw.md) · Fecha de medición: 2026-08-31

## Nota sobre el método

Todo lo marcado **[VERIFICADO]** sale de hablar JSON-RPC en directo contra el
servidor oficial en producción (`https://tldraw-mcp-app.tldraw.workers.dev/mcp`,
sin autenticación) y de leer el código fuente en `tldraw/tldraw@main`. Los
conteos de caracteres son exactos.

Los **tokens** se cuentan con `tiktoken` / `cl100k_base`. Ese es el tokenizador
de OpenAI, no el de Claude: sirve como proxy con un margen realista de ±10-15 %.
Doy también `chars/4` cuando difiere mucho. Cuando un número es **[ESTIMADO]**
digo exactamente de dónde sale.

---

## 1. ¿Existe un MCP oficial de tldraw?

**Sí, y no es lo que se suponía.** Existe, es oficial, está mantenido en el
monorepo (`apps/mcp-app`) y desplegado en producción. Pero **no expone
herramientas de dibujo**: expone un intérprete de JavaScript.

### 1.1 Hallazgo principal: no hay `create_shape` / `edit_shape` / `delete_shape`

El post del blog de tldraw ([tldraw.dev/blog/tldraw-mcp-app](https://tldraw.dev/blog/tldraw-mcp-app))
dice que el agente tiene "una herramienta para crear shapes, otra para editarlas
y otra para borrarlas". **Eso está desactualizado.** El `tools/list` real del
servidor en producción devuelve seis herramientas, de las cuales solo dos son
visibles al modelo:

| Herramienta | Visibilidad | Qué hace |
|---|---|---|
| `search` | modelo | Ejecuta JS contra un objeto `spec` con la API del Editor de tldraw, para *descubrir* métodos |
| `exec` | modelo | Ejecuta JS arbitrario contra la instancia viva de `editor` en el widget |
| `_exec_callback` | app-only | El widget resuelve una petición `exec` pendiente |
| `_get_canvas_state` | app-only | Último checkpoint por `canvasId` |
| `read_checkpoint` | app-only | Lee shapes de un checkpoint |
| `save_checkpoint` | app-only | Guarda shapes (ediciones del usuario) |

Las cuatro `app-only` llevan `_meta.ui.visibility: ["app"]` y **no deberían**
entrar en el contexto del modelo en un host que implemente MCP Apps. En un host
que ignore ese campo, sí entran (ver §2.1).

### 1.2 Nivel de abstracción: ni semántica ni shapes — código

`exec` no habla de nodos ni de diagramas. Tampoco habla de shapes declarativas.
Habla de **JavaScript sobre el objeto `editor` de tldraw**. Su `inputSchema`
completo es:

```json
{"type":"object",
 "properties":{
   "code":{"type":"string","description":"JavaScript code to execute. Has access to `editor` (tldraw Editor instance) and helper functions."},
   "canvasId":{"type":"string","description":"Canvas ID to edit. Omit to create a new blank canvas..."}},
 "required":["code"]}
```

Dos propiedades. Toda la complejidad está en la *descripción*, no en el esquema:
la descripción trae seis ejemplos de código y la instrucción **"Use the `search`
tool first to discover available Editor methods and shape types"**. Es decir: el
diseño asume que el modelo hará una o más llamadas de descubrimiento antes de
dibujar. Ese es el coste oculto grande (§2.3).

Hay una capa intermedia amable: el widget interpone un *focused editor proxy*
(`src/widget/focused/`) que traduce entre un formato plano para IA (`_type:
'rectangle'`, `shapeId: 'box1'`, ids string simples, `fromId`/`toId` en flechas)
y los `TLShape`/`TLShapeId` internos. Así que `editor.createShape({ _type:
'rectangle', shapeId: 'box1', x, y, w, h, text })` funciona aunque no sea la API
real de tldraw. **Esta idea es lo mejor que tiene el proyecto** (§4).

Superficie de la API expuesta a `search` **[VERIFICADO]**, consultada en vivo:

```
{ "members": 331, "shapes": 25, "helpers": 12, "fullJsonChars": 196960 }
```

331 métodos/propiedades del Editor, 25 tipos de shape, 12 helpers, y el spec
completo pesa **196.960 caracteres (~49k tokens)**. No entra en contexto, pero es
el espacio que el modelo tiene que navegar a ciegas con `search`.

### 1.3 Arquitectura y conexión con el visor

No es WebSocket propio ni servidor de sync. Es **MCP Apps** (la extensión
`ext-apps`, SEP-1865, final desde 2026-01-26):

- **Servidor**: Cloudflare Worker (`src/worker.ts`) + Durable Object `TldrawMCP`
  con SQLite para checkpoints. Transporte **Streamable HTTP**.
- **Widget**: app React (`src/widget/mcp-app.tsx`) que renderiza un lienzo
  tldraw completo **dentro del iframe sandbox del host**. El `exec` declara
  `_meta.ui.resourceUri: "ui://show-canvas/mcp-app.html"`.
- **Puente**: cuando el modelo llama a `exec`, el servidor crea una petición
  pendiente; el widget la recoge, ejecuta el código a través del focused proxy y
  llama a `_exec_callback` para resolverla.
- **Espera acotada**: `EXEC_CALLBACK_WAIT_MS = 4000` (8000 en Cursor/VS Code).
  Si el widget no responde a tiempo, `exec` **no falla**: devuelve un mensaje de
  "está renderizando, el estado llegará en breve".
- **Persistencia**: hasta 50 checkpoints por sesión, DO que se autodestruye a
  los 7 días sin guardar (`IDLE_TTL_MS`).

Verificado en vivo: llamé a `exec` sin ningún widget conectado y devolvió
exactamente esa respuesta degradada (336 chars de texto), creando el canvas
`ebjkzob8`. **Sin host MCP-Apps, el servidor "funciona" pero no dibuja nada y el
modelo nunca ve el resultado.** Fuerte acoplamiento al host.

### 1.4 El detalle que decide el presupuesto

`src/widget/persistence.ts`:

```js
export function pushCanvasContext(app, editor, opts) {
  const shapes = [...editor.getCurrentPageShapes()].map((shape) =>
    convertTldrawShapeToFocusedShape(editor, shape))
  ...
  void app.updateModelContext({
    content: [{ type: 'text', text }],
    structuredContent: { shapes },
  })
}
```

**Tras cada `exec` —y también tras cada edición manual del usuario en el
lienzo— el widget vuelca el canvas ENTERO al contexto del modelo**, en formato
focused. No hay diff, no hay ventana, no hay clustering: `getCurrentPageShapes()`
completo. Se llama en dos sitios (`mcp-app.tsx:695` y `:790`).

Esto es exactamente lo contrario de lo que hace el `agent-template` de tldraw,
que sí tiene BlurryShape / FocusedShape / PeripheralShapeCluster para no
reventar el contexto. El MCP app no heredó esa optimización.

---

## 2. Coste en tokens

### 2.1 Peaje fijo: solo por tener el servidor conectado **[VERIFICADO]**

Serializando cada herramienta como `{name, description, input_schema}` (la forma
en que Claude las recibe):

| Elemento | Chars | Tokens (cl100k) |
|---|---:|---:|
| `search` | 1.128 | **260** |
| `exec` | 1.940 | **554** |
| **Subtotal herramientas visibles** | **3.072** | **816** |
| `_exec_callback` (app-only) | 507 | 141 |
| `_get_canvas_state` (app-only) | 293 | 81 |
| `read_checkpoint` (app-only) | 280 | 76 |
| `save_checkpoint` (app-only) | 494 | 134 |
| **Total si el host no filtra `visibility`** | **4.654** | **1.248** |
| `instructions` del servidor | 358 | 86 |
| `serverInfo.description` | 75 | 17 |

> **Peaje fijo: ~900 tokens** en un host que respeta MCP Apps
> (816 + 86 + 17 = 919), o **~1.350 tokens** en un host que no filtra.

Es un peaje **bajo** en términos absolutos — solo dos herramientas. Comparado con
un MCP típico de 15-30 herramientas (5k-15k tokens) esto es barato. El diseño
"dos herramientas genéricas" gana claramente en el peaje fijo. Pero lo paga con
creces en tiempo de uso.

### 2.2 Dibujar un diagrama de 5 nodos con flechas

**a) El paso de descubrimiento [VERIFICADO — llamadas reales a `search`]**

La descripción de `exec` ordena al modelo usar `search` primero. Lo que cuesta
cada consulta plausible, medido sobre la respuesta real:

| Consulta `search` | Chars | Tokens |
|---|---:|---:|
| `return spec.categories` | 267 | 89 |
| `return spec.types.shapeTypes` | 359 | 124 |
| `return spec.types.shapes.find(s => s.shapeType === "arrow")` | 2.267 | **625** |
| `return spec.members.filter(m => m.category === "shapes")...` | 11.642 | **3.478** |
| `return spec.helpers` (el ejemplo de la propia descripción) | 64.329 | **14.576** |

Ese último no es una trampa: `spec.helpers` es el sitio donde vive
`createArrowBetweenShapes`, y la descripción de `search` sugiere literalmente
`spec.helpers.find(h => h.name === "createArrowBetweenShapes")`. Un modelo que
pida los helpers sin filtrar se come **~14.600 tokens de golpe**. Un modelo
prudente que pida solo las props de `arrow` paga 625.

**Rango realista del descubrimiento: 200 – 3.500 tokens; peor caso plausible, 14.600.**

**b) La llamada `exec` en sí [ESTIMADO]**

No pude capturar la llamada de un modelo real, así que escribí el código
siguiendo literalmente los ejemplos de la descripción de `exec` (bucle sobre un
array de nodos + `createArrowBetweenShapes` + `zoomToFit`) y lo medí:

| Elemento | Chars | Tokens |
|---|---:|---:|
| Código JS del diagrama (5 rects + 4 flechas etiquetadas) | 714 | 276 |
| Bloque `tool_use` completo (`{name, input}`) | 767 | **315** |

**c) El estado de canvas que vuelve [ESTIMADO — reconstruido del fuente]**

Reconstruí el payload de `updateModelContext` campo a campo desde
`to-focused.ts` (rectángulo: `_type, color, fill, h, note, shapeId, text,
textAlign, w, x, y`; flecha: `_type, bend, kind, color, fromId, note, shapeId,
text, toId, x1, y1, x2, y2`):

| Elemento | Chars | Tokens |
|---|---:|---:|
| 1 rectángulo focused | 168 | 67 |
| 1 flecha focused | 182 | 88 |
| **Canvas completo (5 rects + 4 flechas)** | **1.751** | **729** |

**d) La respuesta de la herramienta [VERIFICADO]**

Cuando el widget no contesta en 4 s: 336 chars → **74 tokens**. Cuando sí
contesta, el texto incluye `JSON.stringify(result, null, 2)` del valor devuelto
por el código; si el modelo escribió `return editor.getCurrentPageShapes()`, eso
duplica el canvas entero en contexto.

> ### Total para un diagrama de 5 nodos
> - **Camino bueno** (el modelo no busca o busca poco): 315 + 74 + 729 ≈ **1.100 tokens**
> - **Camino típico** (una `search` dirigida a `arrow`): + 625 ≈ **1.750 tokens**
> - **Camino malo** (el modelo pide `spec.helpers` o los members de shapes): **5.000 – 15.700 tokens**
>
> Y a esto hay que sumarle los ~900 del peaje fijo la primera vez.

### 2.3 Una modificación pequeña **[ESTIMADO, misma reconstrucción]**

Cambiar la etiqueta y el color de un solo nodo:

| Elemento | Chars | Tokens |
|---|---:|---:|
| `exec` con `editor.updateShape({shapeId:'n3', text:'...', color:'red'})` + `canvasId` | 146 | **51** |
| **Re-push COMPLETO del canvas tras la edición** | 1.755 | **730** |
| **Total** | | **~780** |

**El 94 % del coste de una modificación mínima es el re-volcado del lienzo
entero.** Y crece linealmente sin techo:

| Tamaño del lienzo | Chars del estado | Tokens por *cada* operación |
|---|---:|---:|
| 5 nodos + 4 flechas | 1.751 | **729** |
| 20 nodos + 19 flechas | 7.272 | **3.163** |

Una sesión de 10 retoques sobre un diagrama de 20 nodos paga **~31.600 tokens
solo en estados de canvas repetidos**, casi todos idénticos entre sí. Peor: el
push también se dispara con las ediciones manuales del usuario, así que mover
una caja con el ratón mete 3.163 tokens en la conversación.

### 2.4 Comparación con los enfoques baratos **[ESTIMADO, mismo tokenizador]**

El mismo diagrama de 5 nodos, expresado de otras formas:

| Forma | Chars | Tokens |
|---|---:|---:|
| Mermaid `flowchart TD` (solo el texto) | 176 | **69** |
| Mermaid dentro de un `tool_use` | 229 | **88** |
| `visual.show` semántico (`nodes[]` + `edges[]` con labels) | 451 | **178** |
| `exec` de tldraw (§2.2b) | 767 | **315** |
| `visual.update` semántico (un nodo) | 109 | **38** |
| `exec` de tldraw, modificación (§2.3) | 146 | 51 |

En la *llamada*, `exec` cuesta ~1,8x lo que un `visual.show` semántico y ~3,6x lo
que Mermaid. **Pero la diferencia real no está en la llamada, está en el retorno**:
tldraw devuelve 729 tokens de estado donde un MCP efímero puede devolver "ok" (≈10
tokens) o una confirmación corta.

Factor total para el escenario "pintar 5 nodos y hacer 3 retoques":

- tldraw MCP: 919 (peaje) + 1.750 (pintar) + 3×780 (retoques) ≈ **5.000 tokens**
- MCP semántico efímero con confirmación corta: ~250 (peaje, 3 herramientas) + 190 + 3×50 ≈ **590 tokens**

**~8x de diferencia.** Y esa ratio empeora con el tamaño del lienzo.

---

## 3. Otros MCP de dibujo y diagramas

- **Excalidraw** — no hay oficial; hay ~8 implementaciones de terceros. Dos
  familias: (a) CRUD de documentos (`create_drawing`, `get_drawing`,
  `update_drawing`, `export_to_svg/png/json`), esquemas gordos y estado enorme
  porque devuelven el JSON de Excalidraw entero; (b) las que añaden un frontend
  colaborativo con **WebSocket sync** (whallysson, debu-sinha con 14
  herramientas). El patrón (b) es el que más se parece a lo que quiere el ticket
  a nivel de transporte, pero con superficies de herramientas de 8-14 entradas —
  peaje fijo del orden de **3.000-6.000 tokens** solo por conectarlo.
- **Mermaid** — varios (`@peng-shawn/mermaid-mcp-server`, `claude-mermaid`,
  `mermaid-live-mcp-server`, el oficial de Mermaid Chart). Enfoque:
  **una herramienta, un string de texto**, render a PNG/SVG con Puppeteer o vía
  mermaid.ink. `claude-mermaid` añade preview con live-reload — que es
  literalmente el modelo "pizarra efímera" del ticket, pero con Mermaid como
  lenguaje. Es de lejos el enfoque más barato en tokens (§2.4). Su límite es
  expresivo, no económico: no hay posicionamiento libre ni anotación incremental.
- **tldraw `agent-template`** — no es MCP, es un bucle LLM directo en un
  Cloudflare Worker. **Pero es la referencia de diseño de contexto**: distingue
  `BlurryShape` (dentro del viewport: bounds, id, type, text),
  `FocusedShape` (lo que el agente mira de cerca, todas las props) y
  `PeripheralShapeCluster` (fuera del viewport: clusters con bounds y conteo).
  Redondea números antes de mandarlos. Es el trabajo de compresión de contexto
  que el MCP oficial **no** hace.
- **MCP Apps / `ext-apps`** — el estándar (SEP-1865, final 2026-01-26; spec
  publicada 2026-07-28). Widgets HTML en iframe sandbox, JSON-RPC de vuelta al
  host. Soportado por Claude web/desktop, VS Code, Goose, ChatGPT, Copilot M365.
  Es la vía correcta para "el agente ve lo que dibuja" **si** el host la soporta;
  el precio es que sin host compatible no hay visor.

---

## 4. Veredicto: **tomar prestadas ideas, no construir encima**

**No construir encima.** Razones, por orden de peso:

1. **El modelo de contexto es incompatible con un presupuesto de tokens.** El
   re-volcado completo del lienzo en cada operación (§2.3) es una decisión
   arquitectónica del widget, no un parámetro. No se puede desactivar sin
   forkear. Un MCP cuyo coste por retoque crece con el tamaño del dibujo es
   justo lo que el esfuerzo `ephemeral-visual-mcp` quiere evitar.
2. **`exec` es un intérprete, no una herramienta.** Traslada al modelo la carga
   de conocer 331 métodos de una API, y luego le cobra el descubrimiento (hasta
   14.576 tokens en una sola `search`). Las tres herramientas semánticas
   propuestas (`visual.show` / `visual.update` / `visual.clear`) van justo en la
   dirección contraria y es la dirección correcta.
3. **Ejecutar JS arbitrario generado por el modelo** dentro del widget es una
   superficie que un servidor local no necesita.
4. **Acoplamiento al host.** Sin un host MCP-Apps, el servidor de tldraw no
   dibuja nada (verificado). Una pizarra efímera propia puede abrir un navegador
   local y funcionar en cualquier cliente MCP.
5. **Sesión persistente, no efímera.** Durable Objects, 50 checkpoints, TTL de 7
   días, endpoint de purga administrativa. Es lo opuesto a "efímero".

**Qué sí robar, y es bastante:**

- **El formato *focused*** (`src/widget/focused/format.ts` + `to-focused.ts`):
  ids string cortos, `_type` plano, paleta cerrada de 13 colores, 20 tipos geo
  con nombres humanos (`pill`, `cloud`, `fat-arrow-right`), `fromId`/`toId` en
  flechas en vez de bindings. Está pensado explícitamente para que un modelo lo
  escriba sin equivocarse. **Copiar el vocabulario tal cual.**
- **`fromId`/`toId` para flechas.** Que el agente conecte nodos por id y el
  servidor calcule la geometría: elimina las coordenadas del contexto, que es de
  donde sale la mitad del ahorro.
- **Los tres niveles de detalle del `agent-template`** (blurry / focused /
  peripheral) como modelo mental para responder a `visual.show`: devolver un
  resumen, no el estado.
- **La idea de "el ancho de las cajas depende del texto, ten cuidado con los
  solapes"** — resolverla en el servidor con auto-layout en vez de avisar al
  modelo en la descripción de la herramienta, como hace tldraw.
- **El peaje fijo de dos herramientas (~900 tokens) como techo.** tldraw
  demuestra que se puede tener un MCP de dibujo por <1.000 tokens de peaje. Tres
  herramientas semánticas bien descritas deberían quedar por debajo.

**Qué invertir respecto a tldraw:** el retorno. tldraw gasta ~75 tokens en la
respuesta de la herramienta y ~730 en el estado empujado aparte. Un MCP efímero
debe gastar ~10-30 tokens en la confirmación y **cero** en estado, salvo que el
agente lo pida explícitamente.

---

## Fuentes

- `https://tldraw-mcp-app.tldraw.workers.dev/mcp` — `initialize`, `tools/list`,
  `tools/call` (search × 7, exec × 1) en directo, 2026-08-31
- [tldraw/tldraw @ apps/mcp-app](https://github.com/tldraw/tldraw/tree/main/apps/mcp-app) —
  `README.md`, `src/tools/exec.ts`, `src/widget/persistence.ts`,
  `src/widget/focused/to-focused.ts`, `src/widget/focused/format.ts`,
  `src/widget/snapshot.ts`
- [tldraw.dev/blog/tldraw-mcp-app](https://tldraw.dev/blog/tldraw-mcp-app) (desactualizado)
- [tldraw/agent-template](https://github.com/tldraw/agent-template)
- [modelcontextprotocol/ext-apps — spec 2026-01-26](https://github.com/modelcontextprotocol/ext-apps/blob/main/specification/2026-01-26/apps.mdx)
- [peng-shawn/mermaid-mcp-server](https://github.com/peng-shawn/mermaid-mcp-server),
  [veelenga/claude-mermaid](https://github.com/veelenga/claude-mermaid),
  [whallysson/excalidraw-mcp](https://github.com/whallysson/excalidraw-mcp),
  [debu-sinha/excalidraw-mcp-server](https://github.com/debu-sinha/excalidraw-mcp-server)

---

## Respuesta corta

El MCP oficial de tldraw existe pero **no es lo que decía el blog**: no expone
crear/editar/borrar shapes, sino dos herramientas — `search` y `exec` — y `exec`
es un **intérprete de JavaScript** sobre el Editor de tldraw, renderizado en un
widget MCP Apps dentro del iframe del host. **Veredicto: tomar prestadas ideas,
no construir encima** — el widget re-vuelca el lienzo *entero* al contexto tras
cada operación (y tras cada edición manual del usuario), un coste que crece sin
techo con el tamaño del dibujo.

Cifras para la línea base (tokens `cl100k_base`, proxy de Claude ±10-15 %):

- **Peaje fijo [VERIFICADO]: ~900 tokens** (816 de `search`+`exec` + 86 de
  `instructions` + 17). En un host que no filtre `_meta.ui.visibility`, **~1.350**.
  Este es el techo a batir: 3 herramientas semánticas deben caber por debajo de 900.
- **Descubrimiento [VERIFICADO, llamadas reales]: 89 – 14.576 tokens.** Pedir
  `spec.helpers` sin filtrar —ejemplo sugerido por la propia descripción— cuesta
  **14.576**. Consulta dirigida a `arrow`: **625**.
- **Diagrama de 5 nodos [llamada ESTIMADA, retorno VERIFICADO en parte]:
  ~1.100 tokens** en el camino bueno, **~1.750** en el típico, **5.000-15.700**
  si el modelo busca mal. Desglose: 315 la llamada `exec`, 74 la respuesta,
  **729 el estado de canvas empujado**.
- **Modificación pequeña [ESTIMADO]: ~780 tokens**, de los cuales solo **51** son
  la llamada — el **94 % es el re-volcado completo**. Con 20 nodos, cada retoque
  cuesta **3.163**.
- **Referencia de lo barato [ESTIMADO]:** el mismo diagrama son **88 tokens** en
  Mermaid y **178** en un `visual.show` semántico `nodes[]`/`edges[]`; un
  `visual.update` de un nodo, **38**. Escenario completo (pintar + 3 retoques):
  **~5.000 tokens con tldraw frente a ~590 con un MCP semántico efímero, ~8x**.

*Estimados* = payloads reconstruidos campo a campo desde `to-focused.ts` y código
JS escrito según los ejemplos de la descripción de `exec`, medidos con el mismo
tokenizador. *Verificados* = respuestas JSON-RPC reales del servidor en
producción, contadas carácter a carácter.
