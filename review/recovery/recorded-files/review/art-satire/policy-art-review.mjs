// Targeted rendered review after the source-art-only coin-return correction.
import {chromium} from './client-playwright.mjs';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const root='/private/tmp/dystrail-art-satire';
const out=`${root}/review/art-satire/policy-corrected-art`;
mkdirSync(out,{recursive:true});
const expected=createHash('sha256').update(readFileSync(`${root}/dystrail-web/static/img/scenes-v2/policy-bulletins-b.png`)).digest('hex');
const browser=await chromium.launch({headless:true});
const rows=[];const errors=[];
try {
 for(const [device,viewport] of [['desktop',{width:1280,height:900}],['phone',{width:393,height:851}]]){
  const page=await browser.newPage({viewport});page.on('pageerror',e=>errors.push(e.message));
  await page.goto('http://127.0.0.1:62531/play/');
  for(const language of ['en','ar']){
   await page.evaluate(language=>localStorage.setItem('dystrail.locale',language),language);await page.reload();
   await page.waitForFunction(()=>typeof window.dystrailLaunch?.then==='function');await page.evaluate(()=>window.dystrailLaunch);
   await page.locator('.policy-bulletin[data-unit="ORDER-GAG-B"]').waitFor();
   const path=await page.locator('.bulletin-illustration image').getAttribute('href');
   const bytes=await page.evaluate(async path=>Array.from(new Uint8Array(await (await fetch(path)).arrayBuffer())),path);
   assert.equal(createHash('sha256').update(Buffer.from(bytes)).digest('hex'),expected);
   await page.evaluate(async path=>{const image=new Image();image.src=path;await image.decode();},path);
   assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
   const title=await page.locator('#bulletin-title').textContent();
   assert.equal(title,JSON.parse(readFileSync(`${root}/dystrail-web/i18n/${language}.json`)).visual_copy['ORDER-GAG-B'].title);
   assert.equal(await page.locator('.policy-bulletin>.conditions-hud .policy-indicator strong').textContent(),title);
   await page.locator('.policy-bulletin>.conditions-hud .policy-indicator button').click();
   assert((await page.locator('.viewport-help').textContent()).includes(JSON.parse(readFileSync(`${root}/dystrail-web/i18n/${language}.json`)).visual_copy['ORDER-GAG-B'].activation));
   await page.keyboard.press('Escape');
   await page.screenshot({path:`${out}/${device}-${language}.png`,fullPage:true,animations:'disabled'});
   rows.push({device,language,sourceAndServedSha256:expected});
  }
  await page.context().setOffline(true);await page.reload();
  await page.waitForFunction(()=>typeof window.dystrailLaunch?.then==='function');await page.evaluate(()=>window.dystrailLaunch);
  await page.locator('.policy-bulletin[data-unit="ORDER-GAG-B"]').waitFor();
  await page.close();
 }
 assert.deepEqual(errors,[]);
 writeFileSync(`${out}/validation.json`,JSON.stringify({build:JSON.parse(readFileSync('/private/tmp/dystrail-art-satire-build/offline-manifest.json')).revision,rows,offlineReload:true,pageErrors:errors},null,2)+'\n');
} finally {await browser.close();}
