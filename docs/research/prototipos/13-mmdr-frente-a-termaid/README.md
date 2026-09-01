# mmdr frente a termaid, sobre los mismos casos

Scripts del prototipo que mide `mermaid-rs-renderer` (`mmdr`) 0.3.1 con los
diagramas que tumbaron a termaid. Informe con los resultados:
[research 08](../../08-mmdr-un-mermaid-que-emite-geometria.md).

Los casos **no se regeneran aquí**: salen de los scripts del
[prototipo 12](../12-limite-de-termaid/), que extraen clases reales con `ast` de
dos paquetes Python y recortan subgrafos conexos por tamaño. Este prototipo solo
cambia el renderer al que se le dan.

## Preparar

```sh
# 1. Los casos, desde el prototipo 12 (ver su README)
python3 -m venv venv
./venv/bin/pip install termaid==0.8.0
./venv/bin/python ../12-limite-de-termaid/extract.py venv/lib/python3.9/site-packages/termaid > graph.json
./venv/bin/python ../12-limite-de-termaid/gen.py          # -> cases/*.mmd

# 2. El binario, del release (verificar el sha256 contra el publicado)
curl -sLO https://github.com/1jehuang/mermaid-rs-renderer/releases/download/v0.3.1/mmdr-aarch64-apple-darwin.tar.gz
shasum -a 256 mmdr-aarch64-apple-darwin.tar.gz
tar xzf mmdr-aarch64-apple-darwin.tar.gz
```

## Ejecutar

```sh
MMDR=./mmdr python3 analyze_mmdr.py cases/n0{3,4,5,6,7,8}_mem.mmd cases/n1{0,2,4,7}_mem.mmd cases/n20_mem.mmd
python3 bench.py ./mmdr
python3 incremental.py ./mmdr
```

## Qué mide `analyze_mmdr.py`

Las cuatro patologías del [research 07](../../07-limite-de-tamano-de-termaid.md),
trasladadas de celdas de carácter a geometría. Dos de ellas —paredes corrompidas
y fragmentos huérfanos— **no pueden darse en SVG**: son artefactos de dibujar
sobre una rejilla de caracteres. Se sustituyen por su equivalente honesto en
píxeles, que es una arista atravesando una caja ajena.

La geometría no se saca del SVG: la emite el propio `mmdr` con `--dumpLayout`, un
JSON con cada nodo (`x`, `y`, `width`, `height`, `label_lines`) y cada arista
(polilínea de puntos).

1. **Aristas perdidas** — relaciones del fuente `.mmd` que no aparecen en
   `edges[]` del layout.
2. **Cruces** — segmento de arista que entra en el rectángulo de un nodo que no
   es su origen ni su destino, con 1 px de margen. Es deliberadamente estricto:
   marca también los roces tangenciales de esquina, así que sus positivos hay que
   mirarlos antes de creerlos (ver el apartado 3 del informe).
3. **Cajas sueltas** — clases con relaciones en el fuente y ninguna arista
   dibujada que las toque.

## Los demás

- `bench.py` — tiempos de proceso completo, mediana de 20 ejecuciones.
- `incremental.py` — determinismo (mismo fuente, mismo SVG byte a byte) y cuántos
  nodos preexistentes se mueven al añadir uno. Importa para una pizarra que se
  actualiza en vivo.
- `arch.mmd` — el caso donde mmdr flojea: cuatro capas como `subgraph` con
  aristas cruzándolas.
- `renders/` — los cuatro renders citados en el informe, en PNG.
