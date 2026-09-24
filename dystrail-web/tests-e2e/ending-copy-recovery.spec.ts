import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
const cases:any[]=[
 ...['hunger','vehicle','weather','breakdown','disease','crossing','panic'].map(cause=>[`END-COLLAPSE-${cause.toUpperCase()}`,{type:'collapse',cause}]),
 ['END-SANITY',{type:'sanity_loss'}],['END-DESTROYED',{type:'vehicle_failure',cause:'destroyed'}],
 ['END-COLD',{type:'exposure',kind:'cold'}],['END-HEAT',{type:'exposure',kind:'heat'}],
 ['END-VICTORY',{type:'boss_victory'}],['END-VOTEFAIL',{type:'boss_vote_failed'}],
];
test('all 45 ending narratives match the approved source',()=>{
 const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.filter((u:any)=>u.category==='Ending packages');
 expect(units).toHaveLength(45);
 for(const unit of units){expect(copy[unit.id].name).toBe(unit.title);expect(copy[unit.id].desc).toBe(unit.source_paragraphs[1]);}
});
test('recorded terminal causes show matching ending variants without changing results',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const [family,ending] of cases)for(const variant of ['A','B','C']){
  const unit=`${family}-${variant}`;const s=structuredClone(base);s.seed=42;s.ending=ending;
  if(['END-VICTORY','END-VOTEFAIL'].includes(family)){
   s.miles_traveled_actual=s.trail_distance;s.boss.outcome={attempted:true,victory:family==='END-VICTORY'};
  }
  s.visual_content={edition:1,selections:{[`${family}/ending/0`]:unit},outcomes:{},policy_bulletins:[],crossing_presentations:[]};
  await importState(page,s);await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
  await expect(page.locator('.result-headline')).toHaveText(copy[unit].name);
  await expect(page.locator('.result-epilogue')).toHaveText(copy[unit].desc);
  const before=await page.evaluate(()=>localStorage.getItem('dystrail.autosave.v1'));
  if(unit==='END-VOTEFAIL-C'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.result-epilogue')).toHaveText(copy[unit].desc);
   await page.screenshot({path:info.outputPath('ending-vote-failed.png'),fullPage:true});
   await context.setOffline(false);
   const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   const old=JSON.parse(before!).state;after.inventory.tags.sort();old.inventory.tags.sort();expect(after).toEqual(old);
  }
 }
});
