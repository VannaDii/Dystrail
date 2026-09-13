import { chromium } from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const out='/tmp/dystrail-narrative-release-prep';
const profile=out+'/old-production-profile';
const screenshots=out+'/output/playwright';
const expected='a9ff83cd924a07fdf4ec';
const frozen=JSON.parse(await readFile('/tmp/dystrail-visual-release/release-site/play/offline-manifest.json','utf8'));
assert.equal(frozen.revision,expected);
await mkdir(screenshots,{recursive:true});
const context=await chromium.launchPersistentContext(profile,{headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py',viewport:{width:1440,height:1000},serviceWorkers:'allow'});
const page=context.pages()[0]||await context.newPage();
const errors=[];
page.on('pageerror',e=>errors.push(e.message));
page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
const report={expected,profile,url:'https://dystrail.com/play/',productionHold:true,errors};
const ordered=value=>Array.isArray(value)?value.map(ordered):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,ordered(value[key])])):value;
const normalizeEnvelope=value=>{const copy=structuredClone(value);for(const key of ['state','pending'])copy[key].inventory.tags.sort();return ordered(copy)};
const digest=value=>{const copy=structuredClone(value);copy.inventory.tags.sort();return createHash('sha256').update(JSON.stringify(ordered(copy))).digest('hex')};
const observe=()=>page.evaluate(()=>({revision:window.dystrailOffline?.revision,readiness:window.dystrailOffline?.state,screen:document.querySelector('#main')?.getAttribute('data-screen'),gate:!!document.querySelector('#launch-gate'),prepared:Object.keys(window.dystrailAssetUrls||{}).length,decoded:(window.dystrailPreparedImages||[]).every(image=>image.complete&&image.naturalWidth>0),controller:navigator.serviceWorker.controller?.scriptURL,cacheNames:[] }));
try{
 const response=await page.goto(report.url,{waitUntil:'domcontentloaded',timeout:60000});
 assert.equal(response?.status(),200);
 console.log('Loading current production into the disposable profile');
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main'),expected,{timeout:240000});
 report.loaded=await observe();
 console.log('Old production cached and decoded',JSON.stringify(report.loaded));
 await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 await page.getByRole('radio',{name:'Journalist',exact:true}).click();
 await page.getByRole('button',{name:'Continue',exact:true}).click();
 await page.getByLabel('Your name',{exact:true}).fill('Release Check');
 await page.getByLabel('Crew name',{exact:true}).fill('The Cache Crew');
 await page.getByRole('button',{name:'Continue',exact:true}).click();
 await page.locator('.store-card').first().waitFor({state:'visible'});
 await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
 await page.getByRole('button',{name:'Start the journey',exact:true}).click();
 await page.waitForFunction(()=>document.querySelector('#main')?.getAttribute('data-screen')==='travel');
 await page.waitForFunction(()=>localStorage.getItem('dystrail.autosave.v1'));
 const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 assert.equal(saved.state.day,1);
 assert.equal(saved.state.persona_id,'journalist');
 report.save={stateSha256:digest(saved.state),phase:saved.phase,day:saved.state.day,miles:saved.state.miles_traveled_actual,crew:saved.state.party.members.map(m=>({name:m.name,persona:m.persona}))};
 await writeFile(out+'/old-production-autosave.json',JSON.stringify(saved,null,2));
 report.cache=await page.evaluate(async manifest=>{
  const name='dystopian-trail:/play/:'+manifest.revision;
  const cache=await caches.open(name),failures=[];
  for(const asset of manifest.assets){
   const response=await cache.match(new URL(asset.path,'https://dystrail.com/play/'));
   if(!response){failures.push(asset.path+': missing');continue;}
   const bytes=await response.arrayBuffer();
   const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>String.fromCharCode(b)).join('');
   if(bytes.byteLength!==asset.bytes||'sha256-'+btoa(hash)!==asset.integrity)failures.push(asset.path+': integrity');
  }
  return {name,assets:manifest.assets.length,keys:(await cache.keys()).length,failures,marker:await cache.match('https://dystrail.com/play/__offline_ready__').then(r=>r?.text())};
 },frozen);
 assert.deepEqual(report.cache.failures,[]);
 assert.equal(report.cache.marker,expected);
 assert.equal(report.loaded.prepared,frozen.assets.filter(a=>/\.(png|jpe?g|webp|gif|svg|ico)$/i.test(a.path)).length);
 assert.equal(report.loaded.decoded,true);
 await page.screenshot({path:screenshots+'/old-production-saved-desktop.png',fullPage:true});
 await context.setOffline(true);
 await page.reload({waitUntil:'domcontentloaded',timeout:60000});
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main')?.getAttribute('data-screen')==='travel',expected,{timeout:90000});
 report.offline=await observe();
 const restored=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 report.offlineStateSha256=digest(restored.state);
 assert.equal(report.offlineStateSha256,report.save.stateSha256);
 assert.deepEqual(normalizeEnvelope(restored),normalizeEnvelope(saved));
 report.completeEnvelopePreserved=true;
 await writeFile(out+'/old-production-offline-autosave.json',JSON.stringify(restored,null,2));
 await page.setViewportSize({width:393,height:852});
 await page.screenshot({path:screenshots+'/old-production-offline-phone.png',fullPage:true});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
 assert.deepEqual(errors,[]);
 report.passed=true;
 report.unorderedFields=['state.inventory.tags (Rust HashSet)','pending.inventory.tags (Rust HashSet)'];
 console.log(JSON.stringify({passed:report.passed,revision:expected,assets:report.cache.assets,cacheKeys:report.cache.keys,images:report.loaded.prepared,save:report.save,offlineStateSha256:report.offlineStateSha256,profile},null,2));
}catch(error){report.failure=String(error.stack||error);console.error(report.failure);process.exitCode=1;try{await page.screenshot({path:screenshots+'/old-production-failure.png',fullPage:true})}catch{}}
finally{await writeFile(out+'/old-production-browser.json',JSON.stringify(report,null,2));await context.close();}
