# El HTML en las etiquetas

Banco de [El HTML en las etiquetas: decidir con el dibujo delante][42], que es el
riesgo 3 del §11.2. Informe con los resultados:
[research 17](../../17-el-html-en-las-etiquetas.md).

El criterio del ticket manda el instrumento: los constructos hay que verlos
**dibujados, no descritos**. Así que aquí no se mide nada nuevo del parser — se
mira lo que sale en pantalla, caso a caso, y se decide con eso delante.

## Preparar

```sh
make build                                     # en la raíz: flipchart check
cd ../21-lo-que-mmdr-traga/bench && cargo build --release && cd -
cd pixeles && cargo build --release && cd -
```

## Ejecutar

```sh
../21-lo-que-mmdr-traga/bench/target/release/bench --permisivo --out svg casos/*.mmd
./pixeles/target/release/pixeles svg/*.svg     # los píxeles que sube el Visor
../../../../target/release/flipchart check casos/*.mmd   # el desenlace y su aviso

python3 mira.py casos/fc-01-br-cerrado.mmd casos/fc-04-negrita.mmd   # la ventana de verdad
```

## Las piezas

- **`casos/`** — 30 casos: 21 de `flowchart`, 8 de `classDiagram` y el diagrama
  espontáneo real que abre el ticket. El marcado se pone en los cuatro sitios
  donde las dos familias que se prometen llevan texto —etiqueta de nodo, de
  arista, de grupo y miembro de clase— y se barren las cinco formas de escribir un
  `br`, nueve etiquetas HTML, las entidades `&…;`, los escapes `#…;` de Mermaid y
  los caracteres crudos que sí salen bien.
- **`svg/`** — el SVG y el PNG de cada caso, generados. El SVG lo saca el `bench`
  del [prototipo 21](../21-lo-que-mmdr-traga) por el camino permisivo, que es la
  tubería de hoy tras el §3.1.
- **`pixeles/`** — el SVG a PNG por el mismo camino que la ventana: `usvg` con las
  fuentes del sistema y `resvg` sobre un `Pixmap`, que es lo que hace
  `src/raster.rs`. Existe porque `screencapture` necesita permiso de grabación de
  pantalla y sin él contesta `could not create image from display`.
- **`mira.py`** — la Pizarra de verdad: arranca el binario, le habla por stdio
  como haría el host, muestra cada caso y fotografía la pantalla en `capturas/`.
  Es el camino a *en la ventana* **cuando hay permiso de grabación de pantalla**.
  La ventana roba el foco una vez, como en producción.

## Qué contestó

`<br>` y `<br/>` —exactamente esas dos cadenas— se interpretan y salen como el
agente quería. Todo lo demás que no sea texto llega al dibujo literal: el resto de
etiquetas, las entidades, los escapes de Mermaid, y también `<br />` con un
espacio dentro. El desenlace decidido es **aviso** para lo segundo y
**convivencia sin aviso** para lo primero; el porqué está en el informe.

## Una trampa del banco, para no volver a caer

En `classDiagram`, mmdr **no entiende `class Pedido["Pedido<br/>agregado"]`**:
fabrica un nodo cuyo id es la línea entera y el `Límite honesto` lo rechaza como
apócrifo antes de dibujar nada. Es un defecto suyo ya apuntado en el banco del
prototipo 21 (`cd-13-etiqueta-clase`) y no tiene nada que ver con esta pregunta,
así que los casos `cd-*` de aquí ponen el marcado donde `classDiagram` sí lo
lleva: la etiqueta de la relación y el miembro.

[42]: https://github.com/javierponferradalopez/ai-render/issues/42
