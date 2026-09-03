# Las dos condiciones de entorno

No son variantes del texto de las herramientas: son ficheros que se copian al **sujeto**
antes de correr, y el texto de la herramienta se deja en **A**.

| Fichero | Se copia como | Corridas | Qué mide |
|---|---|---|---|
| `CLAUDE.md.prohibe-ascii` | `CLAUDE.md` del sujeto | `r7`, `r8`, `r9` | Si el agente descubre la pizarra cuando le quitan el ASCII. **No menciona flipchart**, a propósito. |
| `CLAUDE.md.manda-flipchart` | `CLAUDE.md` del sujeto | `r5`, `r6` | Si una línea de instrucciones de proyecto dispara lo que 325 palabras de descripción no disparan. |

Como canal de producto, las instrucciones de proyecto están **fuera de alcance**: la caja
lleva `.mcp.json` y el binario y nada más. Lo que sí queda dentro tras #28 es *recomendar*
la línea desde la documentación de instalación.
