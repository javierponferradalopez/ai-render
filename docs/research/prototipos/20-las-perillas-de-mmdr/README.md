# El barrido de perillas de mmdr

Scripts del prototipo de [¿Curan las perillas los grupos de mmdr?][25]. Informe con
los resultados: [research 13](../../13-las-perillas-de-mmdr.md).

A diferencia de los prototipos 13 y 18, aquí **no interviene el binario `mmdr`**:
todo pasa por el crate `mermaid-rs-renderer` `=0.3.1` y su API por etapas, que es
lo que [El stack de rendering][8] decidió y nadie había compilado.

## Preparar

```sh
# 1. Los casos, desde el prototipo 12 (clases reales de dos paquetes Python)
/usr/bin/python3 -m venv venv            # 3.9, que es lo que espera extract.py
./venv/bin/pip install termaid==0.8.0
./venv/bin/python ../12-limite-de-termaid/extract.py venv/lib/python3.9/site-packages/termaid > graph.json
./venv/bin/python ../12-limite-de-termaid/gen.py     # -> cases/*.mmd
# renombrar los *_mem.mmd a cases/termaid-*, tirar los *_bare.mmd, y repetir
# apuntando extract.py al asyncio de la stdlib para los cinco cases/asyncio-*
cp ../13-mmdr-frente-a-termaid/arch.mmd cases/

# 2. El barrido
cd sweep && cargo build --release
```

## Ejecutar

```sh
python3 configs.py ejes > configs-ejes.json
./sweep/target/release/sweep --configs configs-ejes.json --out out cases/arch.mmd
python3 metricas.py out cases arch                     # -> JSON con una fila por combinación

# los finalistas, con imagen para mirarlos
./sweep/target/release/sweep --configs configs-img.json --out out-img --png cases/*.mmd
```

## Las piezas

- **`sweep/`** — el crate. Lee un `configs.json` con configuraciones nombradas
  —cada una con overrides **parciales** que se mezclan en profundidad sobre
  `LayoutConfig::default()`, aprovechando que la struct es `Serialize +
  Deserialize`— y corre las tres etapas por combinación:
  `parse_mermaid_strict` → (dirección impuesta sobre el IR) → `compute_layout` →
  `render_svg`. Vuelca el layout con `write_layout_dump`, que es el mismo JSON que
  emitía `--dumpLayout`, y opcionalmente SVG y PNG.
- **`configs.py`** — genera el barrido por ejes: una perilla movida cada vez desde
  el estado de partida, noventa configuraciones.
- **`metricas.py`** — puntúa los layouts contra el criterio de éxito del ticket.
  Además de las tres patologías de research 07 (perdidas, cruces, sueltas), mide
  **rodeos** (aristas que se salen del corredor de sus dos extremos), **desvío**
  (polilínea contra línea recta) y **vacío** (bandas horizontales sin un solo
  nodo), que son la queja de research 08 §6 puesta en números.
- **`renders/`** — las cinco imágenes citadas en el informe.

## Los renders

| fichero | qué es |
|---|---|
| `arch-hoy.png` | 804×1104: `Infrastructure` entre `API` y `Application`, y la arista `Controller → PlaceOrder` bajando por el borde derecho para volver cruzando el lienzo |
| `arch-lr.png` | 2137×409 con la dirección impuesta: cero rodeos, las siete aristas de izquierda a derecha |
| `arch-lr-aspect13.png` | la prueba de que `preferredAspectRatio` no pliega: pidiendo 1,3 sobre el anterior, el ancho sigue siendo 2137 y sólo crece el alto |
| `n20-hoy.png` / `n20-lr.png` | el `classDiagram` de 19 nodos, para ver que la dirección no es un apaño del caso protagonista |

`cases/`, `out*/`, `venv/` y los `configs-*.json` generados están en `.gitignore`:
se regeneran con lo de arriba.

[25]: https://github.com/javierponferradalopez/ai-render/issues/25
[8]: https://github.com/javierponferradalopez/ai-render/issues/8
