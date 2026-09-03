#!/usr/bin/env node
// Arnes: lanza conversaciones reales de Claude Code contra el servidor falso y
// guarda, por corrida, el registro de llamadas y la transcripcion.
//
// Uso: node corre.mjs <variante A|B> <repeticion> [id-escenario ...]

import { execFileSync } from 'node:child_process'
import { writeFileSync, readFileSync, appendFileSync, existsSync, unlinkSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const AQUI = dirname(fileURLToPath(import.meta.url))
// Sin valor por defecto a proposito: el sujeto se pasa siempre, y tiene que ser
// una copia desechable. Una corrida lee el repo entero, y si algo se le permite,
// lo escribe. Nunca apuntar a un repo de trabajo.
const REPO = process.env.REPO_SUJETO
const [variante = 'A', rep = '1', ...soloEstos] = process.argv.slice(2)

if (!REPO) {
  console.error('Falta REPO_SUJETO: la ruta de una copia desechable del repo sujeto.')
  process.exit(1)
}

const escenarios = JSON.parse(readFileSync(resolve(AQUI, 'escenarios.json'), 'utf8'))
  .filter((e) => soloEstos.length === 0 || soloEstos.includes(e.id))

// Solo lectura, y ademas prohibido escribir explicitamente. La primera corrida
// llevaba --permission-mode acceptEdits: el agente leyo un "si, adelante" del
// guion como "haz el refactor" y movio, reescribio y borro codigo del repo
// sujeto. acceptEdits auto-aprueba las ediciones al margen de esta lista, asi
// que la lista blanca no basta -- hacen falta las tres cosas a la vez.
const HERRAMIENTAS = [
  'Read', 'Grep', 'Glob', 'Bash(ls:*)', 'Bash(find:*)', 'Bash(cat:*)', 'Bash(wc:*)',
  'mcp__flipchart__flipchart_show', 'mcp__flipchart__flipchart_clear',
]

const PROHIBIDAS = ['Edit', 'Write', 'NotebookEdit', 'Task', 'Skill']

for (const esc of escenarios) {
  const nombre = `${esc.id}-${variante}-r${rep}`
  const log = resolve(AQUI, 'registros', `${nombre}.jsonl`)
  const trans = resolve(AQUI, 'transcripciones', `${nombre}.json`)
  if (existsSync(log)) unlinkSync(log)

  const mcp = resolve(AQUI, `mcp-${nombre}.json`)
  writeFileSync(mcp, JSON.stringify({
    mcpServers: {
      flipchart: {
        command: 'node',
        args: [resolve(AQUI, 'servidor-falso.mjs')],
        env: { FLIPCHART_VARIANT: variante, FLIPCHART_LOG: log },
      },
    },
  }))

  const turnos = []
  let sid = null
  console.error(`\n=== ${nombre} ===`)

  for (const [i, texto] of esc.turnos.entries()) {
    appendFileSync(log, JSON.stringify({ event: 'turno', n: i + 1, user: texto }) + '\n')
    const args = [
      '-p', texto,
      '--output-format', 'json',
      '--mcp-config', mcp,
      '--strict-mcp-config',
      '--allowedTools', ...HERRAMIENTAS,
      '--disallowedTools', ...PROHIBIDAS,
      '--permission-mode', 'default',
    ]
    if (sid) args.push('--resume', sid)
    let out
    try {
      out = execFileSync('claude', args, {
        cwd: REPO, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 15 * 60 * 1000,
      })
    } catch (e) {
      console.error(`  turno ${i + 1}: FALLO -- ${e.message.slice(0, 200)}`)
      turnos.push({ n: i + 1, user: texto, error: String(e.message).slice(0, 2000) })
      break
    }
    const r = JSON.parse(out)
    sid = r.session_id ?? sid
    turnos.push({
      n: i + 1, user: texto, assistant: r.result,
      modelo: Object.keys(r.modelUsage ?? {}), coste_usd: r.total_cost_usd,
    })
    console.error(`  turno ${i + 1}: ${String(r.result).replace(/\s+/g, ' ').slice(0, 140)}`)
  }

  // Solo el nombre del sujeto: la ruta absoluta lleva usuario y sesion dentro,
  // y esto se commitea.
  const sujeto = REPO.replace(/\/+$/, '').split('/').pop()
  writeFileSync(trans, JSON.stringify({ escenario: esc.id, variante, rep, sujeto, turnos }, null, 2))
  unlinkSync(mcp)
}
