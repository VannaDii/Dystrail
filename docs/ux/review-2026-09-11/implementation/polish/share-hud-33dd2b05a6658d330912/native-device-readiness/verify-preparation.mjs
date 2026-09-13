import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFile,writeFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const base=(await readFile('/tmp/dystrail-native-device-check/origin.txt','utf8')).trim();
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(String(e)));
try{
 await page.goto(base+'/prepare');await page.getByRole('button',{name:'Open test ending',exact:true}).click();await page.locator('#result-share-open').waitFor({timeout:20000});
 assert.equal(await page.evaluate(()=>window.dystrailOffline.revision),'33dd2b05a6658d330912');
 const before=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
 await page.locator('#result-share-open').click();await page.waitForFunction(()=>document.querySelector('.share-preview img')?.naturalWidth===1200);
 assert.equal(await page.locator('#share-post').isVisible(),true);await page.keyboard.press('Escape');
 assert.deepEqual(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state),before);assert.deepEqual(errors,[]);
 const report={passed:true,origin:base,revision:'33dd2b05a6658d330912',fixture:'Synthetic finished journey; isolated local origin',imageSize:1200,gameUnchanged:true,nativeSheet:'not tested by this headless preparation check'};await writeFile('/tmp/dystrail-native-device-check/preparation.json',JSON.stringify(report,null,2));console.log(JSON.stringify(report));
}catch(e){console.log(await page.evaluate(()=>JSON.stringify({url:location.href,offline:window.dystrailOffline,screen:document.querySelector('#main')?.dataset.screen,text:document.body.innerText.slice(0,3500),save:localStorage.getItem('dystrail.autosave.v1')?.slice(0,200)})));await page.screenshot({path:'/tmp/dystrail-native-device-check/preparation-failure.png'});throw e;}finally{await browser.close()}
