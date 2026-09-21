import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {baseline,importState,savedState,snap,waitForLaunch} from './helpers';
import {atTown} from './geography';

const families=['ACT-FORAGE','ACT-GLEAN','ACT-FOODWORK','ACT-CASHWORK'];
const keys=['forage','glean','work_supplies','work_cash'];
const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));
function fixture(base:any,row:number,variant:string){
 const gs=structuredClone(base);
 gs.clock_minutes=600;gs.stats.supplies=8;gs.stats.sanity=8;gs.stats.hp=8;
 gs.day_state.day_initialized=true;gs.current_encounter=null;gs.crew_care.pending=null;
 gs.activities.foraged_on=null;gs.activities.worked_at=null;gs.current_order=null;
 gs.exec_order_days_remaining=0;gs.exec_order_cooldown=100;gs.disease_cooldown=100;
 gs.visual_content={edition:1,selections:{},policy_bulletins:[]};
 const family=families[row],unit=`${family}-${variant}`;
 if(row>1){atTown(gs,'La Crosse');gs.visual_content.selections[`${family}/work/${gs.route_services.route_id}/${gs.route_services.stop}`]=unit;}
 else {gs.route_services.stop=null;for(const region of ['PacificCoast','MountainWest','Southwest','Heartland','RustBelt','Beltway'])gs.visual_content.selections[`${family}/gather/${region}/0`]=unit;}
 // Art must not repopulate companions who have left the actual crew.
 gs.party.members[1].status='Departed';
 return gs;
}
async function showOffer(page:any,gs:any,row:number,lang:string){
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 await importState(page,gs);
 if(row<2)await page.getByRole('button',{name:'Camp',exact:true}).click();
 if(lang!=='en'){await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);}
}

test('activity choices and completed scenes retain the selected satire and actual costs',async({page,context})=>{
 test.setTimeout(300000);
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 const base=await baseline(page);
 for(const variant of ['A','B','C'])for(let row=0;row<4;row++)for(const lang of ['en','es','it','ar']){
  const unit=`${families[row]}-${variant}`,gs=fixture(base,row,variant),locale=copy(lang);
  await showOffer(page,gs,row,lang);
  const offer=page.locator(`.activity-offer[data-unit="${unit}"]`);
  await expect(offer).toBeVisible();
  await expect(offer.locator('h3')).toContainText(locale.visual_copy[unit].title);
  await expect(offer.locator('p')).toHaveText(locale.visual_copy[unit].setup);
  await expect(offer.locator('.journey-scene')).toHaveAttribute('data-completed','false');
  await expect(offer.locator('svg.scene-atlas')).toHaveAttribute('data-cell',String(row*2));
  await expect(offer.locator('.scene-speaker')).toHaveCount(0);
  const before=await current(page);
  if(lang==='en'||lang==='ar')await snap(page,`activity-${unit}-${lang}-offer`);
  await page.getByRole('button',{name:locale.trail[keys[row]],exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  const scene=page.locator('.world-view .scene-activity');
  await expect(scene).toHaveAttribute('data-unit',unit);
  await expect(scene).toHaveAttribute('data-completed','true');
  await expect(scene.locator('svg.scene-atlas')).toHaveAttribute('data-cell',String(row*2+1));
  await expect(scene).toHaveAttribute('data-indoors',String(row===2||(row===3&&variant!=='C')));
  await expect(page.locator('.outcome-copy')).toHaveText(locale.visual_copy[unit].outcome);
  const after=await current(page);
  expect(after.clock_minutes).toBe(before.clock_minutes+(row<2?120:180));
  expect(after.stats.supplies).toBe(before.stats.supplies+[2,4,4,0][row]);
  expect(after.stats.hp).toBe(before.stats.hp-(row===1?1:0));
  expect(after.stats.sanity).toBe(before.stats.sanity+[1,0,-1,-1][row]);
  expect(after.budget_cents).toBe(before.budget_cents+(row===3?1800:0));
  expect(after.rng_bundle).toEqual(before.rng_bundle);
  await expect(scene.locator(`[data-subject="${gs.party.members[1].persona}"]`)).toHaveCount(0);
  if(lang==='en'||lang==='ar')await snap(page,`activity-${unit}-${lang}-completed`);
  if(lang==='en'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(scene).toHaveAttribute('data-unit',unit);
   expect((await current(page)).stats).toEqual(after.stats);
   await context.setOffline(false);
   if(unit==='ACT-CASHWORK-C'){
    const checkpoint=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
    mkdirSync('../review/art-satire/client-fixtures',{recursive:true});
    writeFileSync('../review/art-satire/client-fixtures/activity-completed.json',JSON.stringify(checkpoint));
   }
  }
  await page.locator('#outcome-continue').click();
  for(const key of row<2?keys.slice(0,2):keys.slice(2))await expect(page.getByRole('button',{name:locale.trail[key],exact:true})).toBeDisabled();
 }
 expect(errors).toEqual([]);
});

test('manual save imports retain offers and work restrictions without a new roll',async({page})=>{
 const base=await baseline(page);
 for(let row=0;row<4;row++){
  const gs=fixture(base,row,'C'),unit=`${families[row]}-C`;
  await showOffer(page,gs,row,'en');
  await showOffer(page,await savedState(page),row,'en');
  await expect(page.locator(`.activity-offer[data-unit="${unit}"]`)).toBeVisible();
  await page.getByRole('button',{name:copy('en').trail[keys[row]],exact:true}).dblclick();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  const after=await savedState(page);await importState(page,after);
  if(row<2)await page.getByRole('button',{name:'Camp',exact:true}).click();
  expect((await current(page)).stats).toEqual(after.stats);
  for(const key of row<2?keys.slice(0,2):keys.slice(2))await expect(page.getByRole('button',{name:copy('en').trail[key],exact:true})).toBeDisabled();
 }
});
