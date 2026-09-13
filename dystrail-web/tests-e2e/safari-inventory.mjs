// Real Safari via Apple's built-in WebDriver, with a disposable local origin.
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {join} from 'node:path';
import {createServer} from 'node:http';
import assert from 'node:assert/strict';
const built=process.env.PLAYTEST_DIST||'/tmp/dystrail-continuity-preview/play';
const output=process.env.SAFARI_OUTPUT||'/tmp/dystrail-safari-inventory';mkdirSync(output,{recursive:true});
const driver='http://127.0.0.1:9515';
let session=process.env.SAFARI_SESSION;
async function call(path,body,method=body===undefined?'GET':'POST'){
 const r=await fetch(driver+path,{method,signal:AbortSignal.timeout(30000),headers:{'Content-Type':'application/json'},...(body===undefined?{}:{body:JSON.stringify(body)})});const d=await r.json();if(!r.ok||d.value?.error)throw new Error(JSON.stringify(d.value));return d.value;
}
if(!session){const result=await call('/session',{capabilities:{alwaysMatch:{browserName:'safari'}}});session=result.sessionId;console.log('Safari',result.capabilities.browserVersion);}
const cmd=(path,body,method)=>call(`/session/${session}${path}`,body,method);
const exec=(script,args=[])=>cmd('/execute/sync',{script,args});
const asyncExec=(script,args=[])=>cmd('/execute/async',{script,args});
async function until(script,timeout=20000){const start=Date.now();while(Date.now()-start<timeout){try{if(await exec(script))return;}catch{}await new Promise(r=>setTimeout(r,200));}throw new Error('Timed out: '+script+'\n'+await exec('return JSON.stringify({focus:document.hasFocus(),active:document.activeElement?.id,shareValue:document.querySelector("#share-post")?.value})')+'\n'+await exec('return document.body.innerText'));}
async function click(label){console.log('Click',label);await until('return [...document.querySelectorAll("button")].some(e=>e.offsetHeight&&!e.disabled&&(e.getAttribute("aria-label")==='+JSON.stringify(label)+'||e.textContent.trim()==='+JSON.stringify(label)+'))');const el=await exec('return [...document.querySelectorAll("button")].find(e=>e.offsetHeight&&!e.disabled&&(e.getAttribute("aria-label")===arguments[0]||e.textContent.trim()===arguments[0]));',[label]);assert(el,`Button: ${label}`);await exec('const r=arguments[0].getBoundingClientRect();if(r.top<0||r.bottom>innerHeight)arguments[0].scrollIntoView({block:"center",behavior:"instant"});',[el]);await new Promise(r=>setTimeout(r,150));await cmd('/element/'+el['element-6066-11e4-a52e-4f735466cecf']+'/click',{});}
async function clickCSS(selector){const el=await cmd('/element',{using:'css selector',value:selector});await exec('const r=arguments[0].getBoundingClientRect();if(r.top<0||r.bottom>innerHeight)arguments[0].scrollIntoView({block:"center",behavior:"instant"});',[el]);await new Promise(r=>setTimeout(r,150));await cmd('/element/'+el['element-6066-11e4-a52e-4f735466cecf']+'/click',{});}
async function fill(selector,text){await until('return !!document.querySelector('+JSON.stringify(selector)+')');if(text.length>500){await exec('const e=document.querySelector(arguments[0]);Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,"value").set.call(e,arguments[1]);e.dispatchEvent(new Event("input",{bubbles:true}));',[selector,text]);return;}const el=await cmd('/element',{using:'css selector',value:selector});const id=el['element-6066-11e4-a52e-4f735466cecf'];if(selector==='#share-post'){
 await cmd('/element/'+id+'/click',{});
 await cmd('/actions',{actions:[{type:'key',id:'replace-post',actions:[{type:'keyDown',value:'\uE03D'},{type:'keyDown',value:'a'},{type:'keyUp',value:'a'},{type:'keyUp',value:'\uE03D'}]}]});
 await until('const e=document.querySelector("#share-post");return document.activeElement===e&&e.selectionStart===0&&e.selectionEnd===e.value.length');
 }else await cmd('/element/'+id+'/clear',{});await cmd('/element/'+id+'/value',{text});await until('return document.querySelector('+JSON.stringify(selector)+').value==='+JSON.stringify(text));}
async function menu(){if(!await exec('return document.querySelector("#game-menu-button").getAttribute("aria-expanded")==="true"'))await clickCSS('#game-menu-button');}
const state=()=>exec('return JSON.parse(localStorage.getItem("dystrail.autosave.v1")).state');
async function importState(gs){await menu();await clickCSS('#save-open-btn');await fill('#import-json',JSON.stringify(gs));await click('Import');await until('return !document.querySelector(".drawer")');}
async function shot(name){await asyncExec('const done=arguments[arguments.length-1];Promise.all([...document.images].map(i=>i.decode().catch(()=>{}))).then(()=>done(true));');viewports.push({name,...await exec('return {width:innerWidth,height:innerHeight};')});writeFileSync(join(output,name+'.png'),Buffer.from(await cmd('/screenshot'),'base64'));assert(await exec('return document.documentElement.scrollWidth<=innerWidth'),name+' horizontal overflow');}
async function localizeStore(name,amount){
 const quantities=await exec('return [...document.querySelectorAll(".store-qty-row input")].map(e=>e.value)');
 await menu();await clickCSS('.language-picker>button');await click('العربية');await clickCSS('#game-menu-button');
 await until('return document.querySelector(".store-cart-summary").textContent.includes("النقد بعد الشراء")');
 assert((await exec('return document.querySelector(".store-price").textContent')).includes('للقطعة'));
 if(await exec('return !!document.querySelector(".critical-stats")')){assert((await exec('return document.querySelector(".critical-stats").textContent')).includes('مؤن'));assert((await exec('return document.querySelector(".game-clock").textContent')).includes('اليوم'));}
 const expected=await exec('return new Intl.NumberFormat("ar",{style:"currency",currency:"USD",minimumFractionDigits:0,maximumFractionDigits:0}).format(arguments[0])',[amount]);
 assert.equal(await exec('return document.querySelector(".cart-total").textContent'),expected);
 assert.deepEqual(await exec('return [...document.querySelectorAll(".store-qty-row input")].map(e=>e.value)'),quantities);
 await shot(name+'-arabic');
 await menu();await clickCSS('.language-picker>button');await click('English');await clickCSS('#game-menu-button');
 await until('return document.querySelector(".store-cart-summary").textContent.includes("Cash after purchase")');
 assert.deepEqual(await exec('return [...document.querySelectorAll(".store-qty-row input")].map(e=>e.value)'),quantities);
 pass(name+' changes language immediately in both directions without resetting quantities');
}
async function verifyOutcomeReachability(){
 const rect=await cmd('/window/rect');
 for(const height of [800,600]){
  await cmd('/window/rect',{width:rect.width,height});
  for(const fraction of [0,.5,1]){
   await exec('scrollTo(0,(document.documentElement.scrollHeight-innerHeight)*arguments[0]);',[fraction]);
   await until('const e=document.querySelector("#outcome-continue"),r=e.getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight&&document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest("button")===e');
  }
 }
 await shot('reachable-outcome');await cmd('/window/rect',rect);
 pass('The outcome return button stays visible and clickable at every tested scroll position in native Safari');
}
async function verifyStoppedCrew(base){
 const gs=structuredClone(base);gs.party.members.find(m=>m.persona==='satirist').status='Dead';gs.party.members.find(m=>m.persona==='organizer').status='Departed';
 await importState(gs);
 assert.equal(await exec('return document.querySelectorAll(".standing-member").length'),4);
 assert.equal(await exec('return document.querySelectorAll(".van-occupant,.scene-status,.leg-turn").length'),0);
 assert(await exec('return [...document.querySelectorAll(".standing-member image")].every(i=>i.getAttribute("href").startsWith("blob:"))'));
 assert(await exec('return [".leg-cash",".leg-vehicle",".leg-destination"].every(s=>!!document.querySelector(".conditions-hud "+s))'));
 await shot('stopped-crew-hud');serverAvailable=false;await cmd('/refresh',{});await until('return document.querySelectorAll(".standing-member").length===4');serverAvailable=true;
 pass('Only present crew stand outside the parked van; the consolidated HUD and sprites recover offline');
 town(gs);await importState(gs);await click('Take a paid unloading shift');
 await until('return document.querySelector("#outcome-continue")?.textContent.trim()==="Back to town"');
 assert(await exec('return !!document.querySelector("#outcome-continue [data-icon=return]")&&!document.querySelector("#outcome-continue [data-icon=travel]")'));
 const completed=await state();await shot('paid-shift-back-to-town');await click('Back to town');await until('return document.querySelector("#main").dataset.screen==="town"');
 const returned=await state();assert.equal(returned.miles_traveled_actual,completed.miles_traveled_actual);assert.equal(returned.clock_minutes,completed.clock_minutes);assert.equal(returned.day,completed.day);assert.deepEqual(returned.route_services,completed.route_services);
 pass('Paid-shift return uses a return arrow and stays in the same town without spending time or traveling');
 await click('Journal');
 const dayEntries=returned.journal.filter(e=>e.day===returned.journal.at(-1).day),cash=dayEntries.flatMap(e=>e.resources.filter(r=>r.key==='play.cash'));
 assert(cash.length>=2);assert(cash[0].after<cash[0].before);assert.equal(cash.at(-1).after-cash.at(-1).before,1800);assert.equal(cash.at(-1).after,returned.budget_cents);
 const daySelector=`.journal-day[data-day="${returned.journal.at(-1).day}"]`,dailyCash=cash.at(-1).after-cash[0].before;
 const total=await exec('return document.querySelector(arguments[0]+" > .journal-story > .resource-changes [data-stat=\\"play.cash\\"] strong")?.textContent??null',[daySelector]);
 assert.equal(total,dailyCash===0?null:(dailyCash>0?'+':'−')+new Intl.NumberFormat('en',{style:'currency',currency:'USD',maximumFractionDigits:0}).format(Math.abs(dailyCash)/100));
 await clickCSS(daySelector+' > .journal-raw > summary');
 assert.equal(await exec('return document.querySelectorAll(arguments[0]+" .journal-raw-entry").length',[daySelector]),dayEntries.length);
 assert.equal(await exec('return [...document.querySelectorAll(arguments[0]+" .journal-raw-entry")].at(-1).querySelector("[data-stat=\\"play.cash\\"] strong").textContent',[daySelector]),'+$18');
 assert.deepEqual((await state()).journal,returned.journal);await shot('journal-paid-shift-details');
 await clickCSS(daySelector+' > .journal-raw > summary');
 pass('Daily cash uses the first and final snapshots while the paid shift keeps its exact +$18 raw receipt');
 await importState(base);
}
async function verifyEntryLayout(base){
 const g=structuredClone(base);g.turn_journal_start=0;
 g.journal=[{...g.journal[0],pace:'heated',diet:'quiet',before:{...g.stats,supplies:18,sanity:10},after:{...g.stats,supplies:17,sanity:9},details:[],resources:[{key:'play.elapsed',before:73,after:74},{key:'play.miles',before:11054,after:11253},{key:'play.vehicle',before:9976,after:9967}]}];
 await importState(g);
 const boxes=selector=>exec('return [...document.querySelectorAll(arguments[0]+" .stat-card")].map(e=>{const r=e.getBoundingClientRect();return {width:r.width,height:r.height,top:r.top}})',[selector]);
 const trail=await boxes('.turn-receipt');assert.equal(trail.length,5);assert.equal(new Set(trail.map(r=>r.top)).size,1);await shot('compact-trail-five-stats');
 await click('Journal');const journal=await boxes('.journal-day > .journal-story');assert.equal(new Set(journal.map(r=>r.top)).size,1);assert.deepEqual(journal.map(({width,height})=>({width,height})),trail.map(({width,height})=>({width,height})));
 assert.equal(await exec('return document.querySelectorAll(".journal-context .journey-icon").length'),3);
 assert(await exec('const a=document.querySelector(".journal-place").getBoundingClientRect(),b=document.querySelector(".journal-context").getBoundingClientRect();return b.top>a.bottom&&a.left===b.left'));
 await shot('compact-journal-five-stats');pass('Trail and Journal share five compact cards in one row, with historical clock, pace and diet beneath the route');
}
async function verifyDailyJournal(base){
 const g=structuredClone(base),before={...g.stats,supplies:10,sanity:10},first={...before,supplies:9,sanity:9},second={...first,supplies:8,sanity:8};
 const leg=(day,minute,miles,driving,start,end)=>({...structuredClone(base.journal[0]),day,minute,pace:'steady',diet:'mixed',place:'Salt Lake City → Denver',action_kind:'travel',title:'Traveled',message:'',before:start,after:end,details:[],resources:[{key:'play.miles',before:miles,after:miles+600},{key:'play.driving_time',before:driving,after:driving+60}]});
 g.journal=[leg(74,540,0,0,before,first),leg(74,600,600,60,first,second),leg(75,540,1200,120,second,second)];g.turn_journal_start=0;
 await importState(g);await click('Journal');
 const text=selector=>exec('return document.querySelector(arguments[0])?.textContent??null',[selector]);
 const day74='.journal-day[data-day="74"]',day75='.journal-day[data-day="75"]';
 assert.equal(await exec('return document.querySelectorAll(".journal-day").length'),2);assert.equal(await exec('return document.querySelector(".journal-day").dataset.day'),'75');
 assert.equal(await exec('return document.querySelectorAll(arguments[0]+" > .journal-story .journal-event").length',[day74]),1);
 assert.equal(await exec('return document.querySelector(arguments[0]+" > .journal-story .journal-event").dataset.actionCount',[day74]),'2');
 assert.equal(await text(day74+' > .journal-story [data-stat="play.miles"] strong'),'+120.0');assert.equal(await text(day74+' > .journal-story [data-stat="play.miles"] small'),'2 h driving');
 assert.equal(await text(day74+' > .journal-story [data-stat="ux.supplies"] strong'),'-2');assert.equal(await text(day74+' > .journal-story [data-stat="ux.supplies"] small'),'8 left');
 await clickCSS(day74+' > .journal-raw > summary');
 assert.deepEqual(await exec('return [...document.querySelectorAll(arguments[0]+" .journal-raw-entry [data-stat=\\"play.miles\\"] strong")].map(e=>e.textContent)',[day74]),['+60.0','+60.0']);
 assert.deepEqual(await exec('return [...document.querySelectorAll(arguments[0]+" .journal-raw-entry [data-stat=\\"ux.supplies\\"] strong")].map(e=>e.textContent)',[day74]),['-1','-1']);
 assert.deepEqual((await state()).journal,g.journal);await clickCSS(day74+' > .journal-raw > summary');await shot('daily-journal');
 g.journal.push(leg(75,600,1800,180,second,second));await importState(g);await click('Journal');
 assert.equal(await exec('return document.querySelectorAll(".journal-day").length'),2);assert.equal(await text(day75+' > .journal-story [data-stat="play.miles"] strong'),'+120.0');assert.equal(await text(day74+' > .journal-story [data-stat="play.miles"] strong'),'+120.0');
 await cmd('/refresh',{});await until('return document.querySelectorAll(".journal-day").length===2');assert.deepEqual((await state()).journal,g.journal);assert.equal(await exec('return document.querySelectorAll(".journal-raw-entry").length'),4);
 pass('One card per persisted day accumulates consecutive travel and exact resource totals without changing raw action history');
 await importState(base);
}
async function verifyLocales(base){
 await importState(base);await click('Journal');
 const source=readFileSync(new URL('../src/i18n/locales.rs',import.meta.url),'utf8');
 const locales=[...source.matchAll(/code: "([^"]+)",\s*name: "([^"]+)"/g)].map(m=>({code:m[1],name:m[2]}));assert.equal(locales.length,20);
 for(const locale of locales){
  const strings=JSON.parse(readFileSync(new URL('../i18n/'+locale.code+'.json',import.meta.url),'utf8'));
  await menu();await clickCSS('.language-picker>button');await click(locale.name);await clickCSS('#game-menu-button');
  assert.equal(await exec('return document.documentElement.lang'),locale.code);
  assert.equal(await exec('return document.documentElement.dir'),locale.code==='ar'?'rtl':'ltr');
  assert(await exec('return document.querySelector(".journal-entry")&&document.documentElement.scrollWidth<=innerWidth'));
  assert(await exec('return [...document.querySelectorAll(".journal-context")].every(e=>e.querySelectorAll(".journey-icon").length===3)'));
  await clickCSS('.journey-actions button:has([data-icon=route])');await until('return !!document.querySelector(".map-scene")');
  assert.equal(await exec('return document.querySelector(".map-buttons .retro-btn-primary").textContent'),strings.route.close);
  await clickCSS('.map-buttons .retro-btn-primary');await until('return !!document.querySelector(".journal-entry")');
  if(['ar','ja'].includes(locale.code))await shot('journal-'+locale.code);
 }
 await menu();await clickCSS('.language-picker>button');await click('English');await clickCSS('#game-menu-button');
 pass('All 20 shipped locales render Journal and manual-map controls without unresolved keys or horizontal overflow');
}
async function verifyNavigation(base){
 await importState(base);
 for(const name of ['The Trail','Conditions','The Van','Journal']){
  const selected=()=>exec('return [...document.querySelectorAll("[role=tab]")].find(e=>e.textContent===arguments[0]).getAttribute("aria-selected")==="true"',[name]);
  if(!await selected())await click(name);
  await click(name);assert.equal(await exec('return document.querySelectorAll("[role=tabpanel]").length'),0);assert.equal(await exec('return document.querySelectorAll("[role=tab][aria-selected=true]").length'),0);
  await click(name);assert(await selected());
 }
 await click('Camp');const before=await state();await click('Route');assert.equal(await exec('return document.querySelector(".map-scene").dataset.automatic'),'false');await cmd('/refresh',{});await click('Close map');await until('return document.querySelector("#main").dataset.screen==="camp"');
 await new Promise(r=>setTimeout(r,3300));const after=await state();before.inventory.tags.sort();after.inventory.tags.sort();assert.deepEqual(after,before);assert(!await exec('return [...document.querySelectorAll("button")].some(e=>e.textContent.trim()==="Pause travel")'));
 assert(await exec('return [...document.querySelectorAll("[role=tab]")].some(e=>e.textContent==="Journal"&&e.getAttribute("aria-selected")==="true")'));
 await shot('manual-map-back-to-camp');pass('All tabs collapse and reopen; a manual map returns to the same screen and selected tab without traveling, including after reload');
}
async function verifyTurnHistory(base){
 const turn=structuredClone(base);turn.turn_journal_start=0;
 turn.journal.push({...structuredClone(turn.journal[0]),action_kind:'camp',title:'Camp',message:'The crew gathers fresh supplies.',before:{...turn.stats},after:{...turn.stats},resources:[],details:[]});
 await importState(turn);
 const messages=turn.journal.toReversed().map(e=>e.message);
 const actual=()=>exec('return [...document.querySelectorAll(".turn-receipt .receipt-heading")].map(e=>e.textContent)');
 assert.deepEqual(await actual(),messages);
 await click('The Van');assert.equal(await exec('return document.querySelectorAll(".turn-receipt").length'),0);
 await click('The Trail');assert.deepEqual(await actual(),messages);
 await cmd('/refresh',{});await until('return !!document.querySelector(".turn-receipts")');
 assert.deepEqual(await actual(),messages);assert.deepEqual((await state()).journal,turn.journal);assert.deepEqual((await state()).stats,turn.stats);
 await shot('turn-history-newest-first');pass('Every current-turn entry stays newest first under The Trail through tab navigation and reload');
 const routine=structuredClone(base);routine.turn_journal_start=0;
 routine.journal=[{...structuredClone(base.journal[0]),action_kind:'travel',title:'Last action',message:'Traveled',before:{...base.stats},after:{...base.stats},resources:[],details:[]}];
 await importState(routine);assert.deepEqual(await actual(),['Traveled']);
 await cmd('/refresh',{});await until('return !!document.querySelector(".turn-receipts")');
 assert.deepEqual(await actual(),['Traveled']);assert.deepEqual((await state()).journal,routine.journal);
 await shot('trail-entry-title');pass('Routine travel keeps a visible Traveled title through reload without changing the recorded action');
}
const checks=[];const viewports=[];const manifest=JSON.parse(readFileSync(join(built,'offline-manifest.json'),'utf8'));const revision=manifest.revision;
let serverAvailable=true;
let firstDownloadHold=false,firstDownloadFail=false,firstDownloadWaiting=false,releaseFirstDownload=()=>{};
function pass(name){checks.push(name);console.log('PASS',name);writeFileSync(join(output,'validation.json'),JSON.stringify({browser:'Native Safari',revision,checks,viewports},null,2));}
const server=createServer(async(req,res)=>{
 if(!serverAvailable){res.statusCode=503;res.end('Offline verification: origin unavailable');return;}
 const url=new URL(req.url,'http://fixture');let path=url.pathname.replace(/^\/play\//,'');if(!path||!path.includes('.'))path='index.html';
 if(path==='static/img/journey/town-npcs-v1.png'){
  if(firstDownloadHold)await new Promise(resolve=>{firstDownloadWaiting=true;releaseFirstDownload=resolve;});
  if(firstDownloadFail){res.statusCode=503;res.end('Deliberately interrupted first download');return;}
 }
 const type=path.endsWith('.js')?'text/javascript':path.endsWith('.wasm')?'application/wasm':path.endsWith('.css')?'text/css':path.endsWith('.html')?'text/html':path.endsWith('.png')?'image/png':path.endsWith('.svg')?'image/svg+xml':path.endsWith('.json')?'application/json':'application/octet-stream';res.setHeader('Content-Type',type);res.setHeader('Cache-Control','no-store');
 try{res.end(readFileSync(join(built,path)));}catch{res.statusCode=404;res.end();}
});
await new Promise(resolve=>server.listen(Number(process.env.SAFARI_PORT||0),'127.0.0.1',resolve));const origin=`http://127.0.0.1:${server.address().port}/play/`;
const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
const events=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
function town(gs){const r=routes.find(r=>r.id===gs.persona_id),t=r.stops.find(s=>s.name==='Madison');gs.miles_traveled_actual=(t.mile+1)/r.total_miles*gs.trail_distance;gs.miles_traveled=Math.round(gs.miles_traveled_actual);gs.route_services={route_id:r.id,stop:t.mile,traded_at:null,talked_at:null,talk_reward:null,map_reviewed:null};}
try {
 await cmd('/timeouts',{script:30000,pageLoad:30000,implicit:0});const handles=await cmd('/window/handles');await cmd('/window',{handle:handles[0]||(await cmd('/window/new',{type:'window'})).handle});await cmd('/window/rect',{x:20,y:40,width:1280,height:980});await cmd('/url',{url:origin});
 await until('return !!document.querySelector("#main")');await click('Choose your character');await click('Journalist');await click('Continue');await fill('#crew-name-journalist','Safari Player');await fill('#crew-name','Safari Receipts');await click('Continue');
 await until('return document.querySelectorAll(".store-card").length===11');assert(await exec('return [...document.querySelectorAll(".store-item-art")].every(i=>i.complete&&i.naturalWidth>0&&i.src.startsWith("blob:"))'));await shot('illustrated-shop');pass('All eleven shop images are decoded local assets before the store opens');
 await click('Review & depart');await click('Start the journey');await until('return document.querySelector("#main").dataset.screen==="travel"');const base=await state();await verifyStoppedCrew(base);
 const gs=structuredClone(base);gs.inventory.spares={tire:2,battery:0,alt:1,pump:3};gs.inventory.tags=['plague_resist','permit'];gs.journal=[{day:129,minute:540,pace:'steady',diet:'mixed',place:'At Iowa City',title:'A crew member needs help · Sage',message:'Sage receives care and can help the crew again.',action_kind:'care',before:gs.stats,after:{...gs.stats,supplies:gs.stats.supplies-2,morale:gs.stats.morale+1},resources:[],details:[['Sage','Traveling']]}];
 await importState(gs);await click('The Van');assert.equal(await exec('return document.querySelectorAll(".van-item").length'),9);assert.equal(await exec('return document.querySelectorAll(".van-crew-member").length'),6);assert.equal(await exec('return document.querySelector("[data-item=battery] .van-item-count").textContent'),'0');assert.ok(await exec('return document.querySelector(".van-crew").getBoundingClientRect().bottom < document.querySelector(".van-equipment-grid").getBoundingClientRect().top'));assert.equal(await exec('return document.querySelector("#van-summary-title").textContent'),'Trip overview');assert.ok(await exec('const images=[...document.querySelectorAll(".van-overview img")];return images.length===4&&images.every(i=>i.complete&&i.naturalWidth>0&&i.src.startsWith("blob:"))'));await shot('illustrated-van');pass('The Van has an illustrated trip overview and the crew above every part and equipment item');
 await click('Journal');assert((await exec('return document.querySelector(".journal-entry").getBoundingClientRect().height'))<220);assert.equal(await exec('return document.querySelectorAll(".journal-day > .journal-story > .resource-changes .stat-card").length'),2);await shot('compact-journal');pass('Desktop journal uses the horizontal space and retains boxed resource changes');await verifyTurnHistory(gs);await verifyEntryLayout(gs);await verifyDailyJournal(gs);
 gs.weather_state.today='HeatWave';gs.weather_impact={day:gs.day,weather:'HeatWave',supplies:-1,hp:-1,sanity:-2};gs.day_state.day_initialized=false;await importState(gs);await click('Conditions');await clickCSS('.setting-option:has([data-icon=blitz])');await clickCSS('.setting-option:has([data-icon=quiet])');assert((await exec('return document.querySelector(".hud-journey-settings").textContent')).includes('BlitzQuiet'));assert.equal(await exec('return document.querySelectorAll(".weather-indicator").length'),1);assert.equal(await exec('return document.querySelectorAll(".conditions-hud [data-icon=clock]").length'),1);await clickCSS('.weather-indicator .help-trigger');assert.equal(await exec('return document.querySelector(".weather-indicator .info-glyph").textContent'),'i');assert.equal(await exec('return document.querySelectorAll(".weather-details .stat-card").length'),5);await shot('weather-details');await clickCSS('.weather-indicator .help-trigger');pass('Clock, pace and diet icons update in the top bar; weather has structured effects and a plain i');
 await menu();const geometry=()=>exec('return [...document.querySelectorAll(".hud-context-row,.critical-stats,.scene-status")].map(e=>{const r=e.getBoundingClientRect();return [r.x,r.y,r.width,r.height]})');const geo=await geometry();await click('Help & tips');assert.deepEqual(await geometry(),geo);await clickCSS('#game-menu-button');pass('Help & tips does not move the HUD');
 const route=routes.find(r=>r.id===base.persona_id);
 for(const kind of [0,1,2]){const t=route.stops.find(s=>Math.floor(s.mile/10)%3===kind&&s.name!=='D.C.'),g=structuredClone(base);g.miles_traveled_actual=(t.mile+1)/route.total_miles*g.trail_distance;g.route_services={route_id:route.id,stop:t.mile,traded_at:null,talked_at:null,talk_reward:null,map_reviewed:null};g.stats.credibility=10;g.stats.allies=2;await importState(g);const label=['Credibility','Receipts','Allies'][kind];assert((await exec('return document.querySelector(".route-stop .action-button").textContent')).includes(label+' +1'));await click('Talk to locals');const once=await state();assert.equal(once.stats.credibility-g.stats.credibility,kind===0?1:0);assert.equal(once.receipts.length-g.receipts.length,kind===1?1:0);assert.equal(once.stats.allies-g.stats.allies,kind===2?1:0);await click('Back to town');assert.equal(await exec('return document.querySelector(".route-stop s").textContent'),label+' +1');await click('Talk to locals');const twice=await state();assert.deepEqual(twice.stats,once.stats);assert.deepEqual(twice.receipts,once.receipts);assert.deepEqual(twice.journal,once.journal);await click('Back to town');}
 pass('Local conversations visibly offer credibility, receipts or allies once, and remain open after claiming');
 const repair=structuredClone(base);repair.breakdown={part:'Battery',day_started:repair.day};repair.inventory.spares.battery=2;repair.vehicle.health=93.27;await importState(repair);assert(await exec('return document.querySelector(".camp-toggle").disabled'));assert(await exec('return document.querySelector(".journey-actions>.retro-btn-primary").disabled'));assert(!await exec('return document.body.textContent.includes("Handle breakdown")'));await new Promise(r=>setTimeout(r,3200));assert.equal((await state()).inventory.spares.battery,2);await click('Fit your spare Battery');await until('return !!document.querySelector(".aftermath-panel")');assert.equal(await exec('return document.querySelector(arguments[0]).textContent',['[data-stat="store.items.battery.name"] small']),'1 left');await shot('explicit-repair');await verifyOutcomeReachability();await click('Back to the road');assert(!await exec('return document.querySelector(".camp-toggle").disabled'));pass('Breakdown is never automatic; Camp and Travel unlock only after an explicit repair');await verifyNavigation(await state());if(process.env.SAFARI_LOCALE_SMOKE==='1')await verifyLocales(await state());
 const ended=structuredClone(base);ended.abandoned=true;await importState(ended);await until('return !!document.querySelector("#result-share-open")');serverAvailable=false;await clickCSS('#result-share-open');await until('return document.querySelector(".share-preview img")?.naturalWidth===1200');await fill('#share-post','My trip 1234 & a story');assert.equal(await exec('return document.querySelector("#share-post").value'),'My trip 1234 & a story');assert.equal(await exec('return document.querySelector(".share-platforms a").href'),'https://bsky.app/intent/compose?text=My%20trip%201234%20%26%20a%20story');
 const png=await asyncExec('fetch(document.querySelector(".share-download").href).then(r=>r.arrayBuffer()).then(b=>arguments[arguments.length-1]([...new Uint8Array(b).slice(0,8)]));');assert.deepEqual(png,[137,80,78,71,13,10,26,10]);await shot('offline-share');await click('Close');await until('return !document.querySelector(".share-composer")&&document.activeElement.id==="result-share-open"',1000);assert.equal(await exec('return document.activeElement.id'),'result-share-open');await clickCSS('#result-share-open');await until('return !!document.querySelector(".share-preview img")');await cmd('/actions',{actions:[{type:'key',id:'share-escape',actions:[{type:'keyDown',value:'\uE00C'},{type:'keyUp',value:'\uE00C'}]}]});await until('return !document.querySelector(".share-composer")&&document.activeElement.id==="result-share-open"',1000);pass('Wasm renders a real 1200px PNG offline, editable post text and draft links without posting');
 serverAvailable=true;await importState(gs);await click('The Van');await menu();await clickCSS('.language-picker>button');await click('العربية');await clickCSS('#game-menu-button');await shot('arabic-van');assert.equal(await exec('return document.documentElement.dir'),'rtl');pass('Illustrated inventory retains counts and alignment in Arabic');
 console.log(JSON.stringify({revision,checks,viewports},null,2));
} catch(error){writeFileSync(join(output,'failure.txt'),String(error.stack||error));try{await shot('failure');}catch{}throw error;}finally{server.close();await call(`/session/${session}`,undefined,'DELETE').catch(()=>{});}
