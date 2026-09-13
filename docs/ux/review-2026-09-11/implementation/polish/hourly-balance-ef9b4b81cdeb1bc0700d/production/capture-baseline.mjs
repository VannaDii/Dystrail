import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {writeFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const out='/tmp/dystrail-balance-production-check';
const context=await chromium.launchPersistentContext(out+'/old-production-profile',{headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py',viewport:{width:1440,height:1000},serviceWorkers:'allow'});
const page=context.pages()[0]||await context.newPage(),errors=[];
page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
const report={priorRevision:'23cf83a1a7d5f9d02de9',url:'https://dystrail.com/play/',disposableProfile:true,errors};
try{
 await page.goto(report.url,{waitUntil:'domcontentloaded'});
 await page.waitForFunction(revision=>window.dystrailOffline?.revision===revision&&window.dystrailOffline?.state==='ready'&&!document.querySelector('#launch-gate')&&document.querySelector('#main')?.getAttribute('data-screen')==='travel',report.priorRevision,{timeout:90000});
 const save=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')));
 assert.equal(save.state.day,1);assert.equal(save.state.miles_traveled_actual,0);
 for(const key of ['state','pending']){assert.equal(save[key].pace_fatigue_remainder,0);assert.equal(Object.hasOwn(save[key],'distance_remainder'),false);assert.equal(Object.hasOwn(save[key],'day_start_remainder'),false)}
 await writeFile(out+'/old-production-autosave.json',JSON.stringify(save,null,2));
 report.save={phase:save.phase,day:save.state.day,miles:save.state.miles_traveled_actual,crew:save.state.party.members.length};
 assert.deepEqual(errors,[]);report.passed=true;console.log(JSON.stringify(report));
}finally{await writeFile(out+'/baseline-verification.json',JSON.stringify(report,null,2));await context.close()}
