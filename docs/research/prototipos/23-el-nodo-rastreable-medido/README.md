# El nodo rastreable, medido

Arnés del go/no-go de [La regla del nodo rastreable, medida contra el banco de
63][37]. Informe con los resultados:
[research 16](../../16-el-nodo-rastreable-medido.md).

No hay banco nuevo: el banco es el del [prototipo 21](../21-lo-que-mmdr-traga)
—63 casos con el parser de Mermaid 11.12 sobre jsdom como juez de validez—, y el
instrumento es `flipchart check`, la tubería de verdad sin ventana. Lo que se
añade aquí son las tres piezas que faltaban para poder contestar *sí* o *no*:

- **`esperado.tsv`** — qué es cada uno de los 63 casos (`correcto`, `fuga`,
  `invento`, `deformacion`, `rechazo`, según [research 14](../../14-lo-que-mmdr-traga.md)
  §1), si es Mermaid válido, y **qué pide de él el reparto de `DECISIONS` §4**:
  se rechaza el invento y se dibuja todo lo demás. Sin esta columna «rechazo» y
  «acierto» son la misma palabra.
- **`sondas/`** — nueve casos **fuera del banco**, escritos para las hipótesis
  que el banco no cubre: `subgraph` sin id declarado, ids con caracteres que el
  tokenizado parte distinto (`-`, `.`, `~`, acentos), participante implícito de
  `sequenceDiagram`, `[*]` de `stateDiagram-v2` aislado, y el solape de las dos
  reglas sobre una clase nunca declarada.
- **`careo.py`** — corre `check` sobre los tres corpus, compara con
  `esperado.tsv` y saca los números: falsos positivos entre los 42 correctos,
  inventos cazados, y **qué regla saltó en cada rechazo** (que es lo único que
  distingue a las dos, porque se informan juntas).

## Preparar

```sh
make build                                    # en la raíz: el binario que se mide
cd ../21-lo-que-mmdr-traga/contraste && npm install mermaid@11.12.0 jsdom   # el juez
```

## Ejecutar

```sh
python3 careo.py                              # el veredicto entero
python3 careo.py ruta/al/flipchart            # contra otro binario

cd ../21-lo-que-mmdr-traga/contraste && node contrasta.mjs ../../23-el-nodo-rastreable-medido/sondas
```

`careo.py` marca cada fila con `FP` cuando rechaza lo que había que dibujar y
`FN` cuando dibuja lo que había que rechazar. **Al subir la versión de `mmdr` se
vuelve a correr tal cual**: lo que cambia es el binario, no el banco.

[37]: https://github.com/javierponferradalopez/ai-render/issues/37
