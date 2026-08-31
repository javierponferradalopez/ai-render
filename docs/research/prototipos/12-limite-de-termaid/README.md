# Medición del límite de tamaño de termaid

Scripts del prototipo de [¿Se lee bien un refactor real?][12]. Informe con los
resultados: [research 07](../../07-limite-de-tamano-de-termaid.md).

## Preparar

```sh
python3 -m venv venv
./venv/bin/pip install termaid==0.8.0
```

Los scripts esperan `./venv/bin/python` y se ejecutan desde este directorio.

## Ejecutar, en orden

```sh
# 1. Grafo de clases desde código real (cualquier paquete Python)
./venv/bin/python extract.py venv/lib/python3.9/site-packages/termaid > graph.json

# 2. Subgrafos conexos por tamaño -> cases/*.mmd
./venv/bin/python gen.py

# 3. Render + las cuatro patologías, con --gap 1 --padding-y 0
./venv/bin/python analyze.py

# 4. Lo mismo, pero la mejor de 32 configuraciones por caso  <- la tabla del informe
./venv/bin/python sweep_all.py
```

Para la segunda fuente, repetir desde el paso 1 apuntando `extract.py` a la
stdlib: `$(./venv/bin/python -c "import asyncio,os;print(os.path.dirname(asyncio.__file__))")`.

## Los demás

- `external_layout.py` / `ext_sweep.py` — el techo de la salida 2: inyecta por
  `grid_positions` el orden de capas que minimiza cruces (mejor que Graphviz) y
  mide si el ruteo mejora.
- `width.py` — descarta que la causa sea la anchura de las etiquetas.
- `fanout.py` — descarta que la causa sea el fan-out de una jerarquía.
- `sweep_gap.py` — barrido de `gap`/`padding` sobre la fuente 1.

## Qué mide `analyze.py`

Reconstruye las cajas del render por sus esquinas y las compara con el grafo
fuente, que es la verdad. Cuatro patologías:

1. **Aristas perdidas** — cada arista produce exactamente un marcador
   (`►◄▲▼△◆◇`), calibrado en un caso de control. Menos marcadores que aristas
   significa relaciones que no se dibujan.
2. **Paredes corrompidas** — cualquier `┼`, `┬`, `┴` o punta de flecha incrustada
   en el borde de una caja es una arista atravesándola.
3. **Fragmentos huérfanos** — puntas de flecha sin línea que las alimente.
4. **Cajas sueltas** — clases con relaciones en el fuente y ningún marcador
   tocando su perímetro: su relación es irrastreable.

[12]: https://github.com/javierponferradalopez/ai-render/issues/12
