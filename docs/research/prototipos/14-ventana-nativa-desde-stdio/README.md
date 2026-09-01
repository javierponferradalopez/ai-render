# ¿Puede un servidor MCP por stdio abrir su propia ventana?

Prototipo del 2026-09-01 para
[Transporte y ciclo de vida](https://github.com/javierponferradalopez/ai-render/issues/13).
La decisión de partida 2 del [mapa](https://github.com/javierponferradalopez/ai-render/issues/1)
apuesta por *un solo binario de Rust* que sea a la vez servidor MCP y ventana, y
el ticket marcaba la premisa como **hecho a medir, no opinión**:

> un servidor MCP por **stdio** es un proceso hijo que lanza Claude Code, y abrir
> una ventana con foco desde ahí no está verificado: `winit` exige el hilo
> principal en macOS, y un proceso sin *app bundle* puede no aparecer en el Dock
> ni recibir foco.

## Método

`src/main.rs` es un binario que reparte los hilos como lo haría el producto:

```
hilo principal  -> event loop de winit/egui   (obligatorio en macOS)
hilo secundario -> lector de stdin, hace de servidor
```

El servidor habla **JSON por líneas** (`show`, `clear`, `ping`), no MCP: lo que se
mide es la convivencia de los dos hilos y el comportamiento de la ventana, no el
protocolo. `ping` devuelve lo que el hilo principal sabe de sí mismo leído de
AppKit —política de activación, foco, frames pintados, antigüedad del último
frame—, así que el proceso se autoobserva en vez de que lo adivinemos desde fuera.

Los scripts de Python hacen de Claude Code: lanzan el binario **como proceso
hijo con `stdin`/`stdout` en pipe y sin tty**, que es como un host MCP lanza un
servidor por stdio.

| script | qué mide |
|---|---|
| `host.py` | recorrido completo: arranque, primer `show`, dos vistas, `clear`, muerte |
| `diag.py` | si el event loop avanza cuando el hilo del servidor pide repintar |
| `latencia.py` | latencia de repaint con la ventana en segundo plano pero visible |
| `oclusion.py` | lo mismo con la ventana **completamente tapada** por otra |
| `tanda.py` | repetición de lo anterior por variantes (no llegó a ejecutarse) |

```sh
CARGO_TARGET_DIR=/tmp/spike cargo build
SPIKE_BIN=/tmp/spike/debug/pizarra-spike python3 oclusion.py --policy accessory
```

Banderas del binario: `--policy accessory|regular|prohibited`, `--eager` (ventana
al arrancar), `--no-app-nap` (`beginActivityWithOptions`), `--hard-exit` (el reloj
de la muerte en el hilo del servidor).

Medido en macOS 26.6.2 arm64, Rust 1.98.0, `eframe`/`egui` 0.36.1 sobre `winit`
0.30.13. Una tirada por variante: los números son indicativos, los fenómenos
reproducibles.

## Resultados

| Hecho | Resultado |
|---|---|
| Ventana nativa desde un proceso hijo por stdio, sin *app bundle* | **sí** — event loop arriba en 72-350 ms |
| Dock y foco | **sí** — arranca `Accessory` (sin icono, sin foco) y sube a `Regular` + `activate()` al primer `show` |
| Ventana creada perezosamente | **sí** — oculta hasta el primer `show` |
| stdio vivo con la ventana abierta | **sí** — `ping` responde en 0,1 ms mientras el event loop pinta |
| Muerte al cerrarse stdin, ventana a la vista | **sí** — sale solo, código 0, 3,1 s (los 3 del contador) |
| Repaint con la ventana en segundo plano **visible** | 52-55 ms, estable |

### El hallazgo: la oclusión congela el event loop

Con la ventana **completamente tapada** por otra, macOS no ralentiza el event
loop: lo para.

- 4 `show` seguidos, **ninguno repintó en 12 s**.
- La sesión murió y el proceso tardó **11 s en enterarse**, porque el aviso se
  procesa en el `update()` y no había `update()`. Total hasta desaparecer: 14,1 s,
  contra 3,1 s con la ventana visible.
- `beginActivityWithOptions` (App Nap) arregla **la muerte** —1 ms de detección—
  pero **no el repaint**, que siguió errático: >12 s, 9 s, 55 ms, 52 ms.

Que no se repinte mientras nadie mira no es un defecto: el estado vive en el hilo
del servidor y al destapar la ventana se dibuja lo último. El defecto es el otro
lado: **procesos que sobreviven a su sesión** con una pizarra fantasma, y tapada
es el caso normal — el usuario está en su terminal.

De ahí la regla que el ticket adopta: **el event loop no es un reloj**. Lo que
dependa del tiempo o de la muerte de la sesión vive en el hilo del servidor.

### Memoria

| | RSS |
|---|---:|
| Sesión que **nunca** muestra nada (ventana oculta, sin Dock) | 96,8 MB |
| La misma, tras el primer `show` | 97,2 MB |
| Dos sesiones con ventana | 194,8 MB |

**El 99,6 % del coste se paga al crear el event loop, no al mostrar la ventana.**
Es lo que lleva al arranque diferido: que el hilo principal no llame a
`run_native` hasta el primer `show`.

## Lo que este prototipo NO mide

- **`rmcp`**. El hilo del servidor es JSON por líneas, no el SDK. Que un runtime
  de tokio con `rmcp` conviva en el hilo secundario es muy probable y no está
  verificado.
- Build de **debug** (`opt-level = 1`). El grueso de los 97 MB es el contexto
  Metal, no el código, así que release no debería moverlo mucho — sin medir.
- Una sola máquina y un solo sistema operativo.
- La **tanda de repeticiones** (`tanda.py`) se escribió y no se ejecutó.
