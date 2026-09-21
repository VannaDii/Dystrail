import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const texts=JSON.parse(readFileSync('i18n/en.json','utf8')).visual_copy;
const state=(page:any)=>page.evaluate(()=>{const s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);s.state.inventory.tags.sort();return s;});
function fixture(unit:string,allies=0){
 const save=structuredClone(base);save.phase='AllyLoss';save.aftermath=null;const s=save.state;
 s.clock_minutes=480;s.day=6;s.breakdown=null;s.route_services.stop=null;s.stats.allies=allies;s.crew_care.pending=null;
 const contact='River <3';const t=texts[unit];
 const entry={action_kind:'encounter',day:6,minute:480,pace:s.pace,diet:s.diet,place:'Seattle → Spokane',title:t.title,message:t.setup.replaceAll('{name}',contact)+(allies===0?' '+t.final_ally:''),before:{...s.stats,allies:allies+1},after:{...s.stats},details:[],resources:[]};
 s.journal=[entry];s.turn_journal_start=0;s.ally_notice=entry;s.visual_content={edition:1,selections:{[`${unit.slice(0,-2)}/ally/0`]:unit},ally_departures:[{journal_index:0,unit,contact}]};
 return save;
}
async function load(page:any,unit:string,allies=0,legacy=false,locale='en'){
 const save=fixture(unit,allies);if(legacy)delete save.state.visual_content.ally_departures;
 await page.evaluate(({save,locale}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(save));localStorage.setItem('dystrail.locale',locale);},{save,locale});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.ally-message')).toBeVisible();return await state(page);
}
test('external contacts keep their saved scenes, crew and outcome across reload and acknowledgement',async({page},info)=>{
 test.setTimeout(240000);await page.goto('./');await waitForLaunch(page);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 for(let family=1;family<=6;family++)for(const variant of ['A','B','C']){
  const unit=`ALLY-${String(family).padStart(2,'0')}-${variant}`,before=await load(page,unit,family%2);
  await expect(page.locator('.ally-composition')).toHaveAttribute('data-ally-unit',unit);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-unit',unit);
  await expect(page.locator('#screen-title')).toHaveText(texts[unit].title);
  await expect(page.locator('.ally-message')).toHaveText(before.state.ally_notice.message);
  await expect(page.locator('.scene-portrait,.parked-crew,.crew-van')).toHaveCount(0);
  expect(await page.locator('.ally-composition image').evaluate(async el=>{const image=new Image();image.src=el.getAttribute('href')!;await image.decode();return image.naturalWidth>0;})).toBe(true);
  const art=await page.locator('.scene-art').boundingBox(),caption=await page.locator('.scene-caption').boundingBox();
  expect(art!.width/art!.height).toBeCloseTo(1.5,2);expect(caption!.y).toBeGreaterThanOrEqual(art!.y);expect(caption!.y+caption!.height).toBeCloseTo(art!.y+art!.height,0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  const sign=texts[unit].sign;if(sign)await expect(page.locator('.ally-sign')).toHaveText(sign);else await expect(page.locator('.ally-sign')).toHaveCount(0);
  await page.locator('.scene-art').screenshot({path:info.outputPath(`${unit}-scene.png`),animations:'disabled'});
  await page.screenshot({path:info.outputPath(`${unit}.png`),fullPage:true,animations:'disabled'});
  await page.reload();await waitForLaunch(page);expect(await state(page)).toEqual(before);
  await page.locator('#outcome-continue').click();await expect(page.locator('.ally-message')).toHaveCount(0);
  const after=await state(page);expect(after.state.ally_notice).toBeNull();expect(after.state.party).toEqual(before.state.party);expect(after.state.stats).toEqual(before.state.stats);expect(after.state.rng_state).toEqual(before.state.rng_state);expect(after.state.boss).toEqual(before.state.boss);expect(after.state.visual_content).toEqual(before.state.visual_content);
 }
 expect(errors).toEqual([]);
});
test('legacy notices retain their recorded prose and do not acquire a fictional scene',async({page})=>{
 await page.goto('./');await waitForLaunch(page);const before=await load(page,'ALLY-02-A',1,true);
 await expect(page.locator('.ally-composition')).toHaveCount(0);await expect(page.locator('#screen-title')).toHaveText(before.state.ally_notice.title);
 await page.reload();await waitForLaunch(page);expect(await state(page)).toEqual(before);
});
test('departure captions and controls remain usable at enlarged text',async({page},info)=>{
 test.setTimeout(120000);await page.goto('./');await waitForLaunch(page);
 for(const unit of ['ALLY-01-A','ALLY-04-C','ALLY-06-A']){
  const before=await load(page,unit,0,false,'ar');await page.addStyleTag({content:'html {font-size:200% !important}'});
  const art=await page.locator('.scene-art').boundingBox(),caption=await page.locator('.scene-caption').boundingBox();
  expect(caption!.y).toBeGreaterThanOrEqual(art!.y);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:info.outputPath(`${unit}-enlarged-ar.png`),fullPage:true,animations:'disabled'});
  await page.locator('#outcome-continue').focus();await page.keyboard.press('Enter');await expect(page.locator('.ally-message')).toHaveCount(0);expect((await state(page)).state.party).toEqual(before.state.party);
 }
});
for(const unit of ['ALLY-01-A','ALLY-05-C'])writeFileSync(`../review/art-satire/client-fixtures/${unit.toLowerCase()}.json`,JSON.stringify(fixture(unit)));
