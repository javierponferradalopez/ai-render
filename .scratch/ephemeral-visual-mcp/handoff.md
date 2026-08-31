# Handoff — Ephemeral Visual Context for AI Agents

> Material de partida, tal como lo entregó el usuario el 2026-08-31. Es la fuente
> del esfuerzo, no un documento vivo: donde el mapa contradiga a este handoff,
> manda el mapa.

## 1. Contexto

Queremos crear un MVP de una herramienta que permita a un agente de IA, especialmente agentes de programación como Claude Code, mostrar representaciones visuales durante una conversación.

El objetivo NO es crear otro Miro/Figma.

No queremos que el agente tenga que preocuparse de posiciones, estilos o rendering.

La idea es proporcionar al agente un **canvas visual temporal**.

## 2. Concepto fundamental

Separar completamente la **intención visual del agente** del **rendering visual**.

    AI Agent
        │ Visual Protocol
        ▼
    Visual MCP

El agente dice: "Quiero mostrar una arquitectura donde API depende de Application y Application depende de Domain."

No debe describir: coordenadas, tamaños, colores, SVG, HTML, CSS, shapes concretas, posiciones de píxeles.

El renderer se encarga de todo eso.

## 3. Principio arquitectónico más importante

NO acoplar el protocolo al renderer. En el futuro debe poder cambiarse el
adapter (Excalidraw, renderer propio) sin tocar el protocolo.

## 4. Objetivo del MVP

Un sistema local instalable fácilmente y utilizable desde un agente compatible con MCP. Idealmente `npx ephemeral-visual-mcp`, que levante:

1. servidor MCP local
2. visor web local
3. renderer inicial basado en tldraw
4. comunicación entre MCP y visor
5. herramientas visuales disponibles para el agente

El usuario NO debería tener que configurar manualmente una aplicación compleja.

## 5. UX objetivo

Usuario: "Refactoriza este servicio para separar la infraestructura del dominio."

Agente: "Voy a mostrarte visualmente cómo está actualmente y cómo quedaría."

Se abre/muestra una superficie visual con la arquitectura actual (Controller → Application → PrismaRepository). Después el agente llama a `visual.update(...)` para mostrar la propuesta (Controller → Application → Port → Infrastructure). Cuando deja de ser necesario, `visual.clear()`.

## 6. El canvas es EFÍMERO

Decisión fundamental. No queremos inicialmente: persistencia, boards, cuentas, colaboración, historial, exportación, comentarios, almacenamiento permanente, edición del usuario.

El canvas existe para ayudar a comprender una conversación. Es una pizarra temporal.

## 7. MCP

El MVP expone únicamente tres herramientas:

- `visual.show` — crear/reemplazar la visualización actual.
- `visual.update` — modificar la existente (`add` / `remove` / `update`). Debe ser incremental cuando tenga sentido; no obligar al agente a reenviar todo el diagrama para un cambio pequeño.
- `visual.clear` — eliminar la visualización actual.

## 8. No sobre-diseñar el protocolo inicialmente

Modelo semántico extremadamente pequeño. Primitivas iniciales: Node, Edge, Group, Text, Note.

Pero el agente no debería trabajar directamente con todas ellas si podemos ofrecer abstracciones semánticas, p. ej. `type: "architecture"` con `nodes` y `edges`.

El renderer decide: layout, spacing, colores, shapes, flechas, agrupaciones, tamaños, tipografía.

## 9. Tipos de diagrama contemplados

flow, sequence, class-diagram, state-machine, entity-relationship, dependency-graph, mindmap, timeline, wireframe.

El agente piensa en términos de conceptos, no de shapes.

## 10. Token efficiency

El agente produce `API -> Application -> Domain` y el renderer decide cómo representarlo.

## 11. Incremental updates

El agente debe poder modificar una visualización sin recrearla completamente. El protocolo debería transmitir únicamente el cambio, reduciendo tokens, latencia, operaciones y complejidad de razonamiento.

## 12. Renderer

Primera implementación con tldraw, encapsulado tras una interfaz propia:

    interface VisualRenderer {
      render(document: VisualDocument): void
      update(patch: VisualPatch): void
      clear(): void
    }

Nunca permitir que el MCP exponga conceptos propios de tldraw.

## 13. Frontend

TypeScript + React + tldraw. Aplicación muy pequeña. Responsabilidades: mostrar canvas, recibir estado visual, aplicar updates, gestionar viewport, mostrar/ocultar el canvas, opcionalmente informar de que está conectado.

No necesita autenticación, base de datos, backend complejo, usuarios ni persistencia.

## 14. Comunicación

MCP Server → WebSocket → React Viewer. El MCP Server necesita poder enviar cambios al visor inmediatamente. WebSocket puede sustituirse posteriormente.

## 15. Local-first

Todo en local. Sin servidor cloud, cuenta, API key, SaaS ni almacenamiento externo. Ventajas de privacidad para el código fuente.

## 16. Instalación ideal

`npx ephemeral-visual-mcp` debería: iniciar MCP server, iniciar viewer, abrir o registrar el endpoint local, y dejar el sistema preparado para el agente.

## 17. Claude Code

Investigar qué mecanismos proporciona hoy Claude Code para MCP, MCP Apps, interfaces visuales, URLs/local apps, extensiones y side panels.

UX deseada: Claude Code a la izquierda, canvas visual a la derecha. Pero no asumir que esta integración es posible exactamente así.

## 18. El usuario NO necesita editar

Para el MVP el usuario principalmente observa.

## 19. Posible estructura del repositorio

    packages/
      protocol/          # VisualDocument, VisualNode, VisualEdge, VisualGroup, VisualPatch
      core/              # validar documentos, aplicar patches, estado, IDs, lifecycle
      renderer-tldraw/   # adapter VisualDocument → tldraw shapes
      mcp-server/        # visual.show / visual.update / visual.clear
      viewer/            # React + tldraw

## 20. Principio de independencia

Código como `{ type: "geo", x: 100, y: 200 }` pertenece exclusivamente al adapter de tldraw. El MCP trabaja con `{ type: "node", id: "api", label: "API" }` o abstracciones semánticas mejores.

## 21. Arquitectura final esperada

    AI AGENT (Claude Code)
       │ MCP
       ▼
    MCP SERVER (visual.show / visual.update / visual.clear)
       ▼
    Visual Protocol (semantic model)
       ▼
    Renderer Adapter
       ▼
    tldraw (Renderer)

## 22. Decisiones tomadas

No cambiar durante el MVP salvo razón técnica fuerte:

1. El producto NO es un Miro.
2. El producto NO es un editor colaborativo.
3. El canvas es efímero.
4. El sistema está orientado a agentes.
5. MCP es el mecanismo inicial de integración.
6. El agente controla la visualización.
7. El usuario inicialmente solo visualiza.
8. El protocolo debe ser independiente del renderer.
9. tldraw se utiliza inicialmente como renderer.
10. No generar HTML como mecanismo de comunicación.
11. El agente debe expresar semántica, no píxeles.
12. Las actualizaciones deben poder ser incrementales.
13. El sistema debe funcionar localmente.
14. El coste en tokens es una preocupación de primer nivel.

## 23. Fuera del MVP

Cuentas, cloud, colaboración, persistencia, base de datos, historial, exportación, edición del usuario, marketplace, múltiples renderers, soporte para todos los tipos de diagramas, IA propia para generar layouts, sincronización entre máquinas.

## 24. Primer milestone

1. Ejecutar un comando local.
2. Registrar/conectar el MCP con Claude Code.
3. Claude Code dispone de `visual.show`.
4. El agente llama a `visual.show`.
5. Aparece un canvas local.
6. El canvas muestra un diagrama sencillo.
7. Claude Code llama a `visual.update`.
8. El diagrama cambia sin recrear todo.
9. Claude Code llama a `visual.clear`.
10. El canvas desaparece/se limpia.

Prueba: el usuario pide "Show me the architecture of this application", el agente llama a `visual.show` con nodes y edges, y el usuario ve automáticamente un diagrama razonable.

## 25. Criterio de éxito

El agente puede decir "Te lo voy a enseñar visualmente" y en segundos aparece una representación gráfica útil **sin generar HTML, SVG, React ni instrucciones de posicionamiento**.

El usuario debe sentir que el agente tiene una pizarra temporal. La visualización no debe sentirse como un documento que se está creando, sino como **una extensión visual de la conversación**.

## 26. Investigación previa obligatoria

1. Arquitectura actual de tldraw MCP Apps.
2. Licencia actual de tldraw y restricciones para uso/distribución.
3. Qué partes de tldraw son reutilizables.
4. Qué partes habría que aislar mediante el adapter.
5. Cómo funciona MCP Apps actualmente.
6. Qué soporta Claude Code actualmente respecto a MCP Apps/UI.
7. Si el viewer puede aparecer como ventana/panel independiente.
8. Si puede utilizarse localhost.
9. Cómo hacer lifecycle efímero.
10. Cuántos tokens consumen realmente las herramientas MCP de tldraw actuales.
11. Si merece la pena utilizar directamente tldraw MCP o construir nuestro propio MCP sobre tldraw.
12. Qué alternativas open source existen como renderer.

No asumir que tldraw es necesariamente la solución final.

## 27. Filosofía del proyecto

> **Give AI agents a visual channel for temporary communication.**

El agente ya sabe hablar. Ahora queremos darle una pizarra. Y esa pizarra debe ser: rápida, efímera, barata en tokens, local, semántica, independiente del renderer, invisible cuando no hace falta, y útil especialmente para programación y arquitectura.

El objetivo NO es almacenar lo que el agente dibuja. El objetivo es que el usuario **entienda mejor lo que el agente está diciendo**.
