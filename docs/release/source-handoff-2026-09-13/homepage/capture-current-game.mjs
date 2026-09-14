import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const origin='http://127.0.0.1:49367/play/';
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
const records=[];
try {
 for(const deep of [false]) for(const width of [1280]) {
  const context=await browser.newContext({viewport:{width,height:852}});
  const page=await context.newPage();const errors=[];
  page.on('pageerror',e=>errors.push(String(e)));
  await page.goto(origin);
  if(deep)await page.getByRole('radio',{name:/The Deep End/}).check();
  await page.getByRole('button',{name:'Choose your character',exact:true}).click();
  await page.getByRole('radio',{name:'Journalist',exact:true}).click();
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await page.getByLabel('Your name',{exact:true}).fill('Tab Review');
  await page.getByLabel('Crew name',{exact:true}).fill('Review Crew');
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
  await page.getByRole('button',{name:'Start the journey',exact:true}).click();
  await page.locator('.journey-controls .travel-tabs').waitFor();
  await page.locator('#game-menu-button').click(); await page.getByRole('switch',{name:'Help & tips',exact:true}).click(); await page.locator('#game-menu-button').click(); await page.evaluate(async()=>{await document.fonts.ready;document.activeElement?.blur();scrollTo(0,0);});
  await page.locator('.world-view .scene-background').evaluateAll(imgs=>Promise.all(imgs.map(img=>img.decode())));
  await page.locator('#main').screenshot({path:'/Users/vanna/Source/Dystrail/site/assets/gameplay-current.jpg',type:'jpeg',quality:92,animations:'disabled'});
  const bounds=await page.locator('#main').boundingBox();writeFileSync('/tmp/dystrail-source-handoff-20260913/current-game-capture.json',JSON.stringify({bounds,revision:await page.evaluate(()=>window.dystrailOffline.revision),errors},null,2));assert.deepEqual(errors,[]);await context.close();
 }

}finally{await browser.close();}
