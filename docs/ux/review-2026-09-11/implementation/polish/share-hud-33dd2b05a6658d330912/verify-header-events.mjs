import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const origin=readFileSync('/tmp/dystrail-share-hud-review/verified-origin.txt','utf8').trim();
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
  const before=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
  await page.getByRole('tab',{name:'Journal',exact:true}).click();
  const details=page.locator('.journal-raw>summary');assert.match(await details.first().textContent(),/The Day's Events/);await details.first().click();
  await page.locator('.journal-day').first().screenshot({path:`/tmp/dystrail-share-hud-review/day-events-${deep?'deep':'classic'}-${width}.png`});
  await page.getByRole('tab',{name:'The Van',exact:true}).click();
  await page.evaluate(()=>{document.activeElement?.blur();scrollTo(0,document.body.scrollHeight);});
  await page.waitForTimeout(200);
  const header=await page.locator('.game-header').boundingBox();assert(Math.abs(header.y)<1,JSON.stringify(header));
  const menu=page.locator('#game-menu-button');await menu.click();
  assert.equal(await menu.getAttribute('aria-expanded'),'true');
  const body=await page.locator('.top-game-menu-panel').boundingBox();assert(body.y>=40);assert(body.y+body.height<=852);
  await page.locator('.game-header').screenshot({path:`/tmp/dystrail-share-hud-review/verified-header-${deep?'deep':'classic'}-${width}.png`});
  await page.locator('#save-open-btn').click();await page.locator('.save-manager .save-manager-title').count();
  assert(await page.locator('.save-manager').isVisible());
  const front=await page.locator('.save-manager .drawer-body').evaluate(e=>{const r=e.getBoundingClientRect();return e.contains(document.elementFromPoint(r.x+r.width/2,r.y+20));});assert(front);
  await page.keyboard.press('Escape');
  assert.deepEqual(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state),before);
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);assert.deepEqual(errors,[]);
  records.push({mode:deep?'Deep':'Classic',width,header,menu:body,stateUnchanged:true,errors});console.log('PASS header',deep,width);await context.close();
 }
 writeFileSync('/tmp/dystrail-share-hud-review/verified-header-verification.json',JSON.stringify(records,null,2));
}finally{await browser.close();}
