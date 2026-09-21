import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const texts=JSON.parse(readFileSync('i18n/en.json','utf8')).visual_copy;
function fixture(unit:string,outcome:string){
 const save=structuredClone(base);save.phase='Travel';save.aftermath=null;const s=save.state;
 s.day=6;s.clock_minutes=720;s.breakdown=null;s.current_encounter=null;s.crew_care.pending=null;s.ally_notice=null;s.route_services.stop=null;
 s.party.members[1].status='Dead';s.party.members[3].status='Left';
 s.crossing_events=[{day:6,region:s.region,season:'spring',kind:'checkpoint',permit_used:false,bribe_attempted:true,bribe_success:outcome==='passed',bribe_cost_cents:700,bribe_chance:0.5,bribe_roll:0.4,detour_taken:outcome==='detoured',detour_reason:outcome==='detoured'?'route_diversion':null,detour_hours:outcome==='detoured'?2:null,detour_base_supplies_delta:outcome==='detoured'?-1:null,detour_extra_supplies_loss:null,terminal_threshold:0.1,terminal_roll:null,outcome}];
 s.visual_content={edition:1,selections:{[`${unit.slice(0,-2)}/crossing/0`]:unit},crossing_presentations:[{event_index:0,unit,acknowledged:false}]};
 return save;
}
async function stored(page:any){return page.evaluate(()=>{let s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);s.state.inventory.tags.sort();return s;});}
async function load(page:any,save:any){await page.evaluate(s=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(s));localStorage.setItem('dystrail.locale','en');},save);await page.reload();await waitForLaunch(page);}
async function screenshot(page:any,path:string){await page.evaluate(()=>{scrollTo(0,0);if(document.activeElement instanceof HTMLElement)document.activeElement.blur();});await page.screenshot({path,fullPage:true,animations:'disabled'});}
test('crossing outcomes preserve crew, costs and saved acknowledgment',async({page},info)=>{
 test.setTimeout(180000);await page.goto('./');await waitForLaunch(page);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 for(const unit of ['CROSS-01-A','CROSS-02C-A','CROSS-02D-A','CROSS-03-A'])for(const outcome of ['passed','detoured','failed']){
  const save=fixture(unit,outcome);await load(page,save);await expect(page.locator('[data-screen="crossing-outcome"]')).toBeVisible();
  await expect(page.locator('.crossing-composition')).toHaveAttribute('data-crossing-outcome',outcome);await expect(page.locator('#screen-title')).toHaveText(texts[unit].title);
  await expect(page.locator('.crossing-message')).toContainText(texts[unit].outcomes[outcome==='passed'?'bribe_success':outcome==='detoured'?'diversion':'terminal_failure']);
  await expect(page.locator('.crossing-receipt')).toContainText('−$7.00');await expect(page.locator('.scene-portrait,.parked-crew')).toHaveCount(0);
  const active=save.state.party.members.filter((m:any)=>m.status==='Active').map((m:any)=>m.persona).sort();expect((await page.locator('.van-occupant').evaluateAll(es=>es.map(e=>e.getAttribute('data-member')))).sort()).toEqual(active);
  const art=await page.locator('.scene-art').boundingBox(),caption=await page.locator('.scene-caption').boundingBox();expect(art!.width/art!.height).toBeCloseTo(1.5,2);expect(caption!.y+caption!.height).toBeCloseTo(art!.y+art!.height,0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await screenshot(page,info.outputPath(`${unit}-${outcome}.png`));const before=await stored(page);
  await page.reload();await waitForLaunch(page);await expect(page.locator('.crossing-composition')).toHaveAttribute('data-crossing-outcome',outcome);expect((await stored(page)).state).toEqual(before.state);
  await page.locator('#crossing-continue').focus();await page.keyboard.press('Enter');await expect(page.locator('.crossing-outcome')).toHaveCount(0);
  const after=await stored(page);const expected=structuredClone(before.state);expected.visual_content.crossing_presentations[0].acknowledged=true;expect(after.state).toEqual(expected);
  await page.reload();await waitForLaunch(page);await expect(page.locator('.crossing-outcome')).toHaveCount(0);
  if(unit==='CROSS-02C-A'&&outcome==='passed')writeFileSync('../review/art-satire/client-fixtures/crossing.json',JSON.stringify(save));
 }
 expect(errors).toEqual([]);
});
test('legacy crossings do not replay and queued scenes retain their underlying decision',async({page},info)=>{
 await page.goto('./');await waitForLaunch(page);let save=fixture('CROSS-01-A','passed');delete save.state.visual_content.crossing_presentations;await load(page,save);await expect(page.locator('.crossing-outcome')).toHaveCount(0);
 save=fixture('CROSS-01-A','passed');save.phase='Camp';save.state.clock_minutes=1380;await load(page,save);await expect(page.locator('.journey-scene')).toHaveAttribute('data-time','night');const before=await stored(page);await screenshot(page,info.outputPath('crossing-night.png'));await page.locator('#crossing-continue').click();await expect(page.locator('.crossing-outcome')).toHaveCount(0);expect((await stored(page)).phase).toEqual(before.phase);
});
