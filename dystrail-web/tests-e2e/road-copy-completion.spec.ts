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
  'enc-cafe':['C10-A','C12-A','C14-C','C16-A','D01-A','D01-B','D05-B','D05-C','D12-B','S08-A','S08-B','S16-A','C13-C','D03-C','S05-C','C12-B','D11-B'],
  'enc-library':['C15-C','C17-C','D07-C','S02-B','S06-A','S29-A'],
  'enc-service-counter':['C11-B','D07-B','D10-A','S06-B','S11-C','S19-A','S25-A','S27-A','D06-A','D09-B','D11-C','S21-A','S21-C','S34-A'],
  'enc-service':['S01-B','S07-B','S33-C','D13-C'],
  'enc-rest-area':['D03-A','D06-C','D07-A','D09-C','D11-A'],
  'enc-community':['C15-A','C15-B','C17-A','C17-B','S02-A','S02-C','S15-C','S20-A','S20-C'],
  'enc-civic':['C18-A','C18-C'],
  'enc-museum':['S15-B','S23-C'],
 };
 for(const [setting,ids] of Object.entries(groups))for(const [index,id] of ids.entries()){
  if(process.env.COUNTER_MASK_ONLY==='1'&&(setting!=='enc-service-counter'||index!==0))continue;
  if(process.env.ROAD_COUNTER_ONLY==='1'&&!['enc-cafe','enc-service-counter'].includes(setting))continue;
  if(process.env.ROAD_MUSEUM_ONLY==='1'&&setting!=='enc-museum')continue;
  const unit=`ENC-${id}`,u=source.find((s:any)=>s.id===unit);expect(u.disposition).toBe('compatible_narrative');
  const state=structuredClone(base),event=bank.find((e:any)=>e.id===u.runtime_key),key=`${unit.slice(0,-2)}/road/300`;state.current_encounter=event;
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',['enc-rest-area','enc-community'].includes(setting)?'false':'true');
  if(setting==='enc-service-counter')await expect(page.locator('[data-blank-counter-papers] path')).toHaveCount(4);
  if(setting==='enc-museum')await expect(page.locator('[data-blank-exhibit-labels] path')).toHaveCount(10);
  const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(setting==='enc-rest-area'?2.25:['enc-service','enc-community','enc-civic'].includes(setting)?2:16/9,1);
  await expect(page.locator('.encounter-panel')).toContainText(copy[unit].desc);
  await page.locator('.encounter-choice button').first().click();await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_0);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);expect(saved.party).toEqual(state.party);expect(saved.visual_content.outcomes[key]).toBe(0);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);
  if(index===0){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`${setting}.png`),fullPage:true});await context.setOffline(false);}
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
});


test('all active road variants expose their own source references without changing the run',async({page,context},info)=>{
 test.setTimeout(360000);const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;
 base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 const references=JSON.parse(readFileSync('static/assets/data/road-source-references.json','utf8'));
 const units=source.filter((u:any)=>u.category==='Road encounters'&&u.disposition==='compatible_narrative');expect(Object.keys(references)).toHaveLength(195);
 for(const [index,u] of units.entries()){
  const state=structuredClone(base);state.current_encounter=bank.find((e:any)=>e.id===u.runtime_key);
  const key=`${u.id.slice(0,-2)}/road/300`;state.visual_content={edition:1,selections:{[key]:u.id},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  const before=await page.evaluate(()=>localStorage.getItem('dystrail.autosave.v1'));
  const check=async()=>{
   await page.locator('.encounter-panel .help-trigger').click();
   await expect(page.locator('.viewport-help [data-source-unit]')).toHaveAttribute('data-source-unit',u.id);
   const expected=u.source_paragraphs.filter((p:string)=>p.startsWith('Source:')).map((p:string)=>p.slice(7).trim());
   await expect(page.locator('.viewport-help .source-reference')).toHaveText(expected);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   await page.keyboard.press('Escape');
  };
  await check();expect(await page.evaluate(()=>localStorage.getItem('dystrail.autosave.v1'))).toBe(before);
  if(index===units.length-1){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);await check();
   await page.locator('.encounter-panel .help-trigger').click();await page.screenshot({path:info.outputPath('road-source.png')});await page.keyboard.press('Escape');await context.setOffline(false);
  }
 }
 const inactive=source.filter((u:any)=>u.disposition==='mechanics_dependency_inactive');expect(inactive).toHaveLength(36);
 for(const u of inactive)expect(references[u.id]).toBeUndefined();
});


test('fallback square interiors retain landscape framing through outcomes',async({page,context},info)=>{
 const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;
 base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 for(const id of ['ENC-S06-C','ENC-D03-B','ENC-S18-A','ENC-D01-C','ENC-C12-B']){
  const u=source.find((s:any)=>s.id===id),state=structuredClone(base),key=`${id.slice(0,-2)}/road/300`;
  state.current_encounter=bank.find((e:any)=>e.id===u.runtime_key);
  state.visual_content={edition:1,selections:{[key]:id},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  const check=async()=>{
   if(id==='ENC-S06-C')await expect(page.locator('[data-blank-farm-paper] path')).toHaveCount(1);
   const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(16/9,1);
   const vb=(await page.locator('.scene-atlas').first().getAttribute('viewBox'))!.split(' ').map(Number);
   expect(vb[2]/vb[3]).toBeCloseTo(16/9,5);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  };
  await check();await page.locator('.encounter-choice button').first().click();await check();
  await expect(page.locator('.outcome-copy')).toContainText(copy[id].log_0);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(saved.party).toEqual(state.party);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);await check();
  await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
  await page.screenshot({path:info.outputPath(`${id}.png`)});await context.setOffline(false);
 }
});
