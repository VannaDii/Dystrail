import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
const saved=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
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
  if(family==='CROSS-03'&&v==='C'&&field==='terminal_failure'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.crossing-message')).toContainText(copy[unit][field]);await page.screenshot({path:info.outputPath('crossing-outcome.png'),fullPage:true});await context.setOffline(false);
  }
  await page.locator('#crossing-continue').click();await expect(page.locator('#crossing-continue')).toHaveCount(0);
  const after=await saved(page);expect(after.visual_content.crossing_presentations[0].acknowledged).toBe(true);
  after.visual_content=before.visual_content;expect(after).toEqual(before);
 }
});
