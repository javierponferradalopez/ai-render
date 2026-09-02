import { spawn } from 'node:child_process';
import { readFileSync } from 'node:fs';
const args = process.argv.slice(2);
const cases = JSON.parse(readFileSync(args[0],'utf8'));
const p = spawn('node', ['node_modules/claude-mermaid/build/index.js'], {stdio:['pipe','pipe','pipe'], env:{...process.env, XDG_CONFIG_HOME: process.cwd()+'/xdg'}});
let buf=''; const pending=new Map();
p.stdout.on('data', d => { buf+=d; let i;
  while((i=buf.indexOf('\n'))>=0){ const l=buf.slice(0,i).trim(); buf=buf.slice(i+1);
    if(!l) continue; const m=JSON.parse(l);
    if(m.id && pending.has(m.id)){ pending.get(m.id)(m); pending.delete(m.id); } } });
p.stderr.on('data', d => {});
let id=0;
const call = (method, params) => new Promise(res => { const myId=++id; pending.set(myId,res);
  p.stdin.write(JSON.stringify({jsonrpc:'2.0',id:myId,method,params})+'\n'); });
await call('initialize',{protocolVersion:'2025-06-18',capabilities:{},clientInfo:{name:'d',version:'1'}});
p.stdin.write(JSON.stringify({jsonrpc:'2.0',method:'notifications/initialized'})+'\n');
for (const c of cases) {
  const t0 = performance.now();
  const r = await call('tools/call', {name:'mermaid_preview', arguments:c.args});
  const ms = performance.now()-t0;
  console.log(`--- ${c.name}  ${ms.toFixed(0)} ms`);
  console.log(JSON.stringify(r.result ?? r.error, null, 1).slice(0,900));
}
p.kill('SIGINT');
process.exit(0);
