#!/usr/bin/env node
// Lee registros + transcripciones y saca las tres cuentas del ticket #28.
//
// El detector de Nodo fantasma es heuristico y deliberadamente conservador:
// solo marca lo que la regla de CONTEXT.md declara fantasma -- un id que
// aparece unicamente en relaciones, sin etiqueta ni cuerpo, habiendo en el
// mismo diagrama al menos uno que si los tiene.

import { readdirSync, readFileSync } from 'node:fs'
import { dirname, resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const AQUI = dirname(fileURLToPath(import.meta.url))

// --- Nodo fantasma -----------------------------------------------------------

const CON_CUERPO = [
  /\b([A-Za-z_][\w.-]*)\s*\[\[[^\]]*\]\]/g,   // A[[sub]]
  /\b([A-Za-z_][\w.-]*)\s*\(\([^)]*\)\)/g,    // A((circ))
  /\b([A-Za-z_][\w.-]*)\s*\{\{[^}]*\}\}/g,    // A{{hex}}
  /\b([A-Za-z_][\w.-]*)\s*\[[^\]]+\]/g,       // A[label]
  /\b([A-Za-z_][\w.-]*)\s*\([^)]+\)/g,        // A(label)
  /\b([A-Za-z_][\w.-]*)\s*\{[^}]+\}/g,        // A{label}
  /\b([A-Za-z_][\w.-]*)\s*>[^\]]+\]/g,        // A>label]
  /^\s*class\s+([A-Za-z_][\w.-]*)/gm,         // classDiagram: class X
  /^\s*subgraph\s+([A-Za-z_][\w.-]*)/gm,      // subgraph G [..]
  /^\s*([A-Za-z_][\w.-]*)\s*:\s*\S/gm,        // classDiagram: X : miembro
]

// Relaciones de flowchart y de classDiagram.
const RELACION =
  /^\s*([A-Za-z_][\w.-]*)(?:\s*(?:\[[^\]]*\]|\([^)]*\)|\{[^}]*\}))?\s*(?:<\|--|\*--|o--|<--|--\|>|--\*|--o|-->|---|-\.->|-\.-|==>|===|--)(?:\|[^|]*\|)?\s*([A-Za-z_][\w.-]*)/gm

const PALABRAS_CLAVE = new Set([
  'flowchart', 'graph', 'classDiagram', 'subgraph', 'end', 'class', 'direction',
  'LR', 'RL', 'TB', 'BT', 'TD', 'style', 'classDef', 'linkStyle', 'click', 'note',
])

function fantasmas(fuente) {
  const conCuerpo = new Set()
  for (const re of CON_CUERPO) {
    for (const m of fuente.matchAll(re)) {
      if (!PALABRAS_CLAVE.has(m[1])) conCuerpo.add(m[1])
    }
  }
  const enRelacion = new Set()
  for (const m of fuente.matchAll(RELACION)) {
    for (const id of [m[1], m[2]]) if (!PALABRAS_CLAVE.has(id)) enRelacion.add(id)
  }
  // Sin ninguno etiquetado no hay asimetria, asi que no hay fantasma.
  if (conCuerpo.size === 0) return []
  return [...enRelacion].filter((id) => !conCuerpo.has(id))
}

// --- lectura -----------------------------------------------------------------

function lee(nombre) {
  const reg = readFileSync(resolve(AQUI, 'registros', nombre), 'utf8')
    .split('\n').filter(Boolean).map(JSON.parse)
  const t = resolve(AQUI, 'transcripciones', nombre.replace('.jsonl', '.json'))
  const trans = JSON.parse(readFileSync(t, 'utf8'))
  // Reparte las llamadas por turno.
  const turnos = []
  for (const e of reg) {
    if (e.event === 'turno') turnos.push({ n: e.n, user: e.user, llamadas: [] })
    else turnos[turnos.length - 1]?.llamadas.push(e)
  }
  for (const t of turnos) {
    const dicho = trans.turnos.find((x) => x.n === t.n)
    t.assistant = dicho?.assistant ?? ''
    t.error = dicho?.error
  }
  return { ...trans, turnos }
}

const corridas = readdirSync(resolve(AQUI, 'registros'))
  .filter((f) => f.endsWith('.jsonl')).sort().map(lee)

// --- cuentas -----------------------------------------------------------------

let totalShow = 0
const porVariante = { A: { show: 0, conFantasma: 0, fantasmas: [] }, B: { show: 0, conFantasma: 0, fantasmas: [] } }

for (const c of corridas) {
  console.log(`\n${'='.repeat(72)}\n${c.escenario}  variante ${c.variante}  rep ${c.rep}`)
  for (const t of c.turnos) {
    const shows = t.llamadas.filter((l) => l.event === 'show')
    console.log(`\n  [turno ${t.n}] usuario: ${t.user.slice(0, 90)}${t.user.length > 90 ? '...' : ''}`)
    console.log(`      llamadas: ${t.llamadas.length ? t.llamadas.map((l) => `${l.event}(${l.view_id ?? ''})`).join(' ') : '-- ninguna --'}`)
    if (t.error) console.log(`      ERROR: ${t.error.slice(0, 120)}`)
    for (const s of shows) {
      totalShow++
      const v = porVariante[c.variante]
      v.show++
      const f = fantasmas(s.diagram ?? '')
      if (f.length) {
        v.conFantasma++
        v.fantasmas.push({ corrida: `${c.escenario}/${c.variante}/r${c.rep}`, view: s.view_id, ids: f })
        console.log(`      FANTASMAS en "${s.view_id}": ${f.join(', ')}`)
      }
    }
    const dicho = (t.assistant ?? '').replace(/\s+/g, ' ')
    console.log(`      dice: ${dicho.slice(0, 260)}${dicho.length > 260 ? '...' : ''}`)
  }
}

console.log(`\n${'='.repeat(72)}\nAPARTADO 3 -- la clausula de la asimetria\n`)
for (const v of ['A', 'B']) {
  const d = porVariante[v]
  const etiqueta = v === 'A' ? 'A (con clausula, 325)' : 'B (sin clausula, 291)'
  console.log(`  ${etiqueta}: ${d.conFantasma}/${d.show} show con Nodo fantasma`)
  for (const f of d.fantasmas) console.log(`      ${f.corrida} "${f.view}": ${f.ids.join(', ')}`)
}
console.log(`\n  total de show: ${totalShow}`)
