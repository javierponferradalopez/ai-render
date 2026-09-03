#!/usr/bin/env node
// El instrumento del apartado 1.
//
// El registro del servidor falso solo ve las llamadas que pasan el permiso, y en
// Claude Code 2.1.228 --allowedTools NO concede herramientas MCP en modo -p: el
// agente ve la pizarra, la intenta, y recibe "requested permissions ... but you
// haven't granted it yet". Asi que contar shows registrados confunde dos cosas
// muy distintas -- *no lo intento* y *lo intento y no pudo*.
//
// Este script cuenta lo otro: los tool_use del historial de Claude Code, que se
// graban cuando el agente decide llamar, antes de que el permiso los frene. Es lo
// que hay que contar para saber si el agente QUIERE usar la pizarra. Y de paso
// saca los diagramas, que viajan dentro del tool_use aunque el dibujo no ocurra.
//
// Uso: node intentos.mjs <ruta del sujeto> [--diagramas]

import { readdirSync, readFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { resolve } from 'node:path'

const sujeto = process.argv[2]
if (!sujeto) {
  console.error('Uso: node intentos.mjs <ruta del sujeto> [--diagramas]')
  process.exit(1)
}
const conDiagramas = process.argv.includes('--diagramas')

// Claude Code guarda un historial por proyecto, con la ruta aplanada.
const slug = resolve(sujeto).replace(/\//g, '-')
const dir = resolve(homedir(), '.claude/projects', slug)

const sesiones = readdirSync(dir)
  .filter((f) => f.endsWith('.jsonl'))
  .map((f) => resolve(dir, f))
  .sort()

let totalIntentos = 0, totalTurnos = 0
const diagramas = []

for (const f of sesiones) {
  let intentos = 0, turnos = 0, primero = null, denegados = 0
  for (const linea of readFileSync(f, 'utf8').split('\n')) {
    if (!linea.trim()) continue
    let d
    try { d = JSON.parse(linea) } catch { continue }
    const c = d.message?.content
    if (d.type === 'user' && typeof c === 'string' && c.trim()) {
      turnos++
      primero ??= c
    }
    if (!Array.isArray(c)) continue
    for (const b of c) {
      if (b.type === 'tool_use' && String(b.name).includes('flipchart_show')) {
        intentos++
        diagramas.push({ view_id: b.input?.view_id, diagram: b.input?.diagram })
      }
      if (b.type === 'tool_result' && JSON.stringify(b.content ?? '').includes('permission')) {
        denegados++
      }
    }
  }
  if (!primero) continue
  totalIntentos += intentos
  totalTurnos += turnos
  const marca = denegados ? ` (${denegados} frenados por el permiso)` : ''
  console.log(`  intentos=${String(intentos).padEnd(3)} turnos=${turnos}${marca} | ${primero.slice(0, 60)}`)
}

console.log(`\n  TOTAL: ${totalIntentos} intentos de invocar la pizarra en ${totalTurnos} turnos de usuario`)

if (conDiagramas) {
  const patrones = {
    'HTML en etiquetas (<br>, <b>)': /<br\s*\/?>|<\/?b>|<\/?i>|<\/?span/,
    'color literal (rgba/hex)': /rgba?\(|#[0-9a-fA-F]{6}\b/,
    'style/classDef/linkStyle/click': /^\s*(style|classDef|linkStyle|click)\b/m,
    'familia no medida': /^\s*(sequenceDiagram|stateDiagram|erDiagram|journey|gantt)/m,
    'direction dentro de subgraph': /^\s*direction\s+(LR|RL|TB|BT|TD)/m,
  }
  console.log(`\n  === los ${diagramas.length} diagramas que quiso dibujar ===`)
  for (const [nombre, re] of Object.entries(patrones)) {
    const n = diagramas.filter((d) => re.test(d.diagram ?? '')).length
    console.log(`    ${nombre.padEnd(34)} ${n}/${diagramas.length}`)
  }
  console.log()
  for (const d of diagramas) console.log(`    [${d.view_id}]`)
}
