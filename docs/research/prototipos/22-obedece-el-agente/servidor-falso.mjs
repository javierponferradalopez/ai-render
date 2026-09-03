#!/usr/bin/env node
// Servidor MCP de mentira: expone flipchart_show y flipchart_clear con el texto
// exacto que fijo #26, y en vez de dibujar apunta lo que le llega.
//
// FLIPCHART_VARIANT=A  el texto decidido (325 tokens, con la clausula de la asimetria)
// FLIPCHART_VARIANT=B  el mismo sin la clausula de asimetria (291) -- control del apartado 3
// FLIPCHART_VARIANT=C  A sin la norma de ofrecer-y-esperar -- control de la Apertura pedida
// FLIPCHART_LOG=<ruta> JSONL donde se apunta cada llamada

import { appendFileSync } from 'node:fs'

const VARIANT = process.env.FLIPCHART_VARIANT ?? 'A'
const LOG = process.env.FLIPCHART_LOG ?? '/dev/null'

const ASYMMETRY =
  'Any id used in a relationship must carry a label or a body when another id in the same ' +
  'diagram does; a bare id alongside a labelled one is rejected.\n\n'

// La norma de la Apertura pedida. La variante C es la unica que no la lleva:
// es A menos esta frase, para saber si es ella la que frena al agente.
const APERTURA =
  ' If the window is not open yet, offer it and wait for the user to accept; once it ' +
  'is open, just say what you are drawing.'

// La variante D es A mas esta frase. Las tres anteriores le dicen al agente
// *cuando* usar la pizarra, y ninguna le dice *en vez de que*: medido, lo que
// hace en su lugar es pintar el grafo en ASCII dentro de la respuesta.
const EN_VEZ_DE =
  ' If you would otherwise draw that structure as ASCII art or a diagram inside ' +
  'a code block in your reply, draw it here instead.'

const SHOW_DESC =
  'Show a diagram on the ephemeral flipchart window, as a named view. Takes Mermaid source.\n\n' +
  'Use it when the user needs to understand a structure, or a change to one, before deciding ' +
  'about it.' + (VARIANT === 'C' ? '' : APERTURA) +
  (VARIANT === 'D' ? EN_VEZ_DE : '') + '\n\n' +
  (VARIANT === 'B' ? '' : ASYMMETRY) +
  'Showing an existing view id replaces it and brings it to the front; several named views ' +
  'coexist. The flipchart dies with the session.'

const TOOLS = [
  {
    name: 'flipchart_show',
    description: SHOW_DESC,
    inputSchema: {
      type: 'object',
      properties: {
        view_id: {
          type: 'string',
          description:
            'Short human-readable name, shown to the user above the diagram - e.g. ' +
            '"Current dependencies", not "v1". Reusing a name replaces that view.',
        },
        diagram: { type: 'string', description: 'Mermaid source.' },
      },
      required: ['view_id', 'diagram'],
    },
  },
  {
    name: 'flipchart_clear',
    description:
      'Remove one view from the flipchart, or all of them. Does not close the window.',
    inputSchema: {
      type: 'object',
      properties: {
        view_id: {
          type: 'string',
          description: 'View to remove. Omit to clear the whole flipchart.',
        },
      },
      required: [],
    },
  },
]

// El estado que #11 le devuelve al agente: acuse + recuento + vistas vivas.
const views = []

function apunta(entry) {
  appendFileSync(LOG, JSON.stringify({ ...entry, variant: VARIANT, at: Date.now() }) + '\n')
}

function llama(name, args) {
  if (name === 'flipchart_show') {
    const { view_id, diagram } = args ?? {}
    apunta({ event: 'show', view_id, diagram })
    const i = views.indexOf(view_id)
    if (i === -1) views.push(view_id)
    // Nunca rechaza: el rechazo ensenaria dentro de la conversacion y la variante B
    // no lo tendria, con lo que el control del apartado 3 dejaria de serlo.
    return `Shown. ${views.length} view(s): ${views.join(', ')}`
  }
  if (name === 'flipchart_clear') {
    const { view_id } = args ?? {}
    apunta({ event: 'clear', view_id })
    if (view_id === undefined) views.length = 0
    else {
      const i = views.indexOf(view_id)
      if (i !== -1) views.splice(i, 1)
    }
    return views.length ? `Cleared. ${views.length} view(s): ${views.join(', ')}` : 'Cleared. Flipchart empty.'
  }
  throw new Error(`unknown tool ${name}`)
}

let buf = ''
process.stdin.on('data', (chunk) => {
  buf += chunk
  let nl
  while ((nl = buf.indexOf('\n')) !== -1) {
    const line = buf.slice(0, nl).trim()
    buf = buf.slice(nl + 1)
    if (line) atiende(JSON.parse(line))
  }
})

function responde(id, result) {
  process.stdout.write(JSON.stringify({ jsonrpc: '2.0', id, result }) + '\n')
}

function atiende(msg) {
  const { id, method, params } = msg
  if (method === 'initialize') {
    return responde(id, {
      protocolVersion: '2024-11-05',
      capabilities: { tools: {} },
      serverInfo: { name: 'flipchart', version: '0.0.0-fake' },
    })
  }
  if (method === 'tools/list') return responde(id, { tools: TOOLS })
  if (method === 'tools/call') {
    const text = llama(params.name, params.arguments)
    return responde(id, { content: [{ type: 'text', text }] })
  }
  if (id !== undefined) responde(id, {})
}
