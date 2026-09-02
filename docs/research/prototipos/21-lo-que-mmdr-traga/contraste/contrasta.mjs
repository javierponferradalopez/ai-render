// ¿Es Mermaid de verdad lo que mmdr convierte en basura? Sólo el parser de
// Mermaid 11.12.0 — sin Chromium, sin dibujar: lo único que se pregunta aquí es
// si la sintaxis es válida en el idioma que el agente cree estar escribiendo.
import { JSDOM } from 'jsdom'
const dom = new JSDOM('<!doctype html><html><body></body></html>')
globalThis.window = dom.window
globalThis.document = dom.window.document
Object.defineProperty(globalThis, "navigator", { value: dom.window.navigator, configurable: true })
globalThis.DOMPurify = { sanitize: (s) => s, addHook: () => {} }
const { default: mermaid } = await import('mermaid')
mermaid.initialize({ startOnLoad: false, securityLevel: 'loose' })

import { readFileSync, readdirSync } from 'node:fs'
const dir = process.argv[2] ?? '../cases'
for (const f of readdirSync(dir).filter((f) => f.endsWith('.mmd')).sort()) {
  const src = readFileSync(`${dir}/${f}`, 'utf8')
  try {
    const r = await mermaid.parse(src)
    console.log(`ok      ${f.padEnd(26)} ${r?.diagramType ?? ''}`)
  } catch (e) {
    console.log(`ERROR   ${f.padEnd(26)} ${String(e.message ?? e).split('\n')[0].slice(0, 110)}`)
  }
}
