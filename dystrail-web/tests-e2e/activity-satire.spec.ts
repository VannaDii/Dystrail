import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch,snap} from './helpers';
import {atTown} from './geography';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
const copy=JSON.parse(readFileSync(join(__dirname,'../i18n/en.json'),'utf8')).encounter_copy;

test('all gathering and work variants keep their offer, receipt and current-save identity',async({page,context})=>{
 test.setTimeout(120_000);
 const original=await baseline(page);
 for(const family of ['FORAGE','GLEAN','FOODWORK','CASHWORK']) for(const suffix of ['A','B','C']) {
  const state=structuredClone(original);state.seed=42;state.day=6;state.clock_minutes=600;
  state.stats.supplies=5;state.stats.hp=8;state.stats.sanity=6;
  state.activities={foraged_on:null,worked_at:null,local_word:null};
  const town=['FOODWORK','CASHWORK'].includes(family);
  if(town) atTown(state,'Spokane');
  const unit=`ACT-${family}-${suffix}`;
  const key=`ACT-${family}/${town?'town':'day'}/${town?state.route_services.stop:state.day}`;
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  if(!town) await page.getByRole('button',{name:'Camp',exact:true}).click();
  await expect(page.locator('.activity-offer').filter({hasText:copy[unit].desc})).toBeVisible();
  await page.reload();await waitForLaunch(page);
  const button=page.getByRole('button',{name:copy[unit].choice_0,exact:true});
  await expect(button).toBeVisible();
  if(suffix==='A') await snap(page,`activity-${family.toLowerCase()}-offer`);
  await button.click();
  await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
  const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(after.visual_content.selections[key]).toBe(unit);expect(after.visual_content.outcomes[key]).toBe(0);
  expect(after.clock_minutes).toBe(600+(town?180:120));
  expect(after.stats.supplies).toBe(5+(family==='FORAGE'?2:family==='CASHWORK'?0:4));
  expect(after.stats.hp).toBe(8-(family==='GLEAN'?1:0));
  expect(after.stats.sanity).toBe(6+(family==='FORAGE'?1:town?-1:0));
  expect(after.budget_cents).toBe(state.budget_cents+(family==='CASHWORK'?1800:0));
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await context.setOffline(false);
 }
});

test('rest and all barter variants preserve current-save outcomes and limits',async({page,context})=>{
 test.setTimeout(120_000);
 const original=await baseline(page);
 for(const family of ['REST','BARTERTIRE','BARTERBATTERY','BARTERSUPPLIES']) for(const suffix of ['A','B','C']) {
  const state=structuredClone(original);state.seed=42;state.day=6;state.clock_minutes=600;
  state.stats.supplies=10;state.stats.hp=6;state.stats.sanity=6;state.inventory.spares.tire=1;
  state.camp.rest_cooldown=0;state.disease_cooldown=100;state.exec_order_cooldown=100;
  state.crew_care.last_check_day=100;state.turn_journal_start=null;
  const rest=family==='REST';
  if(!rest) atTown(state,'Spokane');
  const unit=`ACT-${family}-${suffix}`;
  const key=`ACT-${family}/${rest?'day':'town'}/${rest?state.day:state.route_services.stop}`;
  state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
  await importState(page,state);
  await page.getByRole('button',{name:rest?'Camp':'Trade with locals',exact:true}).click();
  await expect(page.getByText(copy[unit].desc,{exact:true})).toBeVisible();
  if(!rest) await expect(page.locator('[data-trade-completed]')).toHaveCount(0);
  await page.reload();await waitForLaunch(page);
  if(suffix==='A') await snap(page,`activity-${family.toLowerCase()}-offer`);
  await page.getByRole('button',{name:copy[unit].choice_0,exact:true}).click();
  await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
  const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(after.visual_content.selections[key]).toBe(unit);expect(after.visual_content.outcomes[key]).toBe(0);
  if(rest) {expect(after.day).toBe(7);expect(after.camp.rest_cooldown).toBeGreaterThan(0);}
  else {
   await expect(page.locator('[data-barter-unit]')).toHaveAttribute('data-barter-unit',unit);
   await expect(page.locator('.journey-scene .standing-member')).toHaveCount(0);
   expect(after.party).toEqual(state.party);
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
   await expect.poll(()=>page.evaluate(()=>scrollY)).toBe(0);
   await snap(page,`barter-completed-${unit}`);
   expect(after.route_services.traded_at).toBe(state.route_services.stop);
   expect(after.clock_minutes).toBe(630);
   expect(after.stats.supplies).toBe(family==='BARTERTIRE'?7:family==='BARTERBATTERY'?6:15);
   expect(after.inventory.spares.tire).toBe(family==='BARTERTIRE'?2:family==='BARTERSUPPLIES'?0:1);
   expect(after.inventory.spares.battery).toBe(state.inventory.spares.battery+(family==='BARTERBATTERY'?1:0));
  }
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
  if(!rest) await expect(page.locator('[data-barter-unit]')).toHaveAttribute('data-barter-unit',unit);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await context.setOffline(false);
 }
});
