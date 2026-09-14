import {test,expect,Page} from '@playwright/test';
import { baseline,importState,snap, waitForLaunch } from './helpers';
import {atTown} from './geography';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));

test('Camp toggles without time or resource costs and Resume leaves camp in one click',async({page})=>{
 const gs=await baseline(page);gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;
 await importState(page,gs);
 const camp=page.getByRole('button',{name:'Camp',exact:true});
 await expect(camp).toHaveAttribute('aria-pressed','false');
 await camp.click();await expect(page.locator('#main')).toHaveAttribute('data-screen','camp');
 await expect(camp).toBeEnabled();await expect(camp).toHaveAttribute('aria-pressed','true');
 const resume=page.getByRole('button',{name:'Travel',exact:true});await expect(resume).toBeEnabled();
 const entered=(await checkpoint(page)).state;
 expect(entered.stats).toEqual(gs.stats);expect(entered.day).toBe(gs.day);expect(entered.clock_minutes).toBe(gs.clock_minutes);
 await camp.focus();await page.keyboard.press('Space');await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
 await expect(camp).toHaveAttribute('aria-pressed','false');
 expect((await checkpoint(page)).state.journal).toEqual(gs.journal);
 await camp.click();await page.reload();await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','camp');
 await expect(camp).toHaveAttribute('aria-pressed','true');await expect(resume).toBeEnabled();
 await snap(page,'camp-toggle');
 await resume.click();await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');
 await expect(page.locator('.camp-modal')).toHaveCount(0);expect((await checkpoint(page)).phase).toBe('Travel');
 await expect.poll(async()=>(await checkpoint(page)).state.miles_traveled_actual).toBeGreaterThan(gs.miles_traveled_actual);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();await expect(camp).toHaveAttribute('aria-pressed','false');
});

test('Camp toggles back to town and Resume from town camp departs without another confirmation',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Spokane');gs.day=6;gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;
 await importState(page,gs);const camp=page.getByRole('button',{name:'Camp',exact:true});
 await camp.click();await expect(camp).toHaveAttribute('aria-pressed','true');
 await camp.click();await expect(page.locator('.route-stop')).toContainText('Spokane');
 expect((await checkpoint(page)).state.route_services.stop).toBe(gs.route_services.stop);
 await camp.click();await page.getByRole('button',{name:'Leave town',exact:true}).click();
 await expect(page.locator('.map-scene')).toHaveAttribute('data-automatic','true');
 await expect(page.locator('.map-countdown')).toBeVisible();
 await expect(page.locator('.map-scene')).toHaveAttribute('data-running','true');
 expect((await checkpoint(page)).state.route_services.stop).toBeNull();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling',{timeout:9000});
 await expect.poll(async()=>(await checkpoint(page)).state.miles_traveled_actual).toBeGreaterThan(gs.miles_traveled_actual);
});

test('even zero-health breakdowns preserve resources and block Camp and Travel until an explicit repair',async({page})=>{
 const gs=await baseline(page);gs.breakdown={part:'Battery',day_started:gs.day};gs.inventory.spares.battery=1;gs.vehicle.health=0;gs.budget_cents=10000;await importState(page,gs);
 const camp=page.getByRole('button',{name:'Camp',exact:true});await expect(camp).toBeDisabled();await expect(camp).toHaveAttribute('aria-pressed','false');
 await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeDisabled();
 await expect(page.getByRole('button',{name:'Handle breakdown',exact:true})).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Fit your spare Battery',exact:true})).toBeVisible();
 await page.waitForTimeout(3100);const before=(await checkpoint(page)).state;expect(before.breakdown).toEqual(gs.breakdown);expect(before.inventory.spares).toEqual(gs.inventory.spares);expect(before.miles_traveled_actual).toBe(gs.miles_traveled_actual);expect(before.vehicle.health).toBe(0);expect(before.budget_cents).toBe(10000);expect(before.stats).toEqual(gs.stats);expect(before.day).toBe(gs.day);expect(before.clock_minutes).toBe(gs.clock_minutes);
 await page.evaluate(()=>{const c=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);c.phase='Camp';localStorage.setItem('dystrail.autosave.v1',JSON.stringify(c));});await page.reload();await waitForLaunch(page);
 await expect(page.locator('.camp-modal')).toHaveCount(0);await expect(camp).toBeDisabled();
 await page.getByRole('button',{name:'Fit your spare Battery',exact:true}).click();await page.getByRole('button',{name:'Back to the road',exact:true}).click();
 await expect(camp).toBeEnabled();await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeEnabled();const repaired=(await checkpoint(page)).state;expect(repaired.inventory.spares.battery).toBe(0);expect(repaired.budget_cents).toBe(10000);expect(repaired.vehicle.health).toBe(8);
});
