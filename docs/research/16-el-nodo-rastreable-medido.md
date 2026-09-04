# El nodo rastreable, medido

Go/no-go disparado el 2026-09-04 por [La regla del nodo rastreable, medida contra
el banco de 63][37], que es el riesgo 2 del §11.2 y el punto 2 de la checklist del
primer día: la pata central del `Límite honesto` estaba **decidida sobre un
mecanismo leído, no ejecutado**.

**Ejecutada, no pasa.** De los 42 casos correctos del banco, el `Límite honesto`
**rechaza 12** — casi tres de cada diez —, y **nueve de esos doce son de la regla
del nodo rastreable**. No son casos raros: son `[*]`, el estado inicial de
`stateDiagram-v2`, que es lo primero que se escribe en un diagrama de estados; y
son cuatro familias enteras más. Es exactamente el desenlace que el §11.2 llamó
*peor que la enfermedad*, y ocurre por la razón que allí se sospechaba: **mmdr
fabrica ids sintéticos legítimos**.

La regla compra lo que prometía —caza cinco de los seis inventos y mata
`radar-beta` sin ninguna lista de familias— pero lo cobra a un precio que el
reparto del §4 no puede pagar.

## Nota sobre el método

El instrumento es `flipchart check`, el subcomando de diagnóstico de [#36]: la
tubería de verdad, entera y en su orden, sin abrir ventana. El banco es el del
[prototipo 21][p21] sin tocar —63 casos, con el parser de **Mermaid 11.12.0**
sobre jsdom como juez de validez— más **nueve sondas nuevas** fuera del banco.
Arnés y sondas: [prototipo 23][p23]; se vuelve a correr con `python3 careo.py`.

Lo que hacía falta añadir era el patrón de medida: **qué pide el §4 de cada
caso**. El reparto dice *lo que se ve de más se rechaza, lo que se ve de menos se
dibuja y se avisa*, así que sobre el banco se traduce en una sola línea —**se
rechaza el invento, se dibuja todo lo demás, fugas incluidas**— y con ella
«rechazo» y «acierto» dejan de ser la misma palabra. Las dos deformaciones quedan
libres: el §4 no dice de qué lado caen.

Un aviso sobre el banco: el único `ParseError` de [research 14][r14] —`<<interface>>`—
ya no lo es. El §3.1 degradó el validador a diagnóstico, así que hoy ese caso entra
por el camino permisivo… y lo rechaza la **otra** regla. Cuenta aparte de los doce.

## 1. Los números

| | |
|---|---|
| Casos del banco | **63** |
| Rechazados por el `Límite honesto` | **21** |
| **Falsos positivos entre los 42 correctos** | **12** |
| — de ellos, de la regla del nodo rastreable | **9** (8 sola, 1 con la asimetría) |
| — de ellos, de la regla de la asimetría | **4** (3 sola, 1 con la rastreable) |
| Inventos cazados | **5 de 6** |
| Fugas rechazadas, que el §4 mandaba dibujar | **1** |
| Rechazos que el §4 mandaba dibujar, en total | **14 de 21** |

Y las **23 familias**, que era el otro barrido pedido: **16 se dibujan y 7 se rechazan**.
Ninguna sorpresa entre las siete —`radar-beta` es el invento que la regla debía matar,
`zenuml` es la deformación que el §4 deja libre, y las cinco restantes son el mismo
mecanismo del §2—, pero cinco de las siete son falsos positivos.

Los doce, con el id que los mata:

| caso | id | regla |
|---|---|---|
| `st-01-note`, `st-03-choice`, `familias/03-state` | `__start_root__` | rastreable |
| `st-02-composite` | `__start_root__`, `__start_Compuesto__` | rastreable |
| `familias/08-journey` | `journey_0` | rastreable |
| `familias/15-quadrant` | `quadrant_0` | rastreable |
| `familias/18-packet` | `packet_0`, `packet_1` | rastreable |
| `familias/22-treemap` | `treemap_0`, `treemap_1`, `treemap_2` | rastreable |
| `cd-07-generic` | `Repositorio~Pedido~` + `Pedido`, `Repositorio` | las dos |
| `cd-06-members`, `cd-11-abstract-static` | `Linea` | asimetría |
| `fc-12-nota-como-nodo` | `Uno`, `Dos` | asimetría |

## 2. El mecanismo que la mata: mmdr sí fabrica ids sintéticos legítimos

Los nueve falsos positivos de la regla rastreable son un solo fenómeno con dos
caras, y ninguna de las dos es un fallo de mmdr: son **nombres internos que el
parser necesita y que el autor no escribió nunca**.

**`[*]`** — el estado inicial y el final de `stateDiagram-v2` no tienen nombre en
el idioma, así que mmdr les pone uno: `__start_root__`, `__end_root__`, y dentro
de un estado compuesto `__start_<estado>__`. Medido aislado en `sonda-09`:

```
stateDiagram-v2          →  nodos: Reposo, __start_root__, __end_root__
  [*] --> Reposo
  Reposo --> [*]
```

El mismo diagrama **sin** `[*]` se dibuja sin problema, así que no muere la
familia por ser la familia: muere **todo diagrama de estados que declare por dónde
empieza**, que es el primero que cualquiera escribe.

**`<familia>_<n>`** — donde el fuente no tiene ids que dar, mmdr los enumera:
`journey_0`, `quadrant_0`, `packet_0`, `treemap_0`. Cuatro familias que
[research 14][r14] §8 dio por buenas —*dibujan lo que se les da*— caen enteras.

**Y aquí está la trampa, que es lo que impide acotar la regla:** el peor invento
del banco se llama **`radar_0`**. Tiene la misma forma exacta que `treemap_0`.
Cualquier excepción por la forma del id —«perdona los `__…__` y los
`<familia>_<n>`»— **deja pasar `radar-beta`**, que es justo lo que el §4.1 se
apuntaba como logro: matarla *sin ninguna lista de familias no soportadas*.
Separar `radar_0` de `treemap_0` exige saber qué familias implementa mmdr de
verdad. Eso **es** la lista de familias, con otro nombre.

## 3. El segundo mecanismo: el careo no sabe leer ids que no son palabras

Dos rechazos más de la regla rastreable son un defecto plano del careo, no del
principio. `named_in` sólo busca el id como token cuando **todos sus caracteres
son alfanuméricos o `_`**; cualquier otro id no se busca, y por tanto «no está en
el fuente» aunque esté escrito ahí mismo:

| fuente | id del `Graph` | qué dice el rechazo |
|---|---|---|
| `class Repositorio~Pedido~ { … }` | `Repositorio~Pedido~` | *not in your source* |
| `dominio.pedido[Pedidodeldominio]` | `dominio.pedido` | *not in your source* |

Los dos son Mermaid válido —genéricos y ids con punto, medidos contra el parser de
11.12— y los dos están **literalmente** en la línea que el mensaje señala. Esto se
arregla careando por frontera de token en vez de por clase de carácter, y **no
salva la regla**: quedarían siete falsos positivos y `stateDiagram-v2` seguiría
muerto.

## 4. Lo que la regla sí compra, y el invento que se le escapa

No todo es contra. Sobre los seis inventos de [research 14][r14] §3:

| invento | id | ¿cazado? |
|---|---|---|
| `flowchart` sin dirección | `flowchart` | **sí**, rastreable |
| `Uno@{ shape: cyl, … }` | `Uno@` | **sí**, rastreable |
| `class Pedido["Pedido de venta"]` | `Pedido["Pedido de venta"]` | **sí**, rastreable |
| `Izquierda <--> Derecha` | `> Derecha` | **sí**, rastreable |
| `radar-beta` | `radar_0` | **sí**, rastreable |
| prosa suelta en `flowchart` | `esto` | **no: se dibuja** |

El careo **sin la primera línea** hace lo que se le pidió: caza el nodo `flowchart`
que mmdr fabrica de una cabecera sin dirección. Y `radar-beta` muere sin lista de
familias, tal como el §4.1 lo escribió.

El sexto escapa por lo que la regla es: el nodo que mmdr fabrica de la línea
`esto es una frase que nadie queria dibujar` se llama **`esto`**, que es una
palabra del fuente. Un id rastreable puede seguir siendo un nodo que nadie
escribió. Y es el peor de los seis para dejar suelto: es el único caso del banco
que **Mermaid rechaza** y mmdr dibuja.

## 5. Las sondas: tres hipótesis del §11.2 refutadas, dos confirmadas

Las nueve sondas son Mermaid válido según el parser de 11.12. Cinco acaban en
rechazo.

**Refutadas** — no fabrican ningún id sintético, y la sospecha del §11.2 sobre
ellas era infundada:

- **`subgraph` sin id declarado** (`subgraph Grupodelanzamiento`, y con título de
  varias palabras): el grupo llega como `Subgraph`, **no como nodo**. La regla ni
  se entera.
- **Participante implícito de `sequenceDiagram`**: `Alicia->>Bruno` sin
  `participant` da los ids `Alicia` y `Bruno`, que están en el fuente.
- **Ids acentuados**: `Añadir`, `Confirmación` — `char::is_alphanumeric` los
  admite, así que el careo los ve.

**Confirmadas**: los ids con `.` y los genéricos con `~` (§3), y el `[*]` (§2).

## 6. El solape: no salta una primero, saltan las dos a la vez

La pregunta era cuál de las dos reglas se adelanta cuando se pisan. La respuesta
es que **no compiten**: el §4.1 las manda informar juntas y en un solo rechazo, y
eso es lo que hace. Sobre `sonda-06` —una clase genérica con cuerpo y otra clase
con cuerpo— el mensaje sale así:

```
2 nodes appear in the drawing that you did not declare.
  "Repositorio"          line 2  — only used in a relation
  "Repositorio~Pedido~"  line 2  — not in your source
```

**Las dos causas hablan de la misma clase**, que el autor escribió una sola vez, y
le piden dos cosas incompatibles: *declara el id* y *reescribe la línea*. Con la
regla rastreable en pie, el solape produce mensajes que no se pueden obedecer.

En el caso que el §11.2 nombraba —`classDiagram` con una relación a una clase
nunca declarada, `sonda-07`— **sólo salta la asimetría**, que es lo correcto:
`Moneda` está en el fuente, así que la rastreable no tiene nada que decir. Ese
reparto funciona.

## 7. De rebote: la regla de la asimetría también tiene falsos positivos

No es el objeto de este go/no-go, pero sale en la misma corrida y hay que
escribirlo: **cuatro de los doce falsos positivos son suyos**, y son de dos clases
distintas.

**Un defecto plano** — `cd-06` y `cd-11`: `class Linea` **es una declaración**, y
`declares_itself` no lo sabe. Sólo mira etiqueta, forma, o un `[`/`(`/`{` detrás
del id en alguna línea; una clase declarada a secas no tiene nada de eso, así que
un `classDiagram` con una clase con cuerpo y otra sin él se rechaza siempre. Es el
diagrama de clases más común que hay.

**Una decisión que hay que mirar con el dibujo delante** — `fc-12`:

```
flowchart TB
  Uno --> Dos
  Uno -.-> Notaaparte[Aclaracionalmargen]
```

Aquí la regla hace **exactamente lo que se le pidió** (`API[API Layer] --> Db` →
rechazado): hay un nodo con etiqueta, luego los desnudos son fantasmas. Pero
mezclar ids desnudos con nodos etiquetados es Mermaid corriente, no un descuido, y
esto lo rechaza entero.

## 8. Un séptimo invento que el banco no tenía: el guion

`sonda-03` iba a probar el tokenizado y encontró otra cosa. `api-gateway` no es un
id para mmdr:

```
flowchart TB                            →  nodos: api, gateway[Puertadeentrada],
  api-gateway[Puertadeentrada]              cola, de, eventos[Coladeeventos]
    --> cola-de-eventos[Coladeeventos]
```

Dos nodos escritos se convierten en cinco, tres de ellos fabricados de trozos de
un nombre. Es Mermaid válido y es un invento de manual. **Lo caza la asimetría**,
no la rastreable —los trozos sí son tokens del fuente—, así que el usuario está
cubierto; pero el mensaje le echa la culpa por no declarar `api`, `cola` y `de`,
que él nunca quiso escribir.

## 9. Veredicto: se cae

**La regla del nodo rastreable no vive tal como está redactada, y no se puede
acotar sin comprarse lo que se escribió para no comprar.**

- Doce falsos positivos sobre 42 correctos, nueve suyos, y entre ellos
  `stateDiagram-v2` con `[*]` y cuatro familias enteras. El §11.2 puso el listón
  aquí: *un falso positivo hace que la pizarra no dibuje nunca un tipo de diagrama
  entero, y el agente no insiste: se pasa a prosa y no lo dice*. Ocurre, y cinco
  veces.
- La acotación por la forma del id sintético (§2) libera `radar-beta`, que es el
  invento que la regla mejor mataba; distinguirlas pide la lista de familias que el
  §4.1 presume no necesitar. **La rebaja no está disponible.**
- Arreglar el tokenizado (§3) es correcto y barato, y deja el problema intacto.
- Y aun perfecta, se le escapa uno de los seis inventos (§4), porque un id
  rastreable puede ser un nodo apócrifo.

Lo que vuelve a estar abierto es **qué se hace con los seis inventos** —no el
reparto del §4, que sigue en pie: lo que se ve de más se rechaza—. Lo que este
censo deja sobre la mesa para esa decisión:

1. **Cinco de los seis se ven en el `id`, pero el careo contra el fuente no es la
   forma de verlo.** Tres (`Uno@`, `> Derecha`, `Pedido["Pedido de venta"]`) traen
   en el id **caracteres que ningún id de Mermaid lleva**: comillas, corchetes,
   `@` final, un `>` inicial. Eso es una pregunta sobre la forma del id, no sobre
   el fuente, y no fabrica falsos positivos sobre ids sintéticos porque
   `__start_root__` y `treemap_0` son nombres perfectamente formados.
2. **El nodo de la cabecera** (`flowchart`) es un caso de una línea: el id
   coincide con la primera palabra del fuente.
3. **`radar-beta` y la prosa suelta** no se distinguen por el id, y se quedan sin
   cazador. Para `radar-beta` el dato es que su nodo trae **el fuente entero** en
   la etiqueta.
4. **La regla de la asimetría sigue en pie y hace la mitad del trabajo** —caza el
   guion de §8 y la clase no declarada de §6— pero necesita saber que `class X` es
   una declaración (§7) antes de que se pueda contar con ella.

## 10. Lo que no se ha medido

- **Si las alternativas del §9 se sostienen.** Este censo mide la regla que había,
  no la que venga; lo que deja son los datos para elegirla.
- **La regla de la asimetría contra el banco entero.** Sus cuatro falsos positivos
  salen de esta corrida de rebote; nadie ha ido a buscarlos.
- **Diagramas grandes y diagramas espontáneos.** El banco son constructos mínimos,
  uno por constructo.

[37]: https://github.com/javierponferradalopez/ai-render/issues/37
[#36]: https://github.com/javierponferradalopez/ai-render/issues/36
[r14]: ./14-lo-que-mmdr-traga.md
[p21]: ./prototipos/21-lo-que-mmdr-traga/
[p23]: ./prototipos/23-el-nodo-rastreable-medido/
