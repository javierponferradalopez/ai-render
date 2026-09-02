# Lo que mmdr traga

Banco del prototipo de [¿Qué traga mmdr sin dibujar?][27]. Informe con los
resultados: [research 14](../../14-lo-que-mmdr-traga.md).

Sesenta y tres casos de Mermaid contra la tubería que flipchart va a ejecutar,
buscando el tercer tipo de fuga: la que sale con `exit 0` y sin dibujo. Como el
[prototipo 20](../20-las-perillas-de-mmdr), todo pasa por el crate
`mermaid-rs-renderer` `=0.3.1` y su API por etapas; el binario `mmdr` no
interviene.

A diferencia del 20, aquí hay un segundo motor: **el parser de Mermaid 11.12.0**,
que no dibuja nada y sólo contesta a una pregunta —*¿es esto Mermaid válido?*—.
Sin él, un caso que mmdr destroza no se distingue de un caso mal escrito.

## Preparar

```sh
cd bench && cargo build --release && cd ..
cd contraste && npm install mermaid@11.12.0 jsdom && cd ..   # sin Chromium
```

## Ejecutar

```sh
./bench/target/release/bench --out out      cases/*.mmd
./bench/target/release/bench --out out-fam  familias/*.mmd
./bench/target/release/bench --out out-probe  probe/*.mmd
./bench/target/release/bench --out out-permisivo --permisivo probe/*.mmd  # sin validador

python3 censo.py out cases          # el careo palabra a palabra
python3 censo.py out-fam familias

cd contraste && node contrasta.mjs ../cases && node contrasta.mjs ../familias
```

## Las piezas

- **`bench/`** — el crate. Corre la tubería de flipchart tal como quedó decidida:
  `parse_mermaid_strict` → vaciado de los siete campos de estilo ([#11]) →
  `Direction::LeftRight` impuesta en `flowchart` y `classDiagram` ([#25]) →
  `compute_layout` → `render_svg`. Por caso deja el SVG y una ficha del `Graph`
  en `out/censo.json`: qué nodos y aristas quedaron, qué subgrafos, y **qué
  canales laterales vinieron llenos**, que es donde se ve dónde aterrizó cada
  constructo. Envuelve las etapas 2 y 3 en `catch_unwind` para que un pánico no
  se lleve el banco entero. Con `--permisivo` cambia `parse_mermaid_strict` por
  `parse_mermaid` —el camino que flipchart **no** usa—, que es como se comprueba
  qué le quita el validador a mmdr.
- **`censo.py`** — el careo barato: **las palabras del fuente contra las palabras
  del SVG**. Lo que el autor escribió y no sale en pantalla es sospechoso de
  fuga. Necesita una lista de palabras que son sintaxis —y esa lista es el
  problema del método, ver el informe—.
- **`contraste/contrasta.mjs`** — `mermaid.parse()` sobre jsdom, sin renderizar.
  Contesta si el caso es Mermaid válido. 153 MB de `node_modules` y ni un
  Chromium.
- **`cases/`** — cuarenta constructos de `flowchart`, `classDiagram`,
  `stateDiagram-v2`, `sequenceDiagram` y `erDiagram`. Las etiquetas van en
  palabras inventadas de una sola pieza (`Notasueltadeldiagrama`) para que el
  careo no tenga que adivinar.
- **`familias/`** — un ejemplo mínimo de cada uno de los **23 tipos de diagrama**
  que mmdr declara.
- **`probe/`** — las cuatro variantes de `<<interface>>` que descartan la sangría
  como causa del rechazo, y que por el camino permisivo salen dibujadas.

[27]: https://github.com/javierponferradalopez/ai-render/issues/27
[#11]: https://github.com/javierponferradalopez/ai-render/issues/11
[#25]: https://github.com/javierponferradalopez/ai-render/issues/25
