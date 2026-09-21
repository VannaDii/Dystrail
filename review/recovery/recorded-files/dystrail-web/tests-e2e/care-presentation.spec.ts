import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const texts:any=Object.fromEntries(['en','es','it','ar'].map(l=>[l,JSON.parse(readFileSync(`i18n/${l}.json`,'utf8'))]));
const state=(page:any)=>page.evaluate(()=>{const s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);s.state.inventory.tags.sort();return s;});
async function load(page:any,reason:number,variant:string,strain=1,player=false,locale='en'){
 const save=structuredClone(base);save.phase='CrewCare';save.aftermath=null;const s=save.state;
 s.day=5;s.clock_minutes=480;s.route_services.stop=null;s.stats.supplies=10;s.stats.sanity=10;s.stats.morale=7;s.breakdown=null;
 s.persona_id='journalist';const persona=player?'journalist':'organizer';s.party.members.find((m:any)=>m.persona===persona).name='River <3';
 s.crew_care={reason,pending:persona,strain:{[persona]:strain},last_check_day:5};
 const family=`CARE-${String(reason+1).padStart(2,'0')}`,unit=`${family}-${variant}`;
 s.visual_content={edition:1,selections:{[`${family}/care/${persona}/5`]:unit}};
 await page.evaluate(({save,locale}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(save));localStorage.setItem('dystrail.locale',locale);},{save,locale});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.crew-incident')).toBeVisible();return {before:await state(page),unit,persona,family};
}
async function capture(page:any,path:string){await page.evaluate(()=>{window.scrollTo({top:0,behavior:'instant'});if(document.activeElement instanceof HTMLElement)document.activeElement.blur();});await page.screenshot({path,fullPage:true,animations:'disabled'});}
test('care variants preserve the affected traveler and all distinct outcomes through reload',async({page},info)=>{
 test.setTimeout(180000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('./');await waitForLaunch(page);
 const cases:any[]=[[0,1,false,'Helped','helped'],[1,2,false,'Sheltered','sheltered'],[2,1,false,'Deferred','deferred'],[2,3,false,'CompanionLost','companion_lost'],[2,3,true,'PlayerLost','player_lost']];
 for(let reason=0;reason<8;reason++)for(const variant of ['A','B','C']){
  const [choice,strain,player,outcome,key]=cases[(reason*3+['A','B','C'].indexOf(variant))%cases.length];
  const {before,unit,persona,family}=await load(page,reason,variant,strain,player);
  const text=variant==='A'?texts.en.workshop[family]:texts.en.visual_copy[unit];
  await expect(page.locator('#screen-title')).toHaveText(`River <3 · ${text.title}`);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-care-persona',persona);
  await expect(page.locator('.scene-narrative')).toContainText((strain>1?text.continuing:text.setup).replaceAll('{name}','River <3'));
  if(player)await expect(page.locator('.crew-incident .action-button').nth(1)).toBeDisabled();
  if(strain===3)await expect(page.locator('.scene-narrative')).toContainText(text.critical.replaceAll('{name}','River <3'));
  await page.locator('.crew-incident .action-button').nth(choice).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
  await expect(page.locator('.outcome-copy')).toHaveText(text.outcomes[key].replaceAll('{name}','River <3'));
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-care-outcome',key);
  const after=await state(page);expect(after.aftermath.scene).toEqual({CareIncident:{unit,persona,strain,outcome}});
  expect(after.state.crew_care.pending).toBeNull();expect(after.state.clock_minutes-before.state.clock_minutes).toBe(60);expect(after.state.rng_bundle).toEqual(before.state.rng_bundle);expect(after.state.boss).toEqual(before.state.boss);
  const status=after.state.party.members.find((m:any)=>m.persona===persona).status;expect(status).toBe(outcome==='Sheltered'?'Departed':key.includes('lost')?'Dead':'Active');
  if(outcome==='Sheltered')await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.reload();await waitForLaunch(page);await expect(page.locator('.aftermath-panel')).toBeVisible();expect(await state(page)).toEqual(after);
  if(reason<2)await capture(page,info.outputPath(`care-${outcome}.png`));
 }
 expect(errors).toEqual([]);
});
test('translated care captions overlay the image and unavailable care stays disabled',async({page},info)=>{
 await page.goto('./');await waitForLaunch(page);
 for(const locale of ['en','es','it','ar']){
  const {family}=await load(page,0,'A',3,true,locale);
  await expect(page.locator('#screen-title')).toHaveText(`River <3 · ${texts[locale].workshop[family].title}`);
  await expect(page.locator('.scene-portrait')).toHaveCount(0);
  const art=await page.locator('.scene-art').boundingBox(),caption=await page.locator('.scene-caption').boundingBox();
  expect(caption!.y).toBeGreaterThanOrEqual(art!.y);expect(caption!.y+caption!.height).toBeCloseTo(art!.y+art!.height,0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await capture(page,info.outputPath(`care-offer-${locale}.png`));
 }
});
