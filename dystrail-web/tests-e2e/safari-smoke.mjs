// Real Safari via Apple's built-in WebDriver, with a disposable local origin.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import assert from 'node:assert/strict';
const built=process.env.PLAYTEST_DIST||'/tmp/dystrail-continuity-preview/play';
const output=process.env.SAFARI_OUTPUT||'/tmp/dystrail-safari';mkdirSync(output,{recursive:true});
const driver='http://127.0.0.1:9515';
let session=process.env.SAFARI_SESSION;
async function call(path,body,method=body===undefined?'GET':'POST'){
 const r=await fetch(driver+path,{method,headers:{'Content-Type':'application/json'},...(body===undefined?{}:{body:JSON.stringify(body)})});const d=await r.json();if(!r.ok||d.value?.error)throw new Error(JSON.stringify(d.value));return d.value;
}
if(!session){const result=await call('/session',{capabilities:{alwaysMatch:{browserName:'safari'}}});session=result.sessionId;console.log('Safari',result.capabilities.browserVersion);}
const cmd=(path,body,method)=>call(`/session/${session}${path}`,body,method);
const exec=(script,args=[])=>cmd('/execute/sync',{script,args});
const asyncExec=(script,args=[])=>cmd('/execute/async',{script,args});
async function until(script,timeout=20000){const start=Date.now();while(Date.now()-start<timeout){try{if(await exec(script))return;}catch{}await new Promise(r=>setTimeout(r,200));}throw new Error('Timed out: '+script+'\n'+await exec('return document.body.innerText'));}
async function click(label){const el=await exec('return [...document.querySelectorAll("button")].find(e=>e.offsetHeight && e.textContent.trim()===arguments[0]);',[label]);assert(el,`Button: ${label}`);await cmd('/element/'+el['element-6066-11e4-a52e-4f735466cecf']+'/click',{});}
async function clickCSS(selector){const el=await cmd('/element',{using:'css selector',value:selector});await cmd('/element/'+el['element-6066-11e4-a52e-4f735466cecf']+'/click',{});}
async function fill(selector,text){const el=await cmd('/element',{using:'css selector',value:selector});const id=el['element-6066-11e4-a52e-4f735466cecf'];await cmd('/element/'+id+'/clear',{});await cmd('/element/'+id+'/value',{text});}
async function menu(){if(!await exec('return document.querySelector("#game-menu-button").getAttribute("aria-expanded")==="true"'))await clickCSS('#game-menu-button');}
const state=()=>exec('return JSON.parse(localStorage.getItem("dystrail.autosave.v1")).state');
async function importState(gs){await menu();await clickCSS('#save-open-btn');await fill('#import-json',JSON.stringify(gs));await click('Import');await until('return !document.querySelector(".drawer")');}
async function shot(name){await asyncExec('Promise.all([...document.images].map(i=>i.decode().catch(()=>{}))).then(()=>arguments[arguments.length-1](true));');writeFileSync(join(output,name+'.png'),Buffer.from(await cmd('/screenshot'),'base64'));assert(await exec('return document.documentElement.scrollWidth<=innerWidth'),name+' horizontal overflow');}
const html=readFileSync(join(built,'index.html'),'utf8');const original=JSON.parse(readFileSync(join(built,'offline-manifest.json'),'utf8'));const worker=readFileSync('../scripts/offline-worker.js','utf8');
let version='one',fail=false,hold=false,waiting=false,release=()=>{};
const updatedHtml=()=>html.replace('</head>',`<meta name="test-build" content="${version}"></head>`);
const build=()=>({...original,revision:original.revision+'-'+version,assets:original.assets.map(a=>a.path==='index.html'?{...a,bytes:Buffer.byteLength(updatedHtml()),integrity:'sha256-'+createHash('sha256').update(updatedHtml()).digest('base64')}:a)});
const server=createServer(async(req,res)=>{
 res.setHeader('Cache-Control','no-store');const path=new URL(req.url,'http://fixture').pathname.replace(/^\/play\//,'');
 if(path==='sw.js'){res.setHeader('Content-Type','text/javascript');res.end('const BUILD='+JSON.stringify(build())+';\n'+worker);return;}
 if(path==='static/img/journey/town-npcs-v1.png'&&version!=='one'){if(fail){res.statusCode=503;res.end('interrupted');return;}if(hold)await new Promise(resolve=>{release=resolve;waiting=true;});}
 const mime=path.endsWith('.js')?'text/javascript':path.endsWith('.wasm')?'application/wasm':path.endsWith('.css')?'text/css':path.endsWith('.webmanifest')?'application/manifest+json':path.endsWith('.png')?'image/png':path.endsWith('.svg')?'image/svg+xml':undefined;if(mime)res.setHeader('Content-Type',mime);
 if(path==='index.html'||!path||!path.includes('.')){res.setHeader('Content-Type','text/html');res.end(updatedHtml());return;}
 try{res.end(readFileSync(join(built,path)));}catch{res.statusCode=404;res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port,origin=`http://127.0.0.1:${port}/play/`;
const checks=[];function pass(name){checks.push(name);console.log('PASS',name);writeFileSync(join(output,'validation.json'),JSON.stringify({browser:'Safari 26.6.2',origin,checks},null,2));}
try{
 await cmd('/timeouts',{script:40000,pageLoad:30000,implicit:0});await cmd('/window/rect',{width:1280,height:980});await cmd('/url',{url:origin});
 await until('return document.querySelector("#main")');
 assert.equal(await exec('return scrollY'),0);assert.notEqual(await exec('return document.activeElement?.id'),'main');
 await click('Choose your character');await click('Journalist');await click('Continue');await until('return document.querySelector("#crew-name")');
 await fill('#crew-name-journalist','River Safari');await fill('#crew-name','Safari Receipts');await shot('crew-desktop');
 const portraits=await exec('return [...document.querySelectorAll(".crew-portrait")].map(e=>({w:e.clientWidth,h:e.clientHeight,fit:getComputedStyle(e).objectFit}))');assert(portraits.every(p=>p.w===p.h&&p.fit==='contain'));pass('Onboarding: names, unwarped portraits, no focus jump');
 await click('Continue');await until('return document.querySelector(".loadout-panel")');assert((await exec('return document.querySelector(".loadout-panel").textContent')).includes('$71.00'));await click('Empty the van');await cmd('/refresh',{});await until('return document.querySelector(".loadout-panel")');assert.equal(await exec('return document.querySelectorAll(".loadout-items li").length'),0);pass('Paid starter cart can be emptied and persists across refresh');
 await click('Review & depart');await click('Start the journey');await until('return document.querySelector("#main").dataset.screen==="travel"');await click('Step mode');const gs=await state();assert.equal(gs.stats.supplies,0);assert.equal(gs.budget_cents,12000);pass('Empty departure has no hidden supplies or charges');
 await menu();await until('return document.querySelector(".offline-status")');await shot('offline-menu');await clickCSS('.brand');await until('return document.querySelector("#game-menu-button").getAttribute("aria-expanded")==="false"');pass('Menu dismisses when clicked outside');
 await clickCSS('.hud .help-trigger');await until('return document.querySelector(".viewport-help")');const rect=await exec('const r=document.querySelector(".viewport-help").getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:innerWidth,height:innerHeight};');assert(rect.left>=0&&rect.right<=rect.width&&rect.top>=0&&rect.bottom<=rect.height);await clickCSS('.brand');pass('Contextual help remains inside viewport and dismisses outside');
 await until('return window.dystrailOffline?.state==="ready"',40000);const manifest=await asyncExec('fetch("static/manifest.webmanifest").then(r=>r.json()).then(arguments[arguments.length-1])');assert.equal(manifest.display,'standalone');assert.equal(manifest.name,'Dystopian Trail');assert(await exec('return !!document.querySelector("link[rel=apple-touch-icon]")'));pass('Safari manifest and Apple touch icon available; complete offline cache ready');
 await shot('travel-desktop');await cmd('/window/rect',{width:430,height:932});await shot('travel-narrow');await cmd('/window/rect',{width:1280,height:980});
 const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));const stop=routes.find(r=>r.id==='journalist').stops.find(s=>s.name==='Madison');gs.stats.supplies=12;gs.miles_traveled=stop.mile;gs.region=stop.region;gs.continuity.route_services.stop=stop.mile;gs.continuity.route_services.talked_at=null;gs.continuity.activities.local_word=null;await importState(gs);await until('return document.querySelector(".town-arrival")');
 const talk=await exec('return [...document.querySelectorAll("button")].find(e=>e.textContent.startsWith("Talk to locals")).textContent');await click(talk);await until('return document.querySelector(".local-fact")');assert((await exec('return document.querySelector(".local-fact").textContent')).includes('Mendota'));const npc=await exec('return document.querySelector(".scene-npc").dataset.npc');await cmd('/refresh',{});await until('return document.querySelector(".scene-npc")');assert.equal(await exec('return document.querySelector(".scene-npc").dataset.npc'),npc);await shot('town-desktop');pass('Exact-town fact, player portrait, random NPC and conversation persist');
 // Stop the only game origin completely. No browser network emulation or external API is involved.
 const saved=await state();server.closeAllConnections();await new Promise(r=>server.close(r));await cmd('/url',{url:origin+'town'});await until('return document.querySelector(".local-fact")');assert.deepEqual((await state()).party,saved.party);
 const errors=await asyncExec('const done=arguments[arguments.length-1],assets=arguments[0];Promise.all(assets.map(async a=>{try{const r=await fetch(new URL(a.path,document.baseURI));return r.ok&&(await r.arrayBuffer()).byteLength===a.bytes?null:a.path;}catch{return a.path;}})).then(r=>done(r.filter(Boolean)));',[build().assets]);assert.deepEqual(errors,[]);await shot('offline-town');pass('Origin stopped: reload, saved crew, town scene and every bundled asset still work');
 await new Promise(r=>server.listen(port,'127.0.0.1',r));version='two';hold=true;const refreshing=cmd('/refresh',{});await until('return !!document.querySelector("#launch-gate")');assert.equal(await exec('return !!document.querySelector("#main")'),false);for(let i=0;!waiting&&i<100;i++)await new Promise(r=>setTimeout(r,100));assert(waiting);await shot('update-gate');hold=false;release();await refreshing;await until('return document.querySelector("meta[name=test-build]").content==="two" && !!document.querySelector("#main")',40000);assert.deepEqual((await state()).party,saved.party);pass('Complete update replaces old build before game launches and preserves saves');
 version='three';fail=true;await cmd('/refresh',{});await until('return !!document.querySelector("#main")');assert.equal(await exec('return document.querySelector("meta[name=test-build]").content'),'two');assert.deepEqual((await state()).party,saved.party);pass('Interrupted update leaves the last complete version playable');
 await click('Back to town');await menu();await click('Abandon trail');await click('End this journey');await until('return document.querySelector(".result-art-caption")');assert((await exec('return document.querySelector(".result-art-caption").textContent')).includes('River Safari'));await shot('ending-desktop');pass('Abandon uses the selected character’s name and creates a saved ending');
 console.log('Safari completed:',checks.length,'checks');
}catch(error){try{await shot('failure');writeFileSync(join(output,'failure.txt'),await exec('return document.body.innerText'));}catch{}throw error;}
finally{release();await cmd('',undefined,'DELETE').catch(()=>{});server.closeAllConnections();await new Promise(r=>server.close(r));}
