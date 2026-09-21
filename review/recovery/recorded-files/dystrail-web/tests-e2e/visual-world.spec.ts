import {test,expect} from '@playwright/test';
import {baseline,importState,savedState,snap,waitForLaunch,fastMode} from './helpers';

test('six cast identities, absence and replacement driver use independent poses',async({page})=>{
 const state=await baseline(page);
 await expect(page.locator('.standing-member')).toHaveCount(6);
 await expect(page.locator('.standing-member .cast-art[data-pose="4"]')).toHaveCount(6);
 await expect(page.locator('.crew-color-key')).toHaveCount(0);
 await snap(page,'cast-v2-stopped');
 state.seed=42;state.rng_bundle=null;state.encounter_cooldown=100;
 state.weather_state.neutral_buffer=100;state.day_state.day_initialized=true;state.encounter_chance_today=0;
 await importState(page,state);await fastMode(page,false);
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(page.locator('.van-occupant')).toHaveCount(6);
 await expect(page.locator('.van-occupant[data-seat="4"] .cast-art')).toHaveAttribute('data-pose','7');
 await snap(page,'cast-v2-moving');
 for(const member of state.party.members) if(member.persona!=='whistleblower') member.status='Departed';
 await importState(page,state);await expect(page.locator('.standing-member')).toHaveCount(1);
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(page.locator('.van-occupant')).toHaveCount(1);
 await expect(page.locator('.van-occupant')).toHaveAttribute('data-member','whistleblower');
 await expect(page.locator('.van-occupant')).toHaveAttribute('data-seat','4');
 await expect(page.locator('.van-occupant .cast-art')).toHaveAttribute('data-pose','7');
 await snap(page,'cast-v2-single-driver');
});

test('billboard and clock lighting survive import, reload, offline and localization',async({page,context})=>{
 const state=await baseline(page);
 const sign=await page.locator('.road-billboard').getAttribute('data-billboard');
 for(const [hour,light] of [[8,'morning'],[13,'day'],[16,'afternoon'],[19,'dusk'],[23,'night']] as const){
  state.clock_minutes=hour*60;await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-time',light);
  await expect(page.locator('.road-billboard')).toHaveAttribute('data-billboard',sign!);
  await snap(page,`road-light-${light}`);
 }
 const before=await savedState(page);await page.reload();await waitForLaunch(page);
 expect((await savedState(page)).stats).toEqual(before.stats);
 await expect(page.locator('.road-billboard')).toHaveAttribute('data-billboard',sign!);
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await expect(page.locator('.road-billboard')).toHaveAttribute('data-billboard',sign!);
 await expect(page.locator('.standing-member .cast-art')).toHaveCount(6);
 await context.setOffline(false);
 for(const locale of ['es','it','ar']) {
  await page.evaluate(locale=>localStorage.setItem('dystrail.locale',locale),locale);
  await page.reload();await waitForLaunch(page);
  await expect(page.locator('.billboard-panel')).not.toContainText(/road_ad\./);
  const clipping=await page.locator('.billboard-panel').evaluate(el=>el.scrollWidth>el.clientWidth+1);
  expect(clipping).toBe(false);
  await snap(page,`road-ad-${locale}`);
 }
 await expect(page.locator('html')).toHaveAttribute('dir','rtl');
});
