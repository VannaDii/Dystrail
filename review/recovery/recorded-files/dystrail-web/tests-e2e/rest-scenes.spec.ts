import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {baseline,importState,savedState,snap,waitForLaunch} from './helpers';

const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));
// Layout coordinates stay meaningful when activating a below-fold button scrolls
// the page. Viewport bounding-box Y does not describe the vehicle's road anchor.
const vanMetrics=(scene:any)=>scene.locator('.crew-van').evaluate((el:HTMLElement)=>({width:el.offsetWidth,height:el.offsetHeight,top:el.offsetTop,left:el.offsetLeft}));
function fixture(base:any,variant:string,supplies=8){
 const gs=structuredClone(base);
 gs.day=5;gs.clock_minutes=660;gs.stats.supplies=supplies;gs.stats.hp=6;gs.stats.sanity=4;
 gs.camp.rest_cooldown=0;gs.current_encounter=null;gs.crew_care.pending=null;gs.breakdown=null;
 gs.current_order=null;gs.exec_order_days_remaining=0;gs.exec_order_cooldown=100;
 gs.disease_cooldown=100;gs.day_state.day_initialized=true;
 gs.visual_content={edition:1,selections:{'ACT-REST/rest/5':`ACT-REST-${variant}`},policy_bulletins:[]};
 gs.party.members[1].status='Departed';
 return gs;
}
async function show(page:any,gs:any,lang='en'){
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 await importState(page,gs);await page.getByRole('button',{name:'Camp',exact:true}).click();
 if(lang!=='en'){await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);}
 await expect(page.locator('.camp-rest')).toBeVisible();
}
async function shot(page:any,name:string){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await snap(page,name);}

test('rest variants retain current crew, real road, localized copy and completed scene offline',async({page,context})=>{
 test.setTimeout(240000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 const base=await baseline(page);
 for(const variant of ['A','B','C'])for(const lang of ['en','es','it','ar']){
  const unit=`ACT-REST-${variant}`,gs=fixture(base,variant),locale=copy(lang),row='ABC'.indexOf(variant);
  await show(page,gs,lang);
  const scene=page.locator('.world-view .journey-scene');
  await expect(scene).toHaveAttribute('data-unit',unit);await expect(scene).toHaveAttribute('data-completed','false');
  await expect(scene).toHaveAttribute('data-hour','11');await expect(scene).toHaveAttribute('data-indoors','false');
  await expect(scene.locator('.rest-props')).toHaveAttribute('data-cell',String(row*2));
  await expect(scene.locator(`[data-member="${gs.party.members[1].persona}"]`)).toHaveCount(0);
  await expect(scene.locator(variant==='B'?'.van-occupant':'.standing-member')).toHaveCount(5);
  const vanBefore=await vanMetrics(scene);
  if(variant!=='B'){
   const table=await scene.locator('.rest-props').boundingBox();
   for(const member of await scene.locator('.standing-member').all()){
    const box=await member.boundingBox();expect(box!.x+box!.width).toBeLessThan(table!.x);
   }
  }else await expect(scene.locator('.rest-clock-hands')).toHaveAttribute('data-hour','11');
  await expect(page.locator('.scene-narrative')).toContainText(locale.visual_copy[unit].setup);
  await expect(page.locator('.camp-rest .action-button')).toContainText(locale.visual_copy[unit].action);
  if(lang==='en'){
   const source=JSON.parse(readFileSync('static/assets/data/visual-sources.json','utf8'))[unit].source;
   await page.locator('.scene-narrative .context-help button').click();await expect(page.locator(`a[href="${source}"]`)).toBeVisible();await page.keyboard.press('Escape');
  }
  if(lang==='en'||lang==='ar')await shot(page,`rest-${variant}-${lang}-offered`);
  const before=await current(page);await page.locator('.camp-rest .action-button').click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  await expect(scene).toHaveAttribute('data-unit',unit);await expect(scene).toHaveAttribute('data-completed','true');
  await expect(scene.locator('.rest-props')).toHaveAttribute('data-cell',String(row*2+1));
  await expect(scene.locator('.van-curtain')).toHaveCount(3);await expect(scene.locator('.crew-van')).toHaveAttribute('data-occupants','5');
  expect(await vanMetrics(scene)).toEqual(vanBefore);
  await expect(page.locator('.outcome-copy')).toHaveText(locale.visual_copy[unit].outcome);
  const after=await current(page);expect(after.day).toBe(before.day+1);expect(after.camp.rest_cooldown).toBe(2);
  expect(after.stats.sanity).toBeGreaterThan(before.stats.sanity);expect(after.budget_cents).toBe(before.budget_cents);
  expect(after.visual_content.selections['ACT-REST/rest/5']).toBe(unit);
  await expect(scene).toHaveAttribute('data-hour',String(Math.floor(after.clock_minutes/60)));
  if(variant==='B')await expect(scene.locator('.rest-clock-hands')).toHaveAttribute('data-hour',String(Math.floor(after.clock_minutes/60)));
  if(lang==='en'||lang==='ar')await shot(page,`rest-${variant}-${lang}-completed`);
  if(lang==='en'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(scene).toHaveAttribute('data-unit',unit);expect((await current(page)).stats).toEqual(after.stats);
   await context.setOffline(false);
   if(variant==='B'){
    const checkpoint=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
    mkdirSync('../review/art-satire/client-fixtures',{recursive:true});writeFileSync('../review/art-satire/client-fixtures/rest-completed.json',JSON.stringify(checkpoint));
   }
  }
  await page.locator('#outcome-continue').click();
  await page.getByRole('button',{name:locale.ux.camp,exact:true}).click();
  await expect(page.locator('.camp-rest .action-button')).toBeDisabled();
  expect((await current(page)).stats).toEqual(after.stats);
 }
 expect(errors).toEqual([]);
});

test('rest clock and atmosphere follow morning, dusk and night state',async({page})=>{
 const base=await baseline(page);
 for(const [minutes,weather,light] of [[390,'ColdSnap','dawn'],[1115,'Smoke','dusk'],[1330,'Storm','night']] as const){
  const gs=fixture(base,'B');gs.clock_minutes=minutes;gs.weather_state.today=weather;
  await show(page,gs);
  const scene=page.locator('.world-view .journey-scene');
  await expect(scene).toHaveAttribute('data-time',light);
  await expect(scene.locator('.rest-clock-hands')).toHaveAttribute('data-hour',String(Math.floor(minutes/60)));
  await expect(scene.locator('.rest-clock-hands')).toHaveAttribute('data-minute',String(minutes%60));
  await expect(scene).toHaveAttribute('data-weather',weather==='ColdSnap'?'cold':weather.toLowerCase());
  await shot(page,`rest-${light}-${weather}`);
 }
});

test('zero-supply rest, imported offers and rapid activation settle only once',async({page})=>{
 const base=await baseline(page),gs=fixture(base,'C',0);
 await show(page,gs);await page.locator('.camp-rest .action-button').dblclick();
 await expect(page.locator('.aftermath-panel')).toBeVisible();const after=await current(page);
 expect(after.day).toBe(6);expect(after.stats.supplies).toBe(0);expect(after.camp.rest_cooldown).toBe(2);
 await page.reload();await waitForLaunch(page);expect((await current(page)).stats).toEqual(after.stats);
 await show(page,await savedState(page));await expect(page.locator('.camp-rest .action-button')).toBeDisabled();
 const unselected=fixture(base,'A');unselected.visual_content.selections={};
 await show(page,unselected);const selected=await current(page);
 expect(selected.visual_content.selections['ACT-REST/rest/5']).toMatch(/^ACT-REST-[ABC]$/);
 expect(selected.rng_bundle).toEqual(unselected.rng_bundle);expect(selected.stats).toEqual(unselected.stats);
 await show(page,await savedState(page));expect((await current(page)).visual_content.selections['ACT-REST/rest/5']).toBe(selected.visual_content.selections['ACT-REST/rest/5']);
});
