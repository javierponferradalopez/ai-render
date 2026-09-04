# El foco que no se roba: Dock sin teclado, medido

Pregunta abierta 1 del §11.1, disparada por [orderFrontRegardless: ¿Dock sin
robar el teclado?][41]. La apuesta estaba asimétrica: si `orderFrontRegardless`
funciona, **desaparece el precio que acepta el §6.2 sin pagar nada**; si no, la
ventana aparece *detrás* del terminal y el agente dice que ha dibujado mientras
el usuario no ve nada, que es peor.

**Se puede, y se adopta — pero `orderFrontRegardless` a secas no habría bastado
para nada.** El ladrón del teclado no era el `activate()` que el ticket
sospechaba: es **`winit`**, que al arrancar el event loop llama a
`activateIgnoringOtherApps(true)` por su cuenta. Sin desarmarlo, cambiar
`makeKeyAndOrderFront` por `orderFrontRegardless` **no cambia absolutamente
nada** — medido, la app se activa igual, 3 de 3. Y desarmándolo pero mandando la
ventana delante en el mismo instante en que el `show` llega, sale justo el
desenlace peor: la ventana aparece **detrás del terminal y se queda ahí**, 3 de 3.

Lo que funciona son **tres cosas juntas**, y ninguna sobra:

| Pieza | Qué quita |
|---|---|
`with_activate_ignoring_other_apps(false)` | el robo de teclado que hace `winit` al arrancar |
esperar al **primer frame** antes de mandar la ventana delante | que la ventana se quede detrás del terminal |
`orderFrontRegardless()` en vez de `Visible(true)` + `Focus` | el robo de teclado que hacíamos nosotros |

Con las tres: la ventana sale **delante del terminal el 100 % del tiempo** y el
teclado **no se mueve de donde estaba**, en el primer `show` y en el renacer tras
un ⌘W. Y el icono del Dock —la mitad del paquete que se quería conservar— sigue
ahí: la política sigue siendo `Regular` y System Events la sigue viendo como app
normal, 15 de 15 tiradas.

## Nota sobre el método

El instrumento está en el [prototipo 25][p25]: un binario que reparte los hilos
como el producto —event loop en el principal, servidor en el secundario—,
reproduce la costura entera del §6.2 (arranque diferido, `Accessory` que sube a
`Regular` al construir el event loop, ⌘W que oculta y no mata) y deja conmutar
**sólo** lo que se discute: cómo aparece la ventana. `medir.py` hace de Claude
Code: lo lanza como proceso hijo por stdio y sin tty, y recorre los cuatro
momentos que el ticket manda comprobar.

Lo que se mira, a 30 Hz, son dos cosas distintas que el ticket mezcla en una:

- **El teclado**: `NSApp.isActive` e `isKeyWindow`, leídos de AppKit dentro del
  proceso. El muestreo importa — un foco que se roba y se devuelve en 300 ms no
  se ve mirando al final.
- **La pantalla**: el orden Z real del WindowServer
  (`CGWindowListCopyWindowInfo`), medido **relativo a la ventana del terminal**.
  El z absoluto no sirve: cualquier tercera app que pase por delante lo mueve sin
  que nada de lo que se pregunta haya cambiado.

Y el ⌘W del usuario se dispara con **`performClose:`**, que es literalmente lo
que hace ⌘W, no una imitación.

**Por qué el teclado se mide por la causa y no con los dedos.** El ticket pide
comprobarlo «con el terminal en primer plano y escribiendo», y teclear de verdad
—`System Events keystroke`, `CGEventPost`— necesita permiso de Accesibilidad,
que esta máquina no tiene concedido (`osascript no tiene permiso para enviar
pulsaciones de teclas`). La medición que queda es la de la causa, y es más
fuerte que contar caracteres: el WindowServer entrega el teclado a la **app
activa**, así que una app que nunca está activa y cuya ventana nunca es *key* no
puede haberse comido una tecla. Eso se comprueba además desde fuera, con System
Events diciendo quién tiene el teclado tras el `show`.

**El escritorio está vivo, y eso se declara.** Las tiradas corren sobre la
máquina de trabajo real, con Brave, Slack y el calendario entrando y saliendo del
frente. Cada tirada registra si una tercera app se puso delante mientras se
medía; las 15 de la tabla siguiente están **limpias** por ese criterio.

## 1. El barrido: cinco variantes, tres tiradas limpias cada una

`quiet` es `with_activate_ignoring_other_apps(false)`; `frame` es esperar al
primer frame pintado antes de mandar la ventana delante.

| Variante | Primer `show`: ¿roba el teclado? | ¿encima del terminal? |
|---|---|---|
| `activate` — lo que hacía el producto | **sí**, 3/3 | 100 %, 100 %, 21 % |
| `key` — igual, sin `activate()` | **sí**, 3/3 | 22 %, 77 %, 94 % |
| `regardless` — sin `quiet` | **sí**, 3/3 | 100 %, 100 %, 100 % |
| `regardless+quiet` — sin esperar al frame | **no**, 3/3 | **0 %, 0 %, 0 %** |
| `regardless+quiet+frame` | **no**, 3/3 | **100 %, 100 %, 100 %** |

Las trazas del nacimiento lo cuentan mejor que los porcentajes. `Z` es el orden Z
—`^` encima del terminal, `.` detrás, `_` sin ventana en pantalla— y `A` es
`NSApp.isActive`, una muestra cada 33 ms:

```
activate               Z ___^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                       A ....AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

activate (otra tirada) Z ___^^^^^^^^^^^^^^...................................
                       A ....AAAAAAAAAAAAA...................................

regardless+quiet       Z __..................................................
                       A ....................................................

regardless+quiet+frame Z ___^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                       A ....................................................
```

Tres lecturas, en orden de importancia:

**(a) `orderFrontRegardless` no quita el robo por sí solo.** La variante
`regardless` sin `quiet` roba el teclado en las tres tiradas, exactamente igual
que la de hoy. Quien roba es `winit` en `applicationDidFinishLaunching`
—`app.activateIgnoringOtherApps(self.ivars().activate_ignoring_other_apps)`, con
el valor `true` por omisión—, y eso pasa **antes** de que el Visor llegue a
opinar. El ticket buscaba el ladrón en la llamada equivocada.

**(b) Sin esperar al primer frame, la apuesta sale por el lado peor.** `Z` no
levanta nunca del suelo: la ventana está en pantalla y **detrás del terminal
durante los tres segundos enteros**, en las tres tiradas, y ahí se quedaría. Es
el escenario que el ticket describía como razón para descartar. La causa está
leída en la fuente de `eframe`: crea su ventana **oculta** a propósito —«start
hidden until we render the first frame to fix white flash on startup»— y la
muestra él, con `set_visible(true)` —o sea `makeKeyAndOrderFront`— en
`post_rendering`, tras pintar. Un `orderFrontRegardless` anterior a eso lo pisa
el propio `eframe` un instante después.

**(c) El frente que da el foco es peor que el que no lo da.** La variante de hoy
mantiene la ventana delante **sólo mientras está activa**: en 3 de los 6
nacimientos medidos la activación duró menos de un segundo y **la ventana cayó
detrás con ella** —las dos trazas de `activate` de arriba son la misma variante—.
Con `regardless+quiet+frame` la ventana está delante el 100 % del tiempo sin
estar activa nunca: el frente no depende de una activación que macOS puede
deshacer.

## 2. Los cuatro momentos del ciclo

Con la receta completa, y sobre las mismas tiradas limpias:

| Momento | Qué hace la ventana | El teclado |
|---|---|---|
| **1. Primer `show`** | delante del terminal, 100 % 3/3 | intacto, 3/3 |
| **2. `show` sobre ventana en pie** | **no se mueve** | **no se toca**, 3/3 |
| **3. ⌘W del usuario** | desaparece de la pantalla (`isVisible` a false) y el proceso sigue | el terminal lo tenía y lo conserva |
| **4. Primer `show` tras el ⌘W** | renace delante, 89-100 % 3/3 | intacto, 3/3 |

El momento 2 es el que ya estaba decidido y sigue igual: un `show` sobre ventana
abierta no toca el foco **en ninguna** de las cinco variantes. Lo que sí se ve
ahí es un efecto del momento 3 que conviene tener escrito: cuando el usuario
**activa** su terminal, la ventana de la Pizarra pasa detrás, como cualquier
ventana de una app que no está activa, y el siguiente `show` no la vuelve a
traer —por diseño, §6.2—. Eso ya era verdad antes de este cambio.

El renacer del momento 4 no espera nada: la ventana ya pintó su primer frame
hace rato, así que la guarda sólo muerde la primera vez, que es cuando hace
falta.

## 3. Sobre el producto, no sólo sobre el spike

El spike reproduce la costura, pero el veredicto se aplica al binario que se
entrega, así que se comprobó ahí también: `medir_producto.py` habla **MCP de
verdad** por stdio —`initialize`, `notifications/initialized`, `tools/call
show`— y quien mira es `zsonda`, la misma sonda del orden Z sacada a un binario
aparte, porque el producto no tiene comando de diagnóstico y no debe tenerlo.

Tres tiradas seguidas, sin una tercera app en escena en ninguna (la sonda
externa muestrea a 5 Hz — un proceso por muestra —, que basta para ver si la
ventana sube y si se queda):

```
primer show -> ('dibujada', 'Shown as view "actual" (2 nodes, 1 edge)...')
  {"traza_encima": "_^^^^^^^^^^^^^", "encima_al_final": true,
   "app_con_el_teclado": "alacritty", "terceros": []}
  {"traza_encima": "_^^^^^^^^^^^^^", "encima_al_final": true,  ...}
  {"traza_encima": "_^^^^^^^^^^^^",  "encima_al_final": true,  ...}

muere la sesión (EOF en stdin)
  salió con código 0 tras 2.5s
```

La ventana sube y se queda, el teclado no se mueve del terminal, el segundo
`show` no toca nada y la sesión sigue muriendo con su adiós de 2,5 s.

Y la lección de método, que costó una tirada: **una caída del orden Z no dice
nada si no se sabe quién se puso delante**. La primera corrida contra el producto
enseñó la ventana cayendo detrás a los 1,8 s, y lo que había era otra app
entrando en escena; con el registro de terceros puesto, las tres tiradas
siguientes salen limpias.

## 4. Lo que se paga a cambio, que no es cero

El ticket decía «sin pagar nada a cambio». Hay una factura pequeña y hay que
escribirla: **la ventana nace sin el teclado, así que ⌘W no le llega**. Quien vea
aparecer la Pizarra y pulse ⌘W por reflejo se lo va a comer el terminal —donde
⌘W puede cerrarle una pestaña—. Para cerrar la ventana hay que clicarla primero,
que es el gesto normal de cualquier ventana que no tiene el foco, y luego ⌘W o el
botón de cerrar.

Contra lo que se compra —*macOS te roba el teclado a media frase, una vez por
sesión*—, es claramente menor: el robo era involuntario y le pasaba a todo el
mundo; el clic previo sólo lo paga quien quiere cerrar la ventana. Se adopta con
el precio nuevo escrito en el §6.2 y en el §11.4.

## 5. Lo que no se ha medido

- **Las teclas, con los dedos.** Falta el permiso de Accesibilidad para teclear
  de verdad; lo que hay es la medición de la causa (§ Nota sobre el método), y
  el arnés queda escrito para cuando se conceda.
- **El ⌘W que no llega.** El precio del §4 es deducción mecánica —los atajos van
  a la app activa—, no una tirada: enviar un ⌘W necesita el mismo permiso que
  falta.
- **Pantalla completa, Spaces y Stage Manager.** Todo esto se midió con ventanas
  normales en un solo espacio. Una Pizarra que aparece en un espacio que el
  usuario no está mirando no la ve nadie, y eso no se ha probado.
- **Una sola máquina y una sola versión.** macOS 26.6.2 arm64, `eframe` 0.36.1
  sobre `winit` 0.30.13, build de debug. Las dos piezas del mecanismo —el
  `activateIgnoringOtherApps` de `winit` y el `set_visible` tras el primer frame
  de `eframe`— son detalles internos de esas versiones, que el repo pincha; una
  subida de `eframe` hay que volver a mirarla por aquí.

[41]: https://github.com/javierponferradalopez/ai-render/issues/41
[p25]: ./prototipos/25-el-foco-que-no-se-roba/
