import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
test('crossing introductions match approved narratives rather than mode labels',()=>{
 const source=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8'));
 const crossings=source.units.filter((unit:any)=>unit.id.startsWith('CROSS-'));
 expect(crossings).toHaveLength(12);
 for(const unit of crossings){
  const narrative=unit.source_paragraphs.slice(1).find((text:string)=>
   !/^(Deep End only|Classic only)$/.test(text)&&!text.includes('→'));
  expect(narrative,unit.id).toBeTruthy();
  expect(copy[unit.id].desc,unit.id).toBe(narrative);
 }
});
const saved=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
test('crossing layers preserve later-route geography and absent crew',async({page},info)=>{
 const base=await baseline(page);
 const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
 const route=routes.find((r:any)=>r.id===base.persona_id);
 for(const region of ['Heartland','RustBelt','Beltway']){
  const stop=route.stops.find((s:any)=>s.region===region);
  expect(stop,region).toBeTruthy();
  const s=structuredClone(base);s.seed=42;s.miles_traveled_actual=(stop.mile+1)/route.total_miles*s.trail_distance;
  s.party.members[1].status='Departed';s.party.members[2].status='Dead';
  s.crossing_events=[{day:s.day,region,season:s.season,kind:'checkpoint',permit_used:false,bribe_attempted:false,bribe_success:null,bribe_cost_cents:0,bribe_chance:null,bribe_roll:null,detour_reason:null,detour_taken:false,detour_hours:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome:'passed'}];
  s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit:'CROSS-02D-A',permit_receipt:false,acknowledged:false}]};
  await importState(page,s);
  await expect(page.locator('.world-view')).toHaveAttribute('data-region',region);
  const expected=stop.scene==='open-heartland-prairie'?'open-heartland-orchard':stop.scene==='open-rustbelt-foundry'?'open-rustbelt-lakeside':stop.scene;
  await expect(page.locator('.crossing-layered-art')).toHaveAttribute('data-route-art',expected);
  await expect(page.locator('.van-occupant')).toHaveCount(4);
  for(const member of s.party.members.filter((m:any)=>m.status!=='Active'))await expect(page.locator(`.van-occupant[data-member="${member.persona}"]`)).toHaveCount(0);
  if(region==='Beltway')await page.screenshot({path:info.outputPath('crossing-beltway-survivors.png'),fullPage:true});
 }
});
test('Spanish and Arabic crossing narratives preserve receipt and permit distinctions',async({page,context},info)=>{
 test.setTimeout(180000);
 const base=await baseline(page);
 for(const locale of ['es','ar']){
  const translated=JSON.parse(readFileSync(`i18n/${locale}.json`,'utf8')).encounter_copy;
  for(const family of ['CROSS-01','CROSS-02C','CROSS-02D','CROSS-03'])for(const v of ['A','B','C']){
   const unit=`${family}-${v}`;const receipt=v==='A';
   for(const field of Object.keys(copy[unit]))expect(translated[unit][field],`${locale}/${unit}/${field}`).toBeTruthy();
   expect(translated[unit].permit_receipt).not.toBe(translated[unit].permit_tag);
   const s=structuredClone(base);s.seed=42;
   s.crossing_events=[{day:s.day,region:s.region,season:s.season,kind:'checkpoint',permit_used:true,bribe_attempted:false,bribe_success:null,bribe_cost_cents:0,bribe_chance:null,bribe_roll:null,detour_reason:null,detour_taken:false,detour_hours:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome:'passed'}];
   s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit,permit_receipt:receipt,acknowledged:false}]};
   await importState(page,s);
   await page.evaluate(locale=>localStorage.setItem('dystrail.locale',locale),locale);
   const final=family==='CROSS-03'&&v==='C';
   if(final)await context.setOffline(true);
   await page.reload();await waitForLaunch(page);
   await expect(page.locator('.crossing-message')).toContainText(translated[unit].desc);
   await expect(page.locator('.crossing-message')).toContainText(translated[unit][receipt?'permit_receipt':'permit_tag']);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   if(final)await page.screenshot({path:info.outputPath(`crossing-${locale}.png`),fullPage:true});
   await page.locator('#crossing-continue').click();
   await expect(page.locator('#crossing-continue')).toHaveCount(0);
   if(final)await context.setOffline(false);
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));
   await page.reload();await waitForLaunch(page);
  }
 }
});
test('crossing narratives acknowledge committed telemetry without changing the journey',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const family of ['CROSS-01','CROSS-02C','CROSS-02D','CROSS-03'])for(const v of ['A','B','C'])for(const field of ['passage','permit_receipt','permit_tag','bribe_success','diversion','bribe_failure',family==='CROSS-01'?'refused_passage':'terminal_failure']){
  const s=structuredClone(base);s.seed=42;const unit=`${family}-${v}`;
  const outcome=field==='terminal_failure'?'failed':['diversion','bribe_failure','refused_passage'].includes(field)?'detoured':'passed';
  s.crossing_events=[{day:s.day,region:s.region,season:s.season,kind:'checkpoint',permit_used:field.startsWith('permit_'),bribe_attempted:field.startsWith('bribe_'),bribe_success:field==='bribe_success'?true:field==='bribe_failure'?false:null,bribe_cost_cents:field.startsWith('bribe_')?700:0,bribe_chance:null,bribe_roll:null,detour_reason:field==='refused_passage'?'checkpoint_denied':outcome==='detoured'?'route_diversion':null,detour_taken:outcome==='detoured',detour_hours:outcome==='detoured'?2:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome}];
  s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit,permit_receipt:field==='permit_receipt',acknowledged:false}]};
  await importState(page,s);await expect(page.locator('#main')).toHaveAttribute('data-screen','crossing-outcome');await expect(page.locator('.crossing-message')).toContainText(copy[unit][field]);
  await expect(page.locator(".camp-toggle")).toBeDisabled();await expect(page.locator(".journey-actions .retro-btn-primary")).toBeDisabled();
  const before=await saved(page);
  if(['CROSS-02C-A','CROSS-02D-A'].includes(unit)&&['passage','diversion'].includes(field)){
   const asset=unit==='CROSS-02D-A'?'privacy':field==='passage'?'workers-raised':'workers-lowered';
   await expect(page.locator('.crossing-authored-outcome')).toHaveAttribute('href',new RegExp(`crossing-${asset}-v2.png$`));
   await expect(page.locator('.van-occupant')).toHaveCount(s.party.members.filter((m:any)=>m.status==='Active').length);
   if(unit==='CROSS-02D-A'&&field==='passage'){
    await context.setOffline(true);await page.reload();await waitForLaunch(page);
    await expect(page.locator('.crossing-authored-outcome')).toBeVisible();
    await context.setOffline(false);
   }
   await page.screenshot({path:info.outputPath(`${unit}-${field}.png`),fullPage:true});
  }
  if(family==='CROSS-03'&&v==='C'&&field==='terminal_failure'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.crossing-message')).toContainText(copy[unit][field]);await page.screenshot({path:info.outputPath('crossing-outcome.png'),fullPage:true});await context.setOffline(false);
  }
  await page.locator('#crossing-continue').click();await expect(page.locator('#crossing-continue')).toHaveCount(0);
  const after=await saved(page);expect(after.visual_content.crossing_presentations[0].acknowledged).toBe(true);
  // Inventory tags are a Rust HashSet: reload may change serialization order.
  after.inventory.tags.sort();before.inventory.tags.sort();
  after.visual_content=before.visual_content;expect(after).toEqual(before);
 }
});

test('retained checkpoint props preserve outcomes crew and offline state',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const [unit,asset] of [['CROSS-01-B','map'],['CROSS-02C-B','inspection'],['CROSS-02D-B','identity'],['CROSS-02D-C','chair'],['CROSS-03-B','folder'],['CROSS-03-C','bucket']])for(const outcome of ['passed','detoured']){
  const s=structuredClone(base);s.seed=42;s.party.members[1].status='Departed';s.party.members[2].status='Dead';
  s.crossing_events=[{day:s.day,region:s.region,season:s.season,kind:'checkpoint',permit_used:false,bribe_attempted:false,bribe_success:null,bribe_cost_cents:0,bribe_chance:null,bribe_roll:null,detour_reason:outcome==='detoured'?'route_diversion':null,detour_taken:outcome==='detoured',detour_hours:outcome==='detoured'?2:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome}];
  s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit,permit_receipt:false,acknowledged:false}]};
  await importState(page,s);
  await expect(page.locator('.crossing-authored-outcome')).toHaveAttribute('href',new RegExp(`crossing-${asset}-v2.png$`));
  await expect(page.locator('.crossing-message')).toContainText(copy[unit][outcome==='passed'?'passage':'diversion']);
  await expect(page.locator('.van-occupant')).toHaveCount(4);
  for(const m of s.party.members.filter((m:any)=>m.status!=='Active'))await expect(page.locator(`.van-occupant[data-member="${m.persona}"]`)).toHaveCount(0);
  const before=await saved(page);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.locator('.crossing-authored-outcome')).toBeVisible();
  const scene=await page.locator('.scene-art').boundingBox();expect(scene!.width/scene!.height).toBeCloseTo(1.5,1);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(outcome==='passed')await page.locator('.journey-scene').screenshot({path:info.outputPath(`${unit}.png`)});
  await page.locator('#crossing-continue').click();
  const after=await saved(page);expect(after.visual_content.crossing_presentations[0].acknowledged).toBe(true);
  before.inventory.tags.sort();after.inventory.tags.sort();after.visual_content=before.visual_content;expect(after).toEqual(before);
  await context.setOffline(false);
 }
});
