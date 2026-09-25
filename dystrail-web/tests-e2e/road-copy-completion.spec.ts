import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const source=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units;
const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
for(const group of ['C','D','S']) test(`${group} remaining approved road variants render and resolve their own copy`,async({page,context},info)=>{
 test.setTimeout(360_000);
 const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;
 base.stats={...base.stats,supplies:10,hp:8,sanity:8,morale:8,credibility:8};
 base.budget_cents=10000;base.budget=100;base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 const units=source.filter((u:any)=>u.disposition==='compatible_narrative'&&u.id.startsWith(`ENC-${group}`)&&!(group==='C'&&Number(u.id.slice(5,7))<=8));
 for(const [index,u] of units.entries()) {
  const event=bank.find((e:any)=>e.id===u.runtime_key);const state=structuredClone(base);
  state.current_encounter=event;const family=u.id.slice(0,-2);const key=`${family}/road/300`;
  state.visual_content={edition:1,selections:{[key]:u.id},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  await expect(page.locator('#screen-title')).toHaveText(copy[u.id].name);
  await expect(page.locator('.encounter-panel')).toContainText(copy[u.id].desc);
  await expect(page.locator('.scene-atlas').first()).toBeVisible();
  const choice=index%event.choices.length;
  await expect(page.locator('.encounter-choice button').nth(choice)).toContainText(copy[u.id][`choice_${choice}`]);
  await page.locator('.encounter-choice button').nth(choice).click();
  await expect(page.locator('.outcome-copy')).toContainText(copy[u.id][`log_${choice}`]);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(saved.current_encounter).toBeNull();expect(saved.visual_content.outcomes[key]).toBe(choice);
  expect(saved.visual_content.selections[key]).toBe(u.id);expect(saved.journal.at(-1).title).toBe(copy[u.id].name);
  if(index===units.length-1){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.outcome-copy')).toContainText(copy[u.id][`log_${choice}`]);
   await page.evaluate(()=>window.scrollTo(0,0));
   await page.screenshot({path:info.outputPath(`${group}-outcome.png`),fullPage:true});
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   await context.setOffline(false);
  }
 }
});

test('reviewed road settings follow the current variant before and after its choice',async({page,context},info)=>{
 test.setTimeout(300000);const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;
 base.stats={...base.stats,supplies:10,hp:8,sanity:8,morale:8,credibility:8};base.budget_cents=10000;base.budget=100;base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 const groups:Record<string,string[]>={
  'enc-cafe':['C10-A','C12-A','C14-C','C16-A','D01-A','D01-B','D05-B','D05-C','D12-B','S08-A','S08-B','S16-A'],
  'enc-library':['C15-C','C17-C','D07-C','S02-B','S06-A','S29-A'],
  'enc-service-counter':['C11-B','D07-B','D10-A','S06-B','S11-C','S19-A','S25-A','S27-A'],
  'enc-service':['S01-B','S07-B','S33-C'],
 };
 for(const [setting,ids] of Object.entries(groups))for(const [index,id] of ids.entries()){
  const unit=`ENC-${id}`,u=source.find((s:any)=>s.id===unit);expect(u.disposition).toBe('compatible_narrative');
  const state=structuredClone(base),event=bank.find((e:any)=>e.id===u.runtime_key),key=`${unit.slice(0,-2)}/road/300`;state.current_encounter=event;
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
  const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(setting==='enc-service'?2:16/9,1);
  await expect(page.locator('.encounter-panel')).toContainText(copy[unit].desc);
  await page.locator('.encounter-choice button').first().click();await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_0);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);expect(saved.party).toEqual(state.party);expect(saved.visual_content.outcomes[key]).toBe(0);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);
  if(index===0){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`${setting}.png`),fullPage:true});await context.setOffline(false);}
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
});
