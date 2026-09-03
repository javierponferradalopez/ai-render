# 22 — ¿Obedece un agente real las 325 palabras?

Prototipo del ticket [#28](https://github.com/javierponferradalopez/ai-render/issues/28).

[#26](https://github.com/javierponferradalopez/ai-render/issues/26) decidió **qué** se le
dice al agente en la descripción de las herramientas, y lo hizo eligiendo a sabiendas la
opción sin enforcement: la norma de la **Apertura pedida** vive sólo en el texto y el
servidor no puede comprobarla. Esto mide **si sirve de algo decirlo**.

No hace falta ventana, ni renderer, ni mmdr. Hace falta un servidor MCP de mentira que
exponga `flipchart_show` y `flipchart_clear` con exactamente el texto de #26 y que, en vez
de dibujar, **apunte lo que le llega**.

## Las piezas

| Fichero | Qué es |
|---|---|
| `servidor-falso.mjs` | El servidor MCP por stdio. Dos variantes de texto, seleccionadas por `FLIPCHART_VARIANT`; apunta cada llamada a `FLIPCHART_LOG` en JSONL. |
| `escenarios.json` | Los guiones de conversación, con qué mide cada uno. |
| `corre.mjs` | Lanza Claude Code en headless multiturno contra el servidor falso y guarda registro y transcripción. |
| `analiza.mjs` | Lee registro y transcripción juntos y saca las cuentas por turno. |
| `intentos.mjs` | **El instrumento del apartado 1.** Cuenta los `tool_use` del historial de Claude Code, y saca los diagramas que viajan dentro. |
| `condiciones/` | Los dos `CLAUDE.md` que se copian al sujeto, y qué mide cada uno. |
| `registros/` | Un JSONL por corrida: los turnos del usuario y las llamadas que cayeron en cada uno. |
| `transcripciones/` | Un JSON por corrida: lo que el agente respondió en cada turno. |

## Las variantes

Las dos primeras son el texto de #26 al pie de la letra, verificado contra
[`peaje-26.py`](../18-mermaid-frente-al-protocolo/peaje-26.py) con `cl100k_base`. Las dos
siguientes nacieron de lo que fue midiendo el prototipo.

| Variante | Qué le dice al agente | Peaje |
|---|---|---:|
| **A** | Lo decidido en #26: cuándo usarla, **ofrece y espera el sí**, y la cláusula de la asimetría | **325** |
| **B** | A sin la cláusula de la asimetría — control de esos 34 tokens | **291** |
| **C** | A **sin la norma de ofrecer y esperar** — control de la Apertura pedida | **295** |
| **D** | A más *«si ibas a dibujar esa estructura en ASCII, dibújala aquí»* — ataca al competidor | **351** |

La diferencia A−B son los **34 tokens** que #26 atribuye a la cláusula. C existe porque, si
el agente no usa la pizarra, hay que saber si lo que le frena es la cortesía. D existe
porque las tres primeras le dicen *cuándo* usarla y **ninguna le dice en vez de qué**.

Hay una quinta condición que no es una variante del texto sino del entorno: un `CLAUDE.md`
en el sujeto que **prohíbe dibujar en ASCII sin mencionar la pizarra**, para ver si el
agente la descubre por su cuenta cuando le quitan el sustituto. Va con el texto A, y sus
corridas se etiquetan `r7`, `r8` y `r9`. Como canal de producto está **fuera de alcance**
—#26 y #18 dejaron la caja con `.mcp.json` y el binario y nada más—, así que sirve de
diagnóstico, no de solución.

**El servidor falso nunca rechaza.** Un rechazo enseñaría dentro de la conversación, y la
variante B no lo tendría: el control dejaría de serlo. Devuelve el acuse + recuento +
vistas vivas que fijó [#11](https://github.com/javierponferradalopez/ai-render/issues/11).

## El sujeto

`pickypen.nvim` — **una copia desechable**, en el scratchpad de la sesión, nunca el repo
de trabajo. Cuatro módulos Lua bajo `lua/pickypen/` (1.410 líneas) con una capa estricta
de verdad: `init` puede requerir a los demás, `ui` requiere `marks`, y ni `store` ni
`marks` requieren nada por encima. Hay una invariante real que entender antes de tocar
nada —quién puede escribir el número de línea de un comentario— y eso es exactamente el
caso protagonista.

**La copia va sin `AGENTS.md` ni `CLAUDE.md`.** No es por comodidad: ese `AGENTS.md` trae
la arquitectura ya escrita, en tabla y en prosa, que es justo lo que la pizarra vendría a
dibujar. Dejarlo sería medir el texto de las herramientas con un segundo canal de
enseñanza al lado — y [#26](https://github.com/javierponferradalopez/ai-render/issues/26)
dejó dicho que ese canal **no existe** en el producto: la caja lleva `.mcp.json` y el
binario y nada más.

El arnés corre con `--strict-mcp-config`, así que el agente ve el servidor falso y ningún
otro.

**El sujeto es siempre una copia.** `corre.mjs` exige `REPO_SUJETO` y no tiene valor por
defecto, y prohíbe `Edit`, `Write` y `NotebookEdit` además de la lista blanca. Las tres
cosas hacen falta: la primera corrida de este prototipo llevaba
`--permission-mode acceptEdits` y el agente, ante un «Sí, adelante» del guion, se puso a
hacer el refactor de verdad sobre el repo sujeto — `acceptEdits` auto-aprueba las
ediciones al margen de `--allowedTools`.

## Cómo se corre

```sh
export REPO_SUJETO=/ruta/a/una/copia/desechable
node corre.mjs A 1              # variante A, repetición 1, los tres escenarios
node corre.mjs B 1 refactor     # sólo un escenario
node analiza.mjs                # las cuentas sobre todo lo que haya en registros/
```

## Lo que se lee, y lo que no

Los apartados 1 y 2 **no tienen control** y con un puñado de conversaciones no hay
significancia estadística de nada. La pregunta no es *qué porcentaje*, es **¿falla de una
forma que se pueda arreglar escribiendo?**

El turno del usuario va **escrito de antemano**, así que el «Sí, adelante» del escenario
`refactor` se escribió sin saber a qué iba a contestar. Eso no se arregla con más código:
se arregla leyendo la transcripción y juzgando qué preguntó el agente, que es exactamente
lo que el ticket pide («se lee el registro contra la transcripción»).

El detector de Nodo fantasma de `analiza.mjs` es heurístico y conservador; los diagramas
que marca se miran a mano.

## El obstáculo del arnés, y qué significa para lo medido

En Claude Code 2.1.228, **`--allowedTools` no concede herramientas MCP en modo `-p`**. Se
probaron el nombre exacto (`mcp__flipchart__flipchart_show`), el prefijo del servidor
(`mcp__flipchart`), el nombre como único valor de la lista, y un
`.claude/settings.local.json` con `permissions.allow` en el sujeto. En los cuatro casos el
agente ve la herramienta, **la intenta**, y recibe *«requested permissions … but you
haven't granted it yet»*. Lo único que la concede es `--permission-mode bypassPermissions`
— verificado end-to-end: dos `show` llegaron al servidor falso, y la copia del sujeto no
cambió ni un fichero, porque `--disallowedTools` se sigue aplicando por encima del bypass.

**Esto no invalida lo medido en el apartado 1**, y la razón es el control positivo: cuando
se le ordena usar la pizarra, el agente **lo intenta**, y ese intento queda registrado en
el historial de la sesión aunque el permiso lo frene. Así que el instrumento del apartado 1
no es el registro del servidor —que sólo ve lo que pasa el permiso— sino **el recuento de
`tool_use` en el historial**, que distingue *no lo intentó* de *lo intentó y no pudo*.

Sí impide medir los apartados 2 y 3, que necesitan que el `show` llegue a completarse.

## Lo que salió

**Apartado 1 — no ofrece. Nunca.**

| Condición | Intentos / turnos |
|---|---:|
| A (325), B (291), C (295), D (351) | **0** / 28 |
| A + `CLAUDE.md` que prohíbe ASCII | **0** / 8 |
| A + `CLAUDE.md` que manda usar flipchart | **8** / 5 |
| *(validez)* «usa la herramienta flipchart» | **9** / 7 |

Cuatro redacciones y cero disparos. Lo que hace en su lugar es pintar el grafo **en ASCII
dentro de la respuesta**, pero eso era el síntoma: el `CLAUDE.md` que lo prohíbe lo apagó
del todo —0 diagramas en 3 corridas, frente a 1–4 por corrida en las demás— y la pizarra
siguió sin usarse, se pasó a prosa con listas. Y **C mata la hipótesis de la cortesía**:
sin la norma de pedir permiso, idéntico resultado.

Con la línea de instrucciones de proyecto salió el caso protagonista sin pedirlo nadie:
`Dependencias actuales`, `Quién sabe de líneas hoy`, `Después · variante A`,
`Después · variante B`. Y los `view_id` en prosa legible, nunca `v1`.

**Apartado 2 — al revés de lo temido.** Nunca pide el permiso de apertura: anuncia y
dibuja, 8 de 8. Pero **sí avisa** en los siguientes sin volver a preguntar.

**Apartado 3 — sin evaluar.** Sin `show` espontáneo no hay diagramas que contar. Lo que
aparece en su lugar, sobre los 17 diagramas intentados:

| Constructo | Diagramas |
|---|---:|
| **HTML en las etiquetas** (`<br/>`, `<b>`) | **15 / 17** |
| Color literal (`rgba()`, `#hex`) | 13 / 17 |
| `style` / `classDef` / `linkStyle` / `click` | 11 / 17 |
| Familia no medida (`sequenceDiagram`) | 4 / 17 |
| `direction` dentro de `subgraph` | 0 / 17 |

**Cerrado después, en [#30](https://github.com/javierponferradalopez/ai-render/issues/30).**
El detector de `analiza.mjs` quedó sin estrenar porque lee `registros`, y ahí no hay ningún
`show`. Pero los 17 diagramas sí están, dentro del `tool_use` del historial: pasados por el
mismo detector con [`asimetria.mjs`](asimetria.mjs) dan **0/17 con Nodo fantasma y 0 ids
desnudos**. El agente no deja ids sin etiquetar nunca — su pulsión es la contraria, adornar
la etiqueta.
