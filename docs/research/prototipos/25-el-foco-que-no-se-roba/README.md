# ¿Se puede sacar la ventana delante sin robarle el teclado a nadie?

Prototipo del 2026-09-04 para
[orderFrontRegardless: ¿Dock sin robar el teclado?](https://github.com/javierponferradalopez/ai-render/issues/41),
la pregunta abierta 1 del §11.1. El informe con el veredicto es
[`docs/research/18-el-foco-que-no-se-roba.md`](../../18-el-foco-que-no-se-roba.md).

## Método

`src/main.rs` reparte los hilos como el producto y reproduce la costura entera
del §6.2 —arranque diferido, `Accessory` que sube a `Regular` al construir el
event loop, ⌘W que oculta y no mata—, dejando conmutar **sólo** lo que se
discute: cómo aparece la ventana.

| Bandera | Qué cambia |
|---|---|
| `--appear activate` | `Visible(true)` + `Focus` + `NSApp.activate()` — lo que hacía el producto |
| `--appear key` | igual, sin `activate()`, para aislar quién roba qué |
| `--appear regardless` | `orderFrontRegardless()` y nada más — la apuesta del ticket |
| `--quiet-launch` | desarma el `activateIgnoringOtherApps(true)` que `winit` hace al arrancar |
| `--after-frame` | espera a que se haya pintado un frame antes de mandar la ventana delante |
| `--delay-ms N` | lo mismo por reloj, para separar "no puede" de "ha llegado antes de tiempo" |

Habla **JSON por líneas** —`show`, `close`, `probe`—, no MCP: lo que se mide es
el foco, no el protocolo. `close` llama a **`performClose:`**, que es
literalmente lo que hace ⌘W, no una imitación.

`probe` es la auto-observación: lo que la app sabe de sí misma leído de AppKit
(`isActive`, `isKeyWindow`, `isVisible`, política) más el **orden Z real** de
todas las ventanas de la pantalla, leído del WindowServer con
`CGWindowListCopyWindowInfo` (`src/ventanas.rs`). La sonda del orden Z vive en el
hilo del servidor a propósito: es lo único que sigue siendo verdad cuando el
event loop está congelado, que es lo que pasa con la ventana tapada
(prototipo 14).

| Fichero | Qué hace |
|---|---|
| `medir.py` | el barrido: hace de Claude Code por stdio y recorre los cuatro momentos del ciclo por variante |
| `medir_producto.py` | lo mismo contra el **producto**, hablando MCP de verdad |
| `src/bin/zsonda.rs` | la sonda del orden Z desde fuera, para medir un binario que no tiene diagnóstico |
| `medido.json` | el crudo de la corrida que sostiene el informe (5 variantes × 3 tiradas limpias) |

```sh
CARGO_TARGET_DIR=/tmp/foco cargo build
SPIKE_BIN=/tmp/foco/debug/foco-spike TIRADAS=3 python3 medir.py \
    activate key regardless regardless+quiet regardless+quiet+frame

FLIPCHART=../../../../target/debug/flipchart ZSONDA=/tmp/foco/debug/zsonda \
    python3 medir_producto.py
```

## Lo que se mide, y por qué así

Dos cosas que el ticket mezcla en una, y hay que separarlas:

- **El teclado** — `isActive` / `isKeyWindow` a 30 Hz. Muestreado, porque un foco
  que se roba y se devuelve en 300 ms no se ve mirando sólo al final.
- **La pantalla** — el orden Z **relativo a la ventana del terminal**. El z
  absoluto no sirve: cualquier tercera app que pase por delante lo mueve sin que
  nada de lo que se pregunta haya cambiado.

El escritorio de la corrida es el real, con otras apps entrando y saliendo del
frente, así que cada tirada registra si una tercera app se puso delante mientras
se medía (`limpia: false`) y el informe sólo cuenta las limpias.

## Resultados

En el informe. En una línea: **`orderFrontRegardless` a secas no cambia nada**
—el ladrón del teclado es `winit` al arrancar el event loop—, y mandar la ventana
delante antes del primer frame la deja **detrás del terminal para siempre**. Las
tres piezas juntas sí: delante el 100 % del tiempo, teclado intacto, Dock
conservado.

## Lo que este prototipo NO mide

- **Teclear de verdad.** `System Events keystroke` y `CGEventPost` necesitan
  permiso de Accesibilidad, que esta máquina no tiene concedido. Lo que se mide
  es la causa: el WindowServer entrega el teclado a la app activa.
- **El ⌘W que ya no llega a la ventana**, por el mismo permiso que falta.
- **Pantalla completa, Spaces y Stage Manager.** Ventanas normales, un espacio.
- Build de **debug**, una máquina, macOS 26.6.2 arm64.
