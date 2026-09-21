import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {baseline,importState,savedState,snap,waitForLaunch} from './helpers';
import {atTown} from './geography';

const families=['ACT-BARTERTIRE','ACT-BARTERBATTERY','ACT-BARTERSUPPLIES'];
const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));
// Inventory tags are a serialized Rust HashSet; their order is not game state.
const inventory=(gs:any)=>({...gs.inventory,tags:[...gs.inventory.tags].sort()});
function fixture(base:any,variant:string){
 const gs=structuredClone(base);atTown(gs,'La Crosse');
 gs.prev_miles_traveled=gs.miles_traveled_actual;
 gs.clock_minutes=600;gs.stats.supplies=8;gs.stats.hp=8;gs.stats.sanity=8;
 gs.inventory.spares.tire=1;gs.inventory.spares.battery=0;
 gs.route_services.traded_at=null;gs.route_services.trading=true;
 gs.current_encounter=null;gs.crew_care.pending=null;gs.breakdown=null;
 gs.current_order=null;gs.exec_order_days_remaining=0;gs.exec_order_cooldown=100;
 gs.disease_cooldown=100;gs.day_state.day_initialized=true;
 gs.visual_content={edition:1,selections:{},policy_bulletins:[]};
 for(const family of families)gs.visual_content.selections[`${family}/exchange/${gs.route_services.route_id}/${gs.route_services.stop}`]=`${family}-${variant}`;
 gs.party.members[1].status='Departed';
 return gs;
}
async function show(page:any,gs:any,lang='en'){
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 await importState(page,gs);
 if(lang!=='en'){await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);}
 await expect(page.locator('.town-trading')).toBeVisible();
}
async function shot(page:any,name:string){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await snap(page,name);}

test('barter frames, localized offers and inventory changes agree across saved variants',async({page,context})=>{
 test.setTimeout(300000);
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 const base=await baseline(page);
 for(const variant of ['A','B','C'])for(let kind=0;kind<3;kind++)for(const lang of ['en','es','it','ar']){
  const unit=`${families[kind]}-${variant}`,gs=fixture(base,variant),locale=copy(lang);
  await show(page,gs,lang);
  const offer=page.locator(`.activity-offer[data-unit="${unit}"]`);
  await expect(offer.locator('h3')).toContainText(locale.visual_copy[unit].title);
  await expect(offer.locator('p')).toHaveText(locale.visual_copy[unit].setup);
  await expect(offer.locator('.journey-scene')).toHaveAttribute('data-completed','false');
  await expect(offer.locator('svg.scene-atlas')).toHaveAttribute('data-cell',String(kind*2));
  await expect(offer.locator('svg.scene-atlas')).toHaveAttribute('data-atlas',`barter-${variant.toLowerCase()}`);
  await expect(offer.locator('.scene-speaker')).toHaveCount(0);
  const before=await current(page);
  if(lang==='en'){
   const source=JSON.parse(readFileSync('static/assets/data/visual-sources.json','utf8'))[unit].source;
   await offer.locator('.context-help button').click();await expect(page.locator(`a[href="${source}"]`)).toBeVisible();await page.keyboard.press('Escape');
  }
  if(lang==='en'||lang==='ar')await shot(page,`barter-${unit}-${lang}-offer`);
  await page.getByRole('button',{name:locale.visual_copy[unit].action,exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  const scene=page.locator('.world-view .scene-activity');
  await expect(scene).toHaveAttribute('data-unit',unit);await expect(scene).toHaveAttribute('data-completed','true');
  await expect(scene).toHaveAttribute('data-indoors','false');
  await expect(scene.locator('svg.scene-atlas')).toHaveAttribute('data-cell',String(kind*2+1));
  await expect(page.locator('.outcome-copy')).toHaveText(locale.visual_copy[unit].outcome);
  const box=await scene.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(2.25,1);
  const after=await current(page);
  expect(after.stats.supplies).toBe(before.stats.supplies+[-3,-4,5][kind]);
  expect(after.inventory.spares.tire).toBe(before.inventory.spares.tire+[1,0,-1][kind]);
  expect(after.inventory.spares.battery).toBe(before.inventory.spares.battery+(kind===1?1:0));
  expect(after.clock_minutes).toBe(before.clock_minutes+30);
  expect(after.vehicle).toEqual(before.vehicle);expect(after.budget_cents).toBe(before.budget_cents);
  expect(after.rng_bundle).toEqual(before.rng_bundle);
  expect(after.route_services.traded_at).toBe(before.route_services.stop);
  await expect(scene.locator(`[data-subject="${gs.party.members[1].persona}"]`)).toHaveCount(0);
  if(lang==='en'||lang==='ar')await shot(page,`barter-${unit}-${lang}-completed`);
  if(lang==='en'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(scene).toHaveAttribute('data-unit',unit);expect(inventory(await current(page))).toEqual(inventory(after));
   await context.setOffline(false);
   if(unit==='ACT-BARTERBATTERY-C'){
    const checkpoint=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
    mkdirSync('../review/art-satire/client-fixtures',{recursive:true});
    writeFileSync('../review/art-satire/client-fixtures/barter-completed.json',JSON.stringify(checkpoint));
   }
  }
  await page.locator('#outcome-continue').click();
  await expect(page.locator('.town-trading .action-button:disabled')).toHaveCount(3);
  expect(inventory(await current(page))).toEqual(inventory(after));
 }
 expect(errors).toEqual([]);
});

test('unavailable trades, imported saves and rapid clicks cannot duplicate an exchange',async({page})=>{
 const base=await baseline(page);
 for(let kind=0;kind<3;kind++){
  const unit=`${families[kind]}-C`,gs=fixture(base,'C');
  gs.stats.supplies=kind===2?16:kind===1?3:2;
  await show(page,gs);
  await expect(page.locator(`.action-option:has([data-unit="${unit}"]) .action-button`)).toBeDisabled();
  await expect(page.locator(`.action-option:has([data-unit="${unit}"]) .trade-unavailable`)).toHaveText(kind===2?copy('en').trail.gather_capacity:copy('en').visual_trade.need_supplies);
  if(kind===2){gs.stats.supplies=8;gs.inventory.spares.tire=0;await show(page,gs);await expect(page.locator(`.action-option:has([data-unit="${unit}"]) .action-button`)).toBeDisabled();}
  await show(page,fixture(base,'C'));await show(page,await savedState(page));
  await page.getByRole('button',{name:copy('en').visual_copy[unit].action,exact:true}).dblclick();
  await expect(page.locator('.outcome-copy')).toHaveText(copy('en').visual_copy[unit].outcome);
  const after=await savedState(page);
  expect(after.stats.supplies).toBe(8+[-3,-4,5][kind]);
  await show(page,after);await expect(page.locator('.town-trading .action-button:disabled')).toHaveCount(3);
  expect(inventory(await current(page))).toEqual(inventory(after));
 }
});
