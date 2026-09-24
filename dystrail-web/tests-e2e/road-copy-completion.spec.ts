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
