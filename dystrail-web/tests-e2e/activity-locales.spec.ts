import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch,snap} from './helpers';
import {atTown} from './geography';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';

test('recovered barter and rest translations survive choices and offline reload',async({page,context})=>{
 test.setTimeout(180_000);
 const original=await baseline(page);
 for(const lang of ['es','it','ar']) {
  const copy=JSON.parse(readFileSync(join(__dirname,`../i18n/${lang}.json`),'utf8')).encounter_copy;
  for(const unit of ['ACT-BARTERTIRE-B','ACT-BARTERTIRE-C','ACT-BARTERBATTERY-B','ACT-BARTERBATTERY-C','ACT-BARTERSUPPLIES-B','ACT-BARTERSUPPLIES-C','ACT-REST-C']) {
   await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));
   await page.reload();await waitForLaunch(page);
   const state=structuredClone(original);state.seed=42;state.day=6;state.clock_minutes=600;
   state.stats.supplies=10;state.stats.hp=6;state.stats.sanity=6;state.inventory.spares.tire=1;
   state.camp.rest_cooldown=0;state.disease_cooldown=100;state.exec_order_cooldown=100;
   state.crew_care.last_check_day=100;state.turn_journal_start=null;
   const rest=unit==='ACT-REST-C';if(!rest) atTown(state,'Spokane');
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
   if(unit==='ACT-BARTERTIRE-B') await snap(page,`barter-${lang}`);
   await context.setOffline(false);
  }
 }
});
