import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const origin=readFileSync('/tmp/dystrail-source-handoff-20260913/hud-origin.txt','utf8').trim();
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
const records=[];
try {
 for(const deep of [false,true]) for(const width of [1440,1024,850,801,800,393,320]) {
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
  const readout=await page.locator('.conditions-hud').evaluate(e=>{
   const rect=s=>{let n=e.querySelector(s),r=n.getBoundingClientRect();return {text:n.textContent,x:r.x,y:r.y,width:r.width,height:r.height}};
   return {clock:rect('.hud-context'),pace:rect('.hud-setting:nth-child(2)'),diet:rect('.hud-setting:nth-child(3)'),cash:rect('.leg-cash'),vehicle:rect('.leg-vehicle'),city:rect('.leg-destination'),weather:rect('.weather-indicator'),height:e.getBoundingClientRect().height};
  });
  writeFileSync(`/tmp/dystrail-source-handoff-20260913/hud-${deep?'deep':'classic'}-${width}.json`,JSON.stringify(readout,null,2));
  await page.locator('.world-view').screenshot({path:`/tmp/dystrail-source-handoff-20260913/hud-${deep?'deep':'classic'}-${width}.png`});
  for(const k of ['pace','diet','cash','vehicle'])assert(Math.abs(readout[k].y-readout.clock.y)<2,JSON.stringify(readout));
  if(width>800) { const parts=['clock','pace','diet','cash','vehicle','city'].map(k=>readout[k]);for(let i=1;i<parts.length;i++) {assert(Math.abs(parts[i].x-parts[i-1].x-parts[i-1].width-18)<1,JSON.stringify(readout)); assert(Math.abs(parts[i].y-parts[0].y)<2);}} else {assert(readout.city.y>readout.clock.y+20);assert(Math.abs((readout.city.y+readout.city.height/2)-(readout.weather.y+readout.weather.height/2))<2);}
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);assert.deepEqual(errors,[]);
  records.push({mode:deep?'Deep':'Classic',width,readout,errors});console.log('PASS HUD',deep,width);await context.close();
 }
 writeFileSync('/tmp/dystrail-source-handoff-20260913/hud-verification.json',JSON.stringify(records,null,2));
}finally{await browser.close();}
