import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch,snap} from './helpers';
import {atTown} from './geography';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';

test('recovered barter and rest translations survive choices and offline reload',async({page,context})=>{
 test.setTimeout(300_000);
 const original=await baseline(page);
 for(const lang of ['es','it','ar']) {
  const copy=JSON.parse(readFileSync(join(__dirname,`../i18n/${lang}.json`),'utf8')).encounter_copy;
  for(const unit of (process.env.ACTIVITY_FINAL_BATCH?['ACT-BARTERTIRE-A','ACT-BARTERBATTERY-A','ACT-BARTERSUPPLIES-A','ACT-REST-A','ACT-REST-B']:[ 'ACT-BARTERTIRE-A','ACT-BARTERBATTERY-A','ACT-BARTERSUPPLIES-A','ACT-REST-A','ACT-REST-B','ACT-BARTERTIRE-B','ACT-BARTERTIRE-C','ACT-BARTERBATTERY-B','ACT-BARTERBATTERY-C','ACT-BARTERSUPPLIES-B','ACT-BARTERSUPPLIES-C','ACT-REST-C'])) {
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));
   await page.reload();await waitForLaunch(page);
   const state=structuredClone(original);state.seed=42;state.day=6;state.clock_minutes=600;
   state.stats.supplies=10;state.stats.hp=6;state.stats.sanity=6;state.inventory.spares.tire=1;
   state.camp.rest_cooldown=0;state.disease_cooldown=100;state.exec_order_cooldown=100;
   state.crew_care.last_check_day=100;state.turn_journal_start=null;
   const rest=unit.startsWith('ACT-REST-');if(!rest) atTown(state,'Spokane');
   const family=unit.slice(0,-2),key=`${family}/${rest?'day':'town'}/${rest?state.day:state.route_services.stop}`;
   state.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
   await importState(page,state);
   await page.getByRole('button',{name:rest?'Camp':'Trade with locals',exact:true}).click();
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);
   await page.reload();await waitForLaunch(page);
   await expect(page.getByText(copy[unit].desc,{exact:true})).toBeVisible();
   await page.getByRole('button',{name:copy[unit].choice_0,exact:true}).click();
   await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
   const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   expect(after.visual_content.outcomes[key]).toBe(0);
   expect(after.party).toEqual(state.party);
   if(rest) expect(after.day).toBe(7);
   else {
    expect(after.clock_minutes).toBe(630);
    expect(after.stats.supplies).toBe(family==='ACT-BARTERTIRE'?7:family==='ACT-BARTERBATTERY'?6:15);
   }
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   if(unit===(process.env.ACTIVITY_FINAL_BATCH?'ACT-BARTERTIRE-A':'ACT-BARTERTIRE-B')) await snap(page,`barter-${lang}`);
   await context.setOffline(false);
  }
 }
});

test('recovered gathering and work translations keep real action effects',async({page,context},info)=>{
 test.setTimeout(300000);const original=await baseline(page);
 for(const lang of ['es','it','ar']) {
  const copy=JSON.parse(readFileSync(join(__dirname,`../i18n/${lang}.json`),'utf8')).encounter_copy;
  for(const family of ['FORAGE','GLEAN','FOODWORK','CASHWORK']) for(const variant of (process.env.ACTIVITY_FINAL_BATCH?['A']:['A','B','C'])) {
   if(process.env.ACTIVITY_PANTRY_ONLY&&(family!=='FOODWORK'||variant!=='A'))continue;
   if(process.env.ACTIVITY_WORK_ART_ONLY&&(!family.endsWith('WORK')||variant!=='A'))continue;
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
   const s=structuredClone(original);s.seed=42;s.day=6;s.clock_minutes=600;s.turn_journal_start=null;
   s.stats.supplies=5;s.stats.hp=8;s.stats.sanity=6;s.activities={foraged_on:null,worked_at:null,local_word:null};
   const town=family.endsWith('WORK');if(town)atTown(s,'Spokane');
   const unit=`ACT-${family}-${variant}`,key=`ACT-${family}/${town?'town':'day'}/${town?s.route_services.stop:s.day}`;
   s.visual_content={edition:1,selections:{[key]:unit},outcomes:{},policy_bulletins:[]};
   await importState(page,s);if(!town)await page.getByRole('button',{name:'Camp',exact:true}).click();
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);await page.reload();await waitForLaunch(page);
   await expect(page.getByText(copy[unit].desc,{exact:true})).toBeVisible();
   await page.getByRole('button',{name:copy[unit].choice_0,exact:true}).click();
   await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
   const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   expect(after.visual_content.outcomes[key]).toBe(0);expect(after.party).toEqual(s.party);
   expect(after.clock_minutes).toBe(600+(town?180:120));
   expect(after.stats.supplies).toBe(5+(family==='FORAGE'?2:family==='CASHWORK'?0:4));
   expect(after.stats.hp).toBe(8-(family==='GLEAN'?1:0));
   expect(after.stats.sanity).toBe(6+(family==='FORAGE'?1:town?-1:0));
   expect(after.budget_cents).toBe(s.budget_cents+(family==='CASHWORK'?1800:0));
   if(['ACT-FOODWORK-A','ACT-CASHWORK-A'].includes(unit)){
    await expect(page.locator('[data-activity-unit]')).toHaveAttribute('data-activity-unit',unit);
    await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
    const box=await page.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(760/248,1);
    if(family==='CASHWORK')await expect(page.locator('[data-empty-tip-jar]')).toBeVisible();
    await page.locator('.journey-scene').screenshot({path:info.outputPath(`${family}-${lang}.png`)});
   }
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.getByText(copy[unit].log_0,{exact:true})).toBeVisible();
   if(['ACT-FOODWORK-A','ACT-CASHWORK-A'].includes(unit))await expect(page.locator('[data-activity-unit]')).toHaveAttribute('data-activity-unit',unit);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   if(unit==='ACT-CASHWORK-B') {await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await snap(page,`work-${lang}`);}
   await context.setOffline(false);
  }
 }
});
