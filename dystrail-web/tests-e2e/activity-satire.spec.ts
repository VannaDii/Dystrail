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
