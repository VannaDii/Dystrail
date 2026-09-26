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
  if(part==='Tire'&&v==='A'&&choice===0){
   const details=await page.locator('.roadside-options .action-detail').allTextContents();
   expect(details).toHaveLength(4);
   expect(details.map(text=>text.split(' · ').at(-1))).toEqual(['1 hour','90 minutes','2 hours','4 hours']);
   for(const detail of details)expect(detail).not.toMatch(/[+−%]|\$\d/);
   expect(details[0]).toContain('Improves Vehicle');
   expect(details[1]).toContain('Costs Cash');
   expect(details[2]).toContain('Costs Supplies');
   expect(details[3]).toContain('Costs Sanity · Costs Morale');
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  }
  const before=await saved(page);await page.locator('.roadside-options .action-button').nth(choice).click();
  const after=await saved(page);expect(after.breakdown).toBeNull();expect(after.visual_content.outcomes[key]).toBe(choice);
  expect(after.clock_minutes).toBe(480+[60,90,120,240][choice]);expect(after.rng_bundle).toEqual(before.rng_bundle);
  expect(after.inventory.spares[spare]).toBe(choice===0?0:1);expect(after.budget_cents).toBe(choice===1?10000-cost:10000);
  expect(after.stats.supplies).toBe(choice===2?8:12);expect(after.stats.sanity).toBe(choice===3?8:10);expect(after.stats.morale).toBe(choice===3?7:8);
  await expect(page.locator('.outcome-copy')).toContainText(copy[unit][choice===1?'roadside':`log_${choice}`]);
  if(part==='Tire'&&v==='C'&&choice===3){await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(page.locator('.outcome-copy')).toContainText(copy[unit].log_3);const panel=await page.locator('.aftermath-panel').boundingBox();const actions=await page.locator('.outcome-actions').boundingBox();expect(actions!.y).toBeGreaterThanOrEqual(panel!.y+panel!.height-1);await page.screenshot({path:info.outputPath('repair-outcome.png'),fullPage:true});await context.setOffline(false);}
 }
});

test('localized battery and tire repairs retain the cashless exit and offline outcome',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const part of ['Battery','Tire'])for(const lang of ['es','it','ar'])for(const v of ['A','B','C']){
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
  const s=structuredClone(base);s.seed=42;s.day=3;s.clock_minutes=480;s.driving_minutes_total=180;
  s.breakdown={part,day_started:3};s.last_breakdown_part=part;s.vehicle.health=60;s.vehicle.wear=30;s.budget_cents=0;s.budget=0;
  s.inventory.spares={tire:0,battery:0,alt:0,pump:0};s.stats.supplies=0;s.stats.sanity=10;s.stats.morale=8;s.route_services.stop=null;
  const unit=`REPAIR-${part.toUpperCase()}-${v}`,key=`REPAIR-${part.toUpperCase()}/repair/180/3`;
  s.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
  await importState(page,s);await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);
  const translated=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).encounter_copy[unit];
  await expect(page.locator('#repair-title')).toHaveText(translated.name);await expect(page.locator('.roadside-options')).toContainText(translated.desc);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  const buttons=page.locator('.roadside-options .action-button');for(let i=0;i<3;i++)await expect(buttons.nth(i)).toBeDisabled();await buttons.nth(3).click();
  await expect(page.locator('.outcome-copy')).toContainText(translated.log_3);const after=await saved(page);
  expect(after.budget_cents).toBe(0);expect(after.stats.supplies).toBe(0);expect(after.stats.sanity).toBe(8);expect(after.breakdown).toBeNull();
  if(v==='C'){
   const values=await page.locator('.aftermath-panel .stat-card>strong').evaluateAll(nodes=>nodes.map(node=>({height:node.getBoundingClientRect().height,lineHeight:parseFloat(getComputedStyle(node).lineHeight)})));
   expect(values.length).toBeGreaterThan(0);
   for(const value of values)expect(value.height).toBeLessThanOrEqual(value.lineHeight+1);
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.outcome-copy')).toContainText(translated.log_3);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   await page.screenshot({path:info.outputPath(`${part.toLowerCase()}-${lang}.png`),fullPage:true});
   await context.setOffline(false);
  }
 }
});
