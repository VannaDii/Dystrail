import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const out='/tmp/dystrail-narrative-release-prep';
const payload='/tmp/dystrail-narrative-release/release-site';
const manifest=JSON.parse(await readFile(payload+'/play/offline-manifest.json','utf8'));
const expected=process.argv[2];
assert.match(expected||'',/^[a-f0-9]{20}$/,'Pass the final verified revision');
assert.notEqual(expected,'a9ff83cd924a07fdf4ec','Do not verify an unstaged baseline as a new release');
assert.equal(manifest.revision,expected);
const before=JSON.parse(await readFile(out+'/old-production-autosave.json','utf8'));
const profile=out+'/old-production-profile';
const context=await chromium.launchPersistentContext(profile,{headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py',viewport:{width:1440,height:1000},serviceWorkers:'allow'});
await context.addInitScript(()=>{
 let sawGate=false;
 new MutationObserver(()=>{
  const gate=document.querySelector('#launch-gate');
  if(gate)sawGate=true;
  else if(sawGate&&!window.releaseGateReadiness){
   const fonts=window.dystrailPreparedFonts||[];
   window.releaseGateReadiness={revision:window.dystrailOffline?.revision,state:window.dystrailOffline?.state,images:(window.dystrailPreparedImages||[]).length,decoded:(window.dystrailPreparedImages||[]).every(image=>image.complete&&image.naturalWidth>0),fonts:fonts.length,fontsLoaded:fonts.every(face=>face.status==='loaded'&&document.fonts.has(face)),documentFontsStatus:document.fonts.status,documentFontsLoaded:Array.from(document.fonts).every(face=>face.status==='loaded')};
  }
 }).observe(document,{childList:true,subtree:true});
});
const page=context.pages()[0]||await context.newPage();
const errors=[];page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
const report={expected,profile,url:'https://dystrail.com/play/',errors};
const ordered=value=>Array.isArray(value)?value.map(ordered):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,ordered(value[key])])):value;
const normalized=value=>{const state=structuredClone(value);state.inventory.tags.sort();return ordered(state)};
const digest=value=>createHash('sha256').update(JSON.stringify(normalized(value))).digest('hex');
const observe=()=>page.evaluate(()=>({revision:window.dystrailOffline?.revision,readiness:window.dystrailOffline?.state,screen:document.querySelector('#main')?.getAttribute('data-screen'),gate:!!document.querySelector('#launch-gate'),images:Object.keys(window.dystrailAssetUrls||{}).length,decoded:(window.dystrailPreparedImages||[]).every(image=>image.complete&&image.naturalWidth>0),controller:navigator.serviceWorker.controller?.scriptURL,fonts:{prepared:(window.dystrailPreparedFonts||[]).map(face=>({family:face.family,style:face.style,weight:face.weight,status:face.status,registered:document.fonts.has(face)})),documentStatus:document.fonts.status,documentFaces:Array.from(document.fonts).map(face=>({family:face.family,status:face.status}))},atGateRemoval:window.releaseGateReadiness}));
const assertFontReadiness=observation=>{
 assert.equal(observation.fonts.prepared.length,7);
 assert.equal(observation.fonts.prepared.every(face=>face.status==='loaded'&&face.registered),true);
 assert.equal(observation.fonts.documentStatus,'loaded');
 assert.equal(observation.fonts.documentFaces.length,7);
 assert.equal(observation.fonts.documentFaces.every(face=>face.status==='loaded'),true);
 assert.deepEqual(observation.atGateRemoval,{revision:expected,state:'ready',images:68,decoded:true,fonts:7,fontsLoaded:true,documentFontsStatus:'loaded',documentFontsLoaded:true});
};
try{
 // This profile holds the real previous production cache and its saved, untraveled crew.
 await page.goto(report.url,{waitUntil:'domcontentloaded',timeout:60000});
 console.log('Checking the real published update against the preserved production installation');
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main'),expected,{timeout:240000});
 report.updated=await observe();
 assertFontReadiness(report.updated);
 assert.equal(report.updated.screen,'travel');
 const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 await writeFile(out+'/production-updated-autosave.json',JSON.stringify(saved,null,2));
 const expectedSave=structuredClone(before);
 // The preceding production already has the hourly save schema. Preserve every value.
 // Inventory tags are the only order-insensitive collection (a Rust HashSet).
 for(const stateKey of ['state','pending']){
  expectedSave[stateKey].inventory.tags.sort();saved[stateKey].inventory.tags.sort();
 }
 assert.deepEqual(ordered(saved),ordered(expectedSave),'Published narrative update changed the saved envelope');
 report.schemaChanges=[];report.ruleChanges=[];report.addedStateKeys=[];
 report.save={phase:saved.phase,day:saved.state.day,miles:saved.state.miles_traveled_actual,stateSha256:digest(saved.state),playerStatePreserved:true};
 report.cache=await page.evaluate(async manifest=>{
  const name='dystopian-trail:/play/:'+manifest.revision,cache=await caches.open(name),failures=[];
  for(const asset of manifest.assets){
   const response=await cache.match(new URL(asset.path,'https://dystrail.com/play/'));
   if(!response){failures.push(asset.path+': missing');continue;}
   const bytes=await response.arrayBuffer();
   const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),byte=>String.fromCharCode(byte)).join('');
   if(bytes.byteLength!==asset.bytes||'sha256-'+btoa(hash)!==asset.integrity)failures.push(asset.path+': integrity');
  }
  return {name,assets:manifest.assets.length,keys:(await cache.keys()).length,failures,marker:await cache.match('https://dystrail.com/play/__offline_ready__').then(response=>response?.text()),cacheNames:await caches.keys()};
 },manifest);
 assert.deepEqual(report.cache.failures,[]);assert.equal(report.cache.marker,expected);
 assert.equal(report.updated.images,manifest.assets.filter(asset=>/\.(png|jpe?g|webp|gif|svg|ico)$/i.test(asset.path)).length);
 assert.equal(report.updated.decoded,true);
 await page.screenshot({path:out+'/output/playwright/production-updated-desktop.png',fullPage:true});
 console.log('Updated production cache and prior save verified',JSON.stringify({revision:expected,assets:report.cache.assets,images:report.updated.images,ruleChanges:report.ruleChanges.map(change=>change.key)}));
 await context.setOffline(true);
 await page.reload({waitUntil:'domcontentloaded',timeout:60000});
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main')?.getAttribute('data-screen')==='travel',expected,{timeout:90000});
 const offlineSave=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 for(const stateKey of ['state','pending'])offlineSave[stateKey].inventory.tags.sort();
 assert.deepEqual(ordered(offlineSave),ordered(saved));
 report.offline=await observe();
 assertFontReadiness(report.offline);
 await page.getByRole('tab',{name:'The Van',exact:true}).click();
 report.offlineCrewNames=[];
 for(const member of before.state.party.members){assert.equal(await page.getByText(member.name,{exact:true}).first().isVisible(),true,`Saved crew member is not visible: ${member.name}`);report.offlineCrewNames.push(member.name)}
 await page.screenshot({path:out+'/output/playwright/production-offline-van-desktop.png',fullPage:true});
 await page.setViewportSize({width:393,height:852});
 await page.screenshot({path:out+'/output/playwright/production-offline-van-phone.png',fullPage:true});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
 await writeFile(out+'/production-offline-autosave.json',JSON.stringify(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1'))),null,2));
 assert.deepEqual(errors,[]);
 report.passed=true;
 console.log('Production update, saved crew and offline browser verification passed');
}catch(error){report.failure=String(error.stack||error);console.error(report.failure);process.exitCode=1;try{await page.screenshot({path:out+'/output/playwright/production-update-failure.png',fullPage:true})}catch{}}
finally{await writeFile(out+'/production-browser-verification.json',JSON.stringify(report,null,2));await context.close();}
