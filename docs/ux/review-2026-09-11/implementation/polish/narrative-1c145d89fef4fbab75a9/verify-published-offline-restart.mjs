import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFile,writeFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const out='/tmp/dystrail-narrative-release-prep';
const expected=process.argv[2];
assert.match(expected||'',/^[a-f0-9]{20}$/,'Pass the final verified revision');
assert.notEqual(expected,'a9ff83cd924a07fdf4ec','Do not verify an unstaged baseline as a new release');
const saved=JSON.parse(await readFile(out+'/production-offline-autosave.json','utf8'));
const expectedSave=structuredClone(saved);
const manifest=JSON.parse(await readFile('/tmp/dystrail-narrative-release/release-site/play/offline-manifest.json','utf8'));
assert.equal(manifest.revision,expected);
const normalize=value=>{const copy=structuredClone(value);for(const key of ['state','pending'])copy[key].inventory.tags.sort();return copy};
const errors=[];
const report={revision:expected,network:'offline before initial navigation',profile:out+'/old-production-profile',errors};
const context=await chromium.launchPersistentContext(report.profile,{headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py',viewport:{width:393,height:852},serviceWorkers:'allow',offline:true});
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
page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
try{
 await page.goto('https://dystrail.com/play/',{waitUntil:'domcontentloaded',timeout:60000});
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main')?.getAttribute('data-screen')==='travel',expected,{timeout:90000});
 const restored=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 assert.deepEqual(normalize(restored),normalize(expectedSave));
 report.uiNavigation={before:saved.journey_detail,after:restored.journey_detail,reason:'The exact save captured after the deliberate Van selection survives a closed-browser restart.'};
 report.entireSavePreserved=true;
 report.cache=await page.evaluate(async manifest=>{
  const cache=await caches.open('dystopian-trail:/play/:'+manifest.revision),failures=[];
  for(const asset of manifest.assets){
   const response=await cache.match(new URL(asset.path,'https://dystrail.com/play/'));
   if(!response){failures.push(asset.path+': missing');continue;}
   const bytes=await response.arrayBuffer();
   const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),byte=>String.fromCharCode(byte)).join('');
   if(bytes.byteLength!==asset.bytes||'sha256-'+btoa(hash)!==asset.integrity)failures.push(asset.path+': integrity');
  }
  return {revision:window.dystrailOffline?.revision,assets:manifest.assets.length,keys:(await cache.keys()).length,failures,marker:await cache.match('https://dystrail.com/play/__offline_ready__').then(response=>response?.text()),images:Object.keys(window.dystrailAssetUrls||{}).length,decoded:(window.dystrailPreparedImages||[]).every(image=>image.complete&&image.naturalWidth>0),fonts:{prepared:(window.dystrailPreparedFonts||[]).map(face=>({family:face.family,style:face.style,weight:face.weight,status:face.status,registered:document.fonts.has(face)})),documentStatus:document.fonts.status,documentFaces:Array.from(document.fonts).map(face=>({family:face.family,status:face.status}))},atGateRemoval:window.releaseGateReadiness};
 },manifest);
 assert.deepEqual(report.cache.failures,[]);assert.equal(report.cache.marker,expected);assert.equal(report.cache.images,manifest.assets.filter(asset=>/\.(png|jpe?g|webp|gif|svg|ico)$/i.test(asset.path)).length);assert.equal(report.cache.decoded,true);
 assert.equal(report.cache.fonts.prepared.length,7);
 assert.equal(report.cache.fonts.prepared.every(face=>face.status==='loaded'&&face.registered),true);
 assert.equal(report.cache.fonts.documentStatus,'loaded');
 assert.equal(report.cache.fonts.documentFaces.length,7);
 assert.equal(report.cache.fonts.documentFaces.every(face=>face.status==='loaded'),true);
 assert.deepEqual(report.cache.atGateRemoval,{revision:expected,state:'ready',images:68,decoded:true,fonts:7,fontsLoaded:true,documentFontsStatus:'loaded',documentFontsLoaded:true});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
 await page.getByRole('tab',{name:'The Trail',exact:true}).click();
 await page.screenshot({path:out+'/output/playwright/production-restarted-offline-phone.png',fullPage:true});
 await page.screenshot({path:out+'/output/playwright/production-restarted-offline-phone-viewport.png',fullPage:false});
 await page.getByRole('tab',{name:'The Van',exact:true}).click();
 report.crew=[];
 for(const member of saved.state.party.members){assert.equal(await page.getByText(member.name,{exact:true}).first().isVisible(),true);report.crew.push(member.name)}
 assert.deepEqual(errors,[]);report.passed=true;
 console.log(JSON.stringify(report,null,2));
}catch(error){report.failure=String(error.stack||error);console.error(report.failure);process.exitCode=1;}
finally{await writeFile(out+'/production-offline-restart-verification.json',JSON.stringify(report,null,2));await context.close();}
