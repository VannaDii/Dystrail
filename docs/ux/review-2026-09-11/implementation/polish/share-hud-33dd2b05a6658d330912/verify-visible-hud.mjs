import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const origin=readFileSync('/tmp/dystrail-share-hud-review/verified-origin.txt','utf8').trim();
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
const records=[];
try {
 for(const deep of [true]) for(const [width,language] of [[320,'English'],[393,'Deutsch'],[393,'العربية']]) {
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
  const state=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
  state.day=129;state.clock_minutes=719;state.budget=9999;state.budget_cents=999900;state.current_order='WarDeptReorg';state.exec_order_days_remaining=2;state.weather_state.today='ColdSnap';
  await page.locator('#game-menu-button').click();await page.locator('#save-open-btn').click();await page.locator('.save-text-backup>summary').click();await page.locator('#import-json').fill(JSON.stringify(state));await page.getByRole('button',{name:'Import',exact:true}).click();
  if(language!=='English'){await page.locator('#game-menu-button').click();await page.locator('.language-picker>button').click();await page.getByRole('option',{name:language,exact:true}).click();await page.locator('#game-menu-button').click();}
  await page.evaluate(()=>{document.activeElement?.blur();scrollTo(0,0);});await page.waitForTimeout(200);
  const readout=await page.locator('.conditions-hud').evaluate(e=>{
   const rect=s=>{let n=e.querySelector(s),r=n.getBoundingClientRect();return {text:n.textContent,x:r.x,y:r.y,width:r.width,height:r.height}};
   return {clock:rect('.hud-context'),pace:rect('.hud-setting:nth-child(2)'),diet:rect('.hud-setting:nth-child(3)'),cash:rect('.leg-cash'),vehicle:rect('.leg-vehicle'),city:rect('.leg-destination'),weather:rect('.weather-indicator'),height:e.getBoundingClientRect().height};
  });
  writeFileSync(`/tmp/dystrail-share-hud-review/verified-hud-${width}-${language}.json`,JSON.stringify(readout,null,2));
  await page.locator('.world-view').screenshot({path:`/tmp/dystrail-share-hud-review/verified-hud-${width}-${language}.png`});
  for(const k of ['pace','diet','cash','vehicle'])assert(Math.abs(readout[k].y+readout[k].height/2-readout.clock.y-readout.clock.height/2)<2,JSON.stringify(readout));
  assert(readout.city.y>readout.clock.y+20);assert(readout.weather.y>readout.clock.y+20);
  for(const part of Object.values(readout).filter(v=>v&&typeof v==='object'))assert(part.x>=0&&part.x+part.width<=width&&part.width>16,JSON.stringify({width,part,readout}));
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);assert.deepEqual(errors,[]);
  records.push({mode:deep?'Deep':'Classic',width,readout,errors});console.log('PASS HUD',deep,width);await context.close();
 }
 writeFileSync('/tmp/dystrail-share-hud-review/verified-hud-verification.json',JSON.stringify(records,null,2));
}finally{await browser.close();}
