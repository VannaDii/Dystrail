import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
import {readFileSync} from 'node:fs';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
test.setTimeout(120_000);
function arrival(state:any,sanity=7){
 state.boss={ready:true,reached:true,attempted:false,victory:false,presentation:'Arrival',hearing:null};
 state.current_encounter=null;state.breakdown=null;state.ending=null;state.ally_notice=null;
 state.crew_care.pending=null;state.route_services.stop=null;state.stats.sanity=sanity;
 state.policy='balanced';state.rng_bundle=null;state.seed=4242;return state;
}
function committed(state:any,rounds:number[],initial=7,base=.55,draw:number|null=42,guarantee=false){
 arrival(state,initial);let sanity=initial;
 const records=rounds.map((influence,i)=>{const before=sanity;sanity=Math.max(0,sanity-2);return {influence,sanity_before:before,sanity_after:sanity,continuation_roll:!sanity||i===2?null:i<rounds.length-1?10:80};});
 const adjusted=sanity?guarantee?1:base*rounds.reduce((a,b)=>a+b,0)/rounds.length/100:null;
 const outcome=!sanity?'Exhausted':adjusted!>=1?'Secured':draw!<adjusted!*100?'Passed':'Failed';
 state.boss.attempted=true;state.boss.victory=['Passed','Secured'].includes(outcome);
 state.boss.hearing={rules_version:1,starting_stats:{...state.stats},day:state.day,minute:state.clock_minutes,base_chance:base,policy_guarantee:guarantee,rounds:records,adjusted_chance:adjusted,vote_roll:['Secured','Exhausted'].includes(outcome)?null:draw,outcome};
 state.boss.presentation={RoundRolling:0};state.stats.sanity=sanity;return state;
}

test('six hearing narratives preserve reports and distinguish an actual vote from secured success',async({page,context})=>{
 const base=await baseline(page);
 for(const mode of ['Classic','Deep']) for(const suffix of ['A','B','C']) {
  const family=mode==='Deep'?'HEARING-D':'HEARING-C',unit=`${family}-${suffix}`;
  const state=arrival(structuredClone(base));state.mode=mode;state.visual_content.edition=1;
  state.visual_content.selections[`${family}/hearing/0`]=unit;
  state.boss.presentation='Preparation';await importState(page,state);
  await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-unit',unit);
  await expect(page.locator('#hearing-title')).toHaveText(copy[unit].name);
  await expect(page.locator('.hearing-dialogue')).toHaveText(copy[unit].desc);
  const measured=await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth);expect(measured).toBe(true);
  for(const secured of [false,true]) {
   const s=committed(structuredClone(state),[120],7,secured?1:.55,42);s.boss.presentation='Closed';
   await importState(page,s);
   if(secured) await expect(page.locator('.hearing-dialogue')).not.toHaveText(copy[unit].before_vote);
   else await expect(page.locator('.hearing-dialogue')).toHaveText(copy[unit].before_vote);
   const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   expect(saved.boss.hearing).toEqual(s.boss.hearing);expect(saved.stats).toEqual(s.stats);
  }
  const exhausted=committed(structuredClone(state),[120],2);exhausted.boss.presentation='Verdict';await importState(page,exhausted);
  await expect(page.locator('.hearing-dialogue')).toHaveText(copy[unit].exhausted);
  if(unit==='HEARING-D-C') {
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.hearing-dialogue')).toHaveText(copy[unit].exhausted);
   expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.boss.hearing)).toEqual(exhausted.boss.hearing);
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:test.info().outputPath('hearing-exhausted-copy.png'),fullPage:true});await context.setOffline(false);
  }
 }
});
