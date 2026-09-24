import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const en=JSON.parse(readFileSync('i18n/en.json','utf8'));
const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
function stateFor(base:any,reason:number,suffix:string,strain=1,player=false){
 const s=structuredClone(base);s.seed=42;s.day=5;s.clock_minutes=600;s.stats.supplies=10;s.stats.morale=7;s.stats.sanity=8;
 const actor=player?s.persona_id:s.party.members.find((m:any)=>m.persona!==s.persona_id&&m.status==='Active').persona;
 const family=`CARE-${String(reason+1).padStart(2,'0')}`,unit=`${family}-${suffix}`,key=`${family}/care/${actor}/5`;
 s.crew_care={reason,strain:{[actor]:strain},pending:actor,last_check_day:5};
 s.visual_content={edition:1,selections:{[key]:unit,...(strain>1?{[`care-active/${actor}`]:unit}:{})},outcomes:{},policy_bulletins:[]};
 return {s,actor,unit,key};
}
test('all care variants render and preserve treatment shelter and deferral mechanics',async({page,context})=>{
 test.setTimeout(240_000);const base=await baseline(page);
 for(let reason=0;reason<8;reason++)for(const suffix of ['A','B','C'])for(let choice=0;choice<3;choice++){
  const {s,actor,unit,key}=stateFor(base,reason,suffix);const name=s.party.members.find((m:any)=>m.persona===actor).name;
  await importState(page,s);await expect(page.locator('.scene-narrative')).toContainText(en.encounter_copy[unit].desc.replaceAll('{name}',name));
  await page.locator('.crew-incident .action-button').nth(choice).click();
  const after=await checkpoint(page);expect(after.crew_care.pending).toBeNull();expect(after.visual_content.outcomes[key]).toBe(choice);
  expect(after.clock_minutes).toBe(660);expect(after.stats.supplies).toBe(choice===0?8:10);
  expect(after.stats.morale).toBe(choice===0?8:choice===1?6:7);expect(after.stats.sanity).toBe(choice===2?7:8);
  expect(after.party.members.find((m:any)=>m.persona===actor).status).toBe(choice===1?'Departed':'Active');
  if(choice===0)await expect(page.locator('.outcome-copy')).toContainText(en.encounter_copy[unit].log_0.replaceAll('{name}',name));
  if(choice===2)expect(after.visual_content.selections[`care-active/${actor}`]).toBe(unit);
 }
 await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.outcome-copy')).toContainText(en.journey.care_deferred.split('{name}')[0]);await context.setOffline(false);
});

test('critical care preserves warnings and actual companion or player loss',async({page})=>{
 const base=await baseline(page);
 for(const player of [false,true]){
  const {s,actor,unit,key}=stateFor(base,0,'B',3,player);const name=s.party.members.find((m:any)=>m.persona===actor).name;
  await importState(page,s);await expect(page.locator('.scene-narrative')).toContainText(en.encounter_copy[unit].continuing.replaceAll('{name}',name));
  await expect(page.locator('.scene-narrative')).toContainText(en.journey.care_critical.replaceAll('{name}',name));
  if(player)await expect(page.locator('.crew-incident .action-button').nth(1)).toBeDisabled();
  await page.locator('.crew-incident .action-button').nth(2).click();
  const after=await checkpoint(page);expect(after.party.members.find((m:any)=>m.persona===actor).status).toBe('Dead');
  expect(after.visual_content.outcomes[key]).toBe(2);expect(after.stats.morale).toBe(4);expect(Boolean(after.ending)).toBe(player);
  await expect(page.locator('.outcome-copy')).toContainText(en.journey[player?'player_lost':'care_lost'].replaceAll('{name}',name));
 }
});

test('recovered care translations retain names and render offline',async({page,context},info)=>{
 test.setTimeout(120_000);const base=await baseline(page);
 for(const lang of ['es','it','ar']){
  const copy=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).encounter_copy;
  for(let reason=0;reason<8;reason++){
   // Import UI remains English; change the locale after importing the current state.
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
   const {s,actor,unit}=stateFor(base,reason,'C');const name=s.party.members.find((m:any)=>m.persona===actor).name;
   await importState(page,s);await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);
   await page.reload();await waitForLaunch(page);
   await expect(page.locator('.scene-narrative')).toContainText(copy[unit].desc.replaceAll('{name}',name));
   if(reason===7){
    await context.setOffline(true);await page.reload();await waitForLaunch(page);
    await page.evaluate(()=>window.scrollTo(0,0));await page.screenshot({path:info.outputPath(`care-${lang}.png`),fullPage:true});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await context.setOffline(false);
   }
  }
 }
});
