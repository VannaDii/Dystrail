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
  'enc-checkpoint':['D13-B'],
  'enc-convoy':['S04-A','S17-C'],
  'enc-motel':['S27-B','S10-C'],
  'enc-campground':['S04-B','S07-A'],
  'enc-civic-exterior':['D02-A','D12-A'],
  'enc-cafe':['C10-A','C12-A','C14-C','C16-A','D01-A','D01-B','D05-B','D05-C','D12-B','S08-A','S08-B','S16-A','C13-C','D03-C','S05-C','C12-B','D11-B','C11-C','D01-C','D04-C','C13-B','S31-C'],
  'enc-library':['C15-C','C17-C','D07-C','S02-B','S06-A','S29-A','S11-B','S18-A','S18-C'],
  'enc-service-counter':['S03-A','S03-C','S05-B','S14-C','S23-B','C11-B','D07-B','D10-A','S06-B','S11-C','S19-A','S25-A','S27-A','D06-A','D09-B','D11-C','S21-A','S21-C','S34-A','C09-B','C13-A','D03-B','D04-A','D04-B','D05-A','D08-C','D10-C','S21-B','S22-A','S22-C','S25-B','S25-C','S27-C','S28-B','S29-B','S29-C','S31-A','S31-B','S32-C'],
  'enc-service':['S01-B','S07-B','S33-C','D13-C'],
  'enc-rest-area':['C09-A','S01-A','D03-A','D06-C','D07-A','D09-C','D11-A'],
  'enc-community':['S09-B','S15-A','S26-C','S28-C','S30-B','C15-A','C15-B','C17-A','C17-B','S02-A','S02-C','S15-C','S20-A','S20-C'],
  'enc-civic':['C18-A','C18-C'],
  'enc-museum':['S15-B','S23-C'],
 };
 for(const [setting,ids] of Object.entries(groups))for(const [index,id] of ids.entries()){
  if(process.env.ROAD_REVIEW_UNITS&&!process.env.ROAD_REVIEW_UNITS.split(',').includes(id))continue;
  if(process.env.ROAD_MOTEL_ONLY==='1'&&setting!=='enc-motel')continue;
  if(process.env.COUNTER_MASK_ONLY==='1'&&(setting!=='enc-service-counter'||index!==0))continue;
  if(process.env.ROAD_COUNTER_ONLY==='1'&&!['enc-cafe','enc-service-counter'].includes(setting))continue;
  if(process.env.ROAD_MUSEUM_ONLY==='1'&&setting!=='enc-museum')continue;
  const unit=`ENC-${id}`,u=source.find((s:any)=>s.id===unit);expect(u.disposition).toBe('compatible_narrative');
  const state=structuredClone(base),event=bank.find((e:any)=>e.id===u.runtime_key),key=`${unit.slice(0,-2)}/road/300`;state.current_encounter=event;
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',['enc-rest-area','enc-community','enc-campground','enc-civic-exterior','enc-convoy','enc-checkpoint'].includes(setting)?'false':'true');
  if(setting==='enc-service-counter')await expect(page.locator('[data-blank-counter-papers] path')).toHaveCount(4);
  if(setting==='enc-museum')await expect(page.locator('[data-blank-exhibit-labels] path')).toHaveCount(10);
  const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(['enc-rest-area','enc-campground','enc-civic-exterior'].includes(setting)?2.25:['enc-service','enc-community','enc-civic','enc-convoy','enc-checkpoint'].includes(setting)?2:16/9,1);
  await expect(page.locator('.encounter-panel')).toContainText(copy[unit].desc);
  await page.locator('.encounter-choice button').first().click();await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_0);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);expect(saved.party).toEqual(state.party);expect(saved.visual_content.outcomes[key]).toBe(0);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);
  if(index===0||process.env.ROAD_REVIEW_UNITS?.split(',').find(u=>ids.includes(u))===id){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`${setting}.png`),fullPage:true});await context.setOffline(false);}
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


test('retained selected satire panels preserve choices and crew',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;base.driving_minutes_total=300;base.last_encounter_driving_minutes=300;
 const panels:any={'ENC-C09-C':['01',0],'ENC-C10-B':['01',1],'ENC-C10-C':['01',2],'ENC-C12-C':['02',0],'ENC-C14-A':['02',1],'ENC-C16-B':['02',2],'ENC-D12-C':['04',1],'ENC-D13-A':['04',2],'ENC-S11-A':['06',0],'ENC-S13-A':['06',1],'ENC-S14-A':['06',2],'ENC-S16-B':['06',3],'ENC-S16-C':['07',0],'ENC-S17-A':['07',1],'ENC-S19-B':['08',0],'ENC-S20-B':['08',1],'ENC-S22-B':['08',3]};
 for(const [unit,[sheet,cell]] of Object.entries(panels) as any){
  if(process.env.SELECTED_PLANE_ONLY==='1'&&unit!=='ENC-C10-C')continue;
  if(process.env.SELECTED_SHEET6_ONLY==='1'&&sheet!=='06')continue;
  if(process.env.SELECTED_SHEET7_ONLY==='1'&&sheet!=='07')continue;
  if(process.env.SELECTED_SHEET8_ONLY==='1'&&sheet!=='08')continue;
  const u=source.find((s:any)=>s.id===unit),s=structuredClone(base),key=`${unit.slice(0,-2)}/road/300`;s.current_encounter=bank.find((e:any)=>e.id===u.runtime_key);
  s.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,s);
  const art=page.locator(`[data-selected-unit="${unit}"]`);await expect(art).toHaveAttribute('data-atlas',`selected-satire-${sheet}`);await expect(art).toHaveAttribute('data-cell',String(cell));
  if(sheet==='06')await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',unit==='ENC-S11-A'?'false':'true');
  if(sheet==='07')await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',unit==='ENC-S16-C'?'false':'true');
  if(sheet==='08')await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
  const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(1.5,1);
  const overlays=Object.entries(copy[unit]).filter(([key])=>key.startsWith('overlay_')).map(([,value])=>value);
  await expect(art.locator('.selected-prop-label')).toHaveText(overlays);
  await page.evaluate(()=>scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`${unit}.png`),fullPage:true});
  await page.locator('.encounter-choice button').first().click();await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_0);
  await expect(art).toBeVisible();const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);expect(saved.party).toEqual(s.party);expect(saved.visual_content.outcomes[key]).toBe(0);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(art).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await context.setOffline(false);
 }
});

test('selected satire labels fit their surfaces in every supported language',async({page,context},info)=>{
 test.setTimeout(600000);const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;base.driving_minutes_total=300;base.last_encounter_driving_minutes=300;
 const languages=[...readFileSync('src/i18n/locales.rs','utf8').matchAll(/code: "([a-z]+)"/g)].map(m=>m[1]);
 for(const unit of ['ENC-C09-C','ENC-C10-B','ENC-C10-C','ENC-C12-C','ENC-C14-A','ENC-C16-B','ENC-D12-C','ENC-D13-A']){
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
  const u=source.find((s:any)=>s.id===unit),s=structuredClone(base),key=`${unit.slice(0,-2)}/road/300`;s.current_encounter=bank.find((e:any)=>e.id===u.runtime_key);s.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,s);
  for(const lang of languages){
   if(process.env.LABEL_OVERFLOW_ONLY==='1'&&!((unit==='ENC-C09-C'&&lang==='ru')||(unit==='ENC-C14-A'&&lang==='tr')))continue;
   const translations=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).encounter_copy[unit];
   const labels=Object.keys(copy[unit]).filter(k=>k.startsWith('overlay_')).map(k=>translations[k]);expect(labels.every(Boolean)).toBe(true);
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);await context.setOffline(true);await page.reload();await waitForLaunch(page);
   const rendered=page.locator(`[data-selected-unit="${unit}"] .selected-prop-label`);await expect(rendered).toHaveText(labels);
   const overflow=await rendered.evaluateAll(nodes=>nodes.filter(n=>n.scrollHeight>n.clientHeight+1||n.scrollWidth>n.clientWidth+1).map(n=>n.textContent));expect.soft(overflow,`${lang} ${unit}`).toEqual([]);
   if((unit==='ENC-D13-A'&&['ar','de'].includes(lang))||process.env.LABEL_OVERFLOW_ONLY==='1'){await page.evaluate(()=>scrollTo({top:0,behavior:'instant'}));await page.locator('.journey-scene').screenshot({path:info.outputPath(`labels-${lang}.png`)});}
   await context.setOffline(false);
  }
 }
});


test('retained room notices stay blank through an encounter and offline outcome',async({page,context},info)=>{
 const base=await baseline(page);base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;
 base.stats={...base.stats,supplies:10,hp:8,sanity:8,morale:8,credibility:8};base.budget_cents=10000;base.budget=100;
 for(const [runtime,setting,mask,count] of [
  ['deep_secure_line','enc-motel','motel',2],
  ['classic_media_training','enc-media-workshop','media',3],
  ['classic_service_station','enc-service','service',1],
  ['classic_radio_phonebank','enc-radio','radio',3],
 ] as const){
  const state=structuredClone(base);state.current_encounter=structuredClone(bank.find((e:any)=>e.id===runtime));
  // Custom copy keeps this a direct retained-setting fixture instead of selecting newer variant art.
  state.current_encounter.name='Retained setting preview';
  state.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);
  await expect(page.locator(`[data-blank-setting-papers="${mask}"] path`)).toHaveCount(count);
  await page.locator('.journey-scene').screenshot({path:info.outputPath(`${mask}.png`)});
  await page.locator('.encounter-choice button').first().click();
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(saved.current_encounter).toBeNull();expect(saved.party).toEqual(state.party);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',setting);
  await expect(page.locator(`[data-blank-setting-papers="${mask}"] path`)).toHaveCount(count);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await context.setOffline(false);
 }
});

test('French pantry and public demonstration copy survives outcomes and offline reload',async({page,context},info)=>{
 test.setTimeout(120000);const base=await baseline(page),fr=JSON.parse(readFileSync('i18n/fr.json','utf8')).encounter_copy;
 base.seed=42;base.day=6;base.clock_minutes=600;base.turn_journal_start=null;base.driving_minutes_total=300;
 base.stats={...base.stats,supplies:10,hp:8,sanity:8,morale:8,credibility:8};
 for(const family of ['C02','C04'])for(const variant of ['A','B','C']){
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
  const unit=`ENC-${family}-${variant}`,u=source.find((s:any)=>s.id===unit),state=structuredClone(base),key=`ENC-${family}/road/300`;
  state.current_encounter=bank.find((e:any)=>e.id===u.runtime_key);
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};await importState(page,state);
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','fr'));await page.reload();await waitForLaunch(page);
  await expect(page.locator('#screen-title')).toContainText(fr[unit].name);
  for(const label of ['Provisions','Santé','Équilibre mental','Crédibilité','Alliés'])await expect(page.getByRole('button',{name:`Comment ça fonctionne: ${label}`,exact:true})).toBeVisible();
  await expect(page.getByRole('button',{name:'Campement',exact:true})).toHaveCount(1);
  await expect(page.locator('.encounter-panel')).toContainText(fr[unit].desc);
  for(const label of await page.locator('.road-prop-lettering').all()){
   await expect(label).toBeVisible();
   const text=label.locator('div');await expect(text).toContainText(fr[unit].overlay_0);
   expect(await text.evaluate(el=>el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)).toBe(true);
  }
  const actions=page.locator('.encounter-choice button');for(let i=0;i<3;i++)await expect(actions.nth(i)).toContainText(fr[unit][`choice_${i}`]);
  await actions.first().click();await expect(page.locator('.outcome-copy')).toContainText(fr[unit].log_0);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);expect(saved.party).toEqual(state.party);expect(saved.visual_content.outcomes[key]).toBe(0);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.outcome-copy')).toContainText(fr[unit].log_0);await context.setOffline(false);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(variant==='C')await page.screenshot({path:info.outputPath(`${family}-fr.png`),fullPage:true});
 }
});
