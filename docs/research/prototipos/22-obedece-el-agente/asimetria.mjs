#!/usr/bin/env node
// El apartado 3 de #28, cerrado en #30 con el material que ya existia.
//
// analiza.mjs lee `registros`, y el servidor falso no registro ningun show
// (--allowedTools no concede herramientas MCP en -p), asi que su detector de
// Nodo fantasma quedo sin estrenar. Los 17 diagramas si estan: viajan dentro
// del tool_use del historial de Claude Code del sujeto, que es de donde los
// saca intentos.mjs --diagramas.
//
// Esto pasa el mismo detector de analiza.mjs sobre esos 17.
// Resultado: 0/17 con Nodo fantasma, y 0 ids desnudos en total.
//
// Uso: node ../22-obedece-el-agente/intentos.mjs <sujeto> --diagramas > diagramas.json
//      node asimetria.mjs diagramas.json
import { readFileSync } from 'node:fs'
const CON_CUERPO = [
  /\b([A-Za-z_][\w.-]*)\s*\[\[[^\]]*\]\]/g, /\b([A-Za-z_][\w.-]*)\s*\(\([^)]*\)\)/g,
  /\b([A-Za-z_][\w.-]*)\s*\{\{[^}]*\}\}/g, /\b([A-Za-z_][\w.-]*)\s*\[[^\]]+\]/g,
  /\b([A-Za-z_][\w.-]*)\s*\([^)]+\)/g, /\b([A-Za-z_][\w.-]*)\s*\{[^}]+\}/g,
  /\b([A-Za-z_][\w.-]*)\s*>[^\]]+\]/g, /^\s*class\s+([A-Za-z_][\w.-]*)/gm,
  /^\s*subgraph\s+([A-Za-z_][\w.-]*)/gm, /^\s*([A-Za-z_][\w.-]*)\s*:\s*\S/gm,
]
const RELACION = /^\s*([A-Za-z_][\w.-]*)(?:\s*(?:\[[^\]]*\]|\([^)]*\)|\{[^}]*\}))?\s*(?:<\|--|\*--|o--|<--|--\|>|--\*|--o|-->|---|-\.->|-\.-|==>|===|--)(?:\|[^|]*\|)?\s*([A-Za-z_][\w.-]*)/gm
const KW = new Set(['flowchart','graph','classDiagram','subgraph','end','class','direction','LR','RL','TB','BT','TD','style','classDef','linkStyle','click','note','sequenceDiagram','participant','autonumber'])
function fantasmas(src){
  const conCuerpo=new Set()
  for(const re of CON_CUERPO) for(const m of src.matchAll(re)) if(!KW.has(m[1])) conCuerpo.add(m[1])
  const enRel=new Set()
  for(const m of src.matchAll(RELACION)) for(const id of [m[1],m[2]]) if(!KW.has(id)) enRel.add(id)
  if(conCuerpo.size===0) return []
  return [...enRel].filter(id=>!conCuerpo.has(id))
}
const ds = JSON.parse(readFileSync(process.argv[2] ?? 'diagramas.json','utf8'))
let conF=0
ds.forEach((d,i)=>{
  const fam = (d.diagram.trim().split('\n')[0]||'').trim().split(/\s+/)[0]
  const f = fantasmas(d.diagram)
  if(f.length) conF++
  console.log(`${String(i+1).padStart(2)} [${fam}] "${d.view_id}" -> ${f.length? 'FANTASMA: '+f.join(', ') : 'ok'}`)
})
console.log(`\nCon Nodo fantasma: ${conF}/${ds.length}`)

console.log('\n--- reparto de ids por diagrama (etiquetados vs desnudos) ---')
ds.forEach((d,i)=>{
  const conCuerpo=new Set()
  for(const re of CON_CUERPO) for(const m of d.diagram.matchAll(re)) if(!KW.has(m[1])) conCuerpo.add(m[1])
  const enRel=new Set()
  for(const m of d.diagram.matchAll(RELACION)) for(const id of [m[1],m[2]]) if(!KW.has(id)) enRel.add(id)
  const desnudos=[...enRel].filter(id=>!conCuerpo.has(id))
  console.log(`${String(i+1).padStart(2)} etiquetados=${conCuerpo.size} enRelacion=${enRel.size} desnudos=${desnudos.length}`)
})
