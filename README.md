# flipchart — una pizarra efímera para agentes

Un canal visual temporal para que un agente de IA se explique: cuando te cuenta una
estructura o un cambio de estructura, lo dibuja en una ventana nativa en vez de en ASCII o
en prosa. Guarda varias vistas y enseña una cada vez; muere con la sesión y no guarda nada.

Se instala como plugin de Claude Code, y es el único camino de instalación.

## Instalación

Dos líneas dentro de Claude Code:

```
/plugin marketplace add https://raw.githubusercontent.com/javierponferradalopez/ai-render/main/marketplace.json
/plugin install flipchart@flipchart
```

Y un tercer paso que **no es opcional**: pegar esta línea en tu `CLAUDE.md`.

```
Cuando me expliques una estructura o un cambio de estructura, dibújalo en la
pizarra con mcp__flipchart__show en vez de en ASCII o en prosa.
```

Sin ella la pizarra queda instalada y no se usa jamás. Está medido: por su cuenta el
agente no la ofrece, y no la usa hasta que se le pide o se le manda.

## Por qué sólo por `/plugin`

Porque es el camino que está medido, y el único que llega a una máquina utilizable:

- Claude Code descarga el zip, **comprueba su `sha256`** y **rechaza la instalación** si no
  casa, con el error delante.
- El binario que extrae llega **sin `com.apple.quarantine`**, así que su ejecución no pasa
  por Gatekeeper. Un fichero traído por el navegador o por Mail **sí** lleva ese atributo, y
  es el caso que Gatekeeper mata.
- No se ejecuta `git` ni una vez, y lo que queda en disco se poda solo.

## Requisitos

- **macOS**, Intel o Apple Silicon. El umbral de versión es provisional: lo fija lo que
  exijan las bibliotecas de ventana, y sale del primer build.
- Una versión de Claude Code con soporte de plugins.
- **Nada más**: no hay Node, ni Python, ni navegador, ni toolchain de Rust.

## Actualizar y desinstalar

```
/plugin update flipchart
/plugin uninstall flipchart
```

`uninstall` se lleva también los datos del plugin, así que no hay ningún `rm -rf` que
teclear. Para desactivarla sin desinstalarla, `/plugin`.

Un aviso de disco, medido: entre una actualización y la poda automática la caché guarda
**las dos versiones** del binario, no una.

## Desarrollo

Las decisiones del producto están en [`DECISIONS.md`](./DECISIONS.md), el idioma del
dominio en [`CONTEXT.md`](./CONTEXT.md), y lo medido en [`docs/research/`](./docs/research/).
Cómo se construye y cuál es la puerta, en [`CLAUDE.md`](./CLAUDE.md).
