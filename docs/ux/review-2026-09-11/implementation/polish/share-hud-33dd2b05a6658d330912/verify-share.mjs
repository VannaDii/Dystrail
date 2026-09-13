import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const origin=readFileSync('/tmp/dystrail-share-hud-review/share-origin.txt','utf8').trim();
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
const records=[];
try {
 for(const deep of [false,true]) for(const width of [1440,393]) {
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
  state.abandoned=true;state.day=17;state.miles_traveled_actual=state.trail_distance*.38;
  state.party.members.find(m=>m.persona===state.persona_id).name=width===393?'Alexandria River Song With An Exceptionally Long Family Name':'River Safari';
  await page.locator('#game-menu-button').click();await page.locator('#save-open-btn').click();await page.locator('.save-text-backup>summary').click();await page.locator('#import-json').fill(JSON.stringify(state));await page.getByRole('button',{name:'Import',exact:true}).click();
  await page.locator('#result-share-open').waitFor();
  const before=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
  await page.evaluate(()=>{
   window.cardText=[];window.cardImages=[];
   const original=CanvasRenderingContext2D.prototype.fillText,draw=CanvasRenderingContext2D.prototype.drawImage;
   CanvasRenderingContext2D.prototype.fillText=function(text,x,y,...args){let m=this.measureText(text);window.cardText.push({text,x,y,font:this.font,color:this.fillStyle,left:x-m.actualBoundingBoxLeft,right:x+m.actualBoundingBoxRight,top:y-m.actualBoundingBoxAscent,bottom:y+m.actualBoundingBoxDescent});return original.call(this,text,x,y,...args);};
   CanvasRenderingContext2D.prototype.drawImage=function(...args){window.cardImages.push({source:args[0].src,coords:args.slice(1)});return draw.apply(this,args);};
  });
  await context.setOffline(true);
  await page.locator('#result-share-open').click();const img=page.locator('.share-preview img');await img.waitFor();
  const artifact=`/tmp/dystrail-share-hud-review/share-${deep?'deep':'classic'}-${width}`;
  const png=await img.evaluate(async e=>Array.from(new Uint8Array(await(await fetch(e.src)).arrayBuffer())));writeFileSync(artifact+'.png',Buffer.from(png));
  const data=await page.evaluate(()=>({text:window.cardText,images:window.cardImages}));writeFileSync(artifact+'.json',JSON.stringify(data,null,2));
  for(const t of data.text)assert(t.left>=48&&t.right<=1154&&t.top>=40&&t.bottom<1175,JSON.stringify(t));
  assert(data.images.some(i=>i.source.endsWith('occupant-journalist.png')&&i.coords[2]===284));
  assert.equal(await img.evaluate(e=>e.naturalWidth),1200);
  await page.getByLabel('Your post').fill('My journey, my words.');await page.keyboard.press('Escape');assert.equal(await page.locator('#result-share-open').evaluate(e=>e===document.activeElement),true);
  assert.deepEqual(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state),before);assert.deepEqual(errors,[]);
  records.push({mode:deep?'Deep':'Classic',width,offline:true,stateUnchanged:true,text:data.text});console.log('PASS share',deep,width);await context.close();
 }
 writeFileSync('/tmp/dystrail-share-hud-review/share-verification.json',JSON.stringify(records,null,2));
}finally{await browser.close();}
