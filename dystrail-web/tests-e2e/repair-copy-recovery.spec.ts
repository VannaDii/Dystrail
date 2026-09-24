import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
const parts={Tire:['TIRE','tire',2500],Battery:['BATTERY','battery',3200],Alternator:['ALTERNATOR','alt',3100],FuelPump:['FUELPUMP','pump',2700]} as const;
const saved=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
test('all twelve repair stories preserve four costed choices and current saves',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const [part,[family,spare,cost]] of Object.entries(parts))for(const v of ['A','B','C'])for(let choice=0;choice<4;choice++){
  const s=structuredClone(base);s.seed=42;s.day=3;s.clock_minutes=480;s.driving_minutes_total=180;
  s.breakdown={part,day_started:3};s.last_breakdown_part=part;s.vehicle.health=60;s.vehicle.wear=30;s.budget_cents=10000;s.budget=100;
  s.inventory.spares={tire:1,battery:1,alt:1,pump:1};s.stats.supplies=12;s.stats.sanity=10;s.stats.morale=8;s.route_services.stop=null;
  const unit=`REPAIR-${family}-${v}`,key=`REPAIR-${family}/repair/180/3`;
  s.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
  await importState(page,s);await expect(page.locator('#repair-title')).toHaveText(copy[unit].name);
  await expect(page.locator('.roadside-options')).toContainText(copy[unit].desc);
  const before=await saved(page);await page.locator('.roadside-options .action-button').nth(choice).click();
  const after=await saved(page);expect(after.breakdown).toBeNull();expect(after.visual_content.outcomes[key]).toBe(choice);
  expect(after.clock_minutes).toBe(480+[60,90,120,240][choice]);expect(after.rng_bundle).toEqual(before.rng_bundle);
  expect(after.inventory.spares[spare]).toBe(choice===0?0:1);expect(after.budget_cents).toBe(choice===1?10000-cost:10000);
  expect(after.stats.supplies).toBe(choice===2?8:12);expect(after.stats.sanity).toBe(choice===3?8:10);expect(after.stats.morale).toBe(choice===3?7:8);
  await expect(page.locator('.outcome-copy')).toContainText(copy[unit][choice===1?'roadside':`log_${choice}`]);
  if(part==='Tire'&&v==='C'&&choice===3){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_3);const panel=await page.locator('.aftermath-panel').boundingBox();const actions=await page.locator('.outcome-actions').boundingBox();expect(actions!.y).toBeGreaterThanOrEqual(panel!.y+panel!.height-1);await page.screenshot({path:info.outputPath('repair-outcome.png'),fullPage:true});await context.setOffline(false);}
 }
});
