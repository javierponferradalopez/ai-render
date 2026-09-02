# Prototipo 18 — Mermaid frente al protocolo propio

Material ejecutado para
[¿Qué añade esto sobre Mermaid?](https://github.com/javierponferradalopez/ai-render/issues/15).
Dos sujetos: **`claude-mermaid` 1.6.5**, el competidor que el ticket mandaba
probar, y **mmdr 0.3.1**, la candidata a renderer.

Todo lo de aquí corre en local y no publica nada. `corre-claude-mermaid.sh` sí
**abre pestañas de navegador** y deja un servidor HTTP en `localhost:3737` hasta
que mates el proceso — eso es parte de lo que se mide.

## Qué corre cada cosa

| Script | Qué mide |
|---|---|
| `corre-claude-mermaid.sh` | Instala el competidor, captura su `tools/list`, renderiza dos vistas con nombre, dispara el live reload y enseña la galería que sobrevive |
| `peaje.py` | Peaje fijo en tokens: competidor real contra un flipchart que trague Mermaid, y el break-even contra el protocolo propio |
| `sondea-mmdr.sh` | Qué hace mmdr con lo que Mermaid permite y la pizarra no debería aceptar: `classDef`/`style`/`linkStyle`, basura sintáctica, y un id con typo |
| `probe.mjs` / `drive.mjs` | Cliente MCP mínimo por stdio — handshake, `tools/list`, `tools/call` con cronómetro |

```sh
pip install tiktoken          # solo para peaje.py
./sondea-mmdr.sh              # no necesita npm
./corre-claude-mermaid.sh     # 489 MB de node_modules, abre navegador
```

`sondea-mmdr.sh` baja el binario del release v0.3.1 y **verifica el sha256**
contra `562d0250…f3223`, el mismo que dejó escrito
[research 08](../../08-mmdr-un-mermaid-que-emite-geometria.md). El binario y
`node_modules/` están en el `.gitignore`: los traen los scripts.

## Resultados, para no tener que correrlo

**Peaje fijo** (`cl100k_base`, el tokenizador de research 04 y del issue #10):

| | tokens |
|---|---:|
| `claude-mermaid` 1.6.5, 2 tools | **611** |
| Flipchart tragando Mermaid, 2 tools | **204** |
| Protocolo propio, 3 tools *(issue #10)* | 738 |
| Protocolo propio con guía de uso *(issue #10)* | 1.047 |

Break-even del protocolo propio: **6,0 retoques** sobre la misma vista, u **8,8**
si el protocolo lleva guía de uso.

**Velocidad del competidor**, con Chromium detrás: 1.827 ms en frío, ~485 ms en
caliente. mmdr hace el mismo `arch.mmd` en 62 ms
([research 08](../../08-mmdr-un-mermaid-que-emite-geometria.md) §4).

**Lo que mmdr acepta y no debería** — los tres con `exit 0`:

1. `classDef` / `style` / `linkStyle` **se aplican**: `fill="#f00"`, `fill="#0f0"`
   y `stroke="#00f"` salen en el SVG (`renders/pixel-mmdr.svg`).
2. Basura sintáctica **se dibuja**: `@@@` y `esto_no_es_mermaid_en_absoluto`
   aterrizan como cajas (`renders/basura-mmdr.svg`).
3. `Ordr --> Money` con `Order` declarada dibuja **tres** clases, y la relación
   sale de la fantasma (`renders/typo-mmdr.svg`).

El caso 3 es el que decidió el ticket. Mermaid.js hace lo mismo y lo dibuja
mejor: `renders/typo-mermaidjs.png` — la caja fantasma con sus secciones vacías
es indistinguible de una clase legítima de la que se sabe poco.

**Calidad de grupos, el mismo `arch.mmd` de
[prototipo 13](../13-mmdr-frente-a-termaid/arch.mmd):**
`renders/arch-mermaidjs.png` contra
[`13-mmdr-frente-a-termaid/renders/arch-subgraphs.png`](../13-mmdr-frente-a-termaid/renders/arch-subgraphs.png).
Mermaid.js coloca las cuatro capas sin solaparse; mmdr mete `Infrastructure`
entre `API` y `Application` y saca una arista en una U por un hueco vacío.

## Ficheros

- `fixtures/pixel.mmd`, `basura.mmd`, `typo.mmd` — los tres sondeos
- `fixtures/vista-actual.mmd`, `vista-propuesto.mmd` — dos vistas con nombre del
  propio dominio de la pizarra, para probar la convivencia
- `renders/` — las salidas de los dos motores sobre los mismos fuentes
- `claude-mermaid-tools.json` — su `tools/list` tal cual, capturado
