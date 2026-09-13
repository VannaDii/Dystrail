import {test,expect} from '@playwright/test';
import {setup,depart,savedState,snap,settle,openMenu} from './helpers';
test('outfitting recovery, direct controls, timed travel, manual save and camp',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('./');await expect(page).toHaveTitle(/Dystopian Trail/);
 await expect(page.locator('#main')).toHaveAttribute('data-screen','setup');
 expect(await page.evaluate(()=>document.activeElement?.tagName)).toBe('BODY');
 expect(await page.evaluate(()=>scrollY)).toBe(0);
 await snap(page,'setup');await setup(page);
 await page.getByRole('group',{name:'Fuel Pump',exact:true}).getByRole('button',{name:'Add +1',exact:true}).click();
 await expect(page.locator('.store-cart-summary .cart-total')).toHaveText('$88');
 await page.reload();await expect(page.locator('.store-cart-summary .cart-total')).toHaveText('$88');
 expect(await page.evaluate(()=>document.activeElement?.tagName)).toBe('BODY');
 await snap(page,'outfitting');
 await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
 await page.reload();await expect(page.getByRole('button',{name:'Start the journey',exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Start the journey',exact:true}).click();
 await expect(page.locator('.critical-stats')).toContainText('Credibility');
 await expect(page.locator('.critical-stats')).toContainText('Morale');
 await savedState(page);await snap(page,'travel');
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');
 const recovery=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
 await page.reload();await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
 await expect(page.locator('#main')).toHaveAttribute('data-screen',recovery.phase.toLowerCase());
 expect((await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!))).state.day).toBe(recovery.state.day);
 if(await page.locator('.encounter-choice button').count()) {
  await page.locator('body').click({position:{x:4,y:4}});await page.keyboard.press('1');
  await expect(page.locator('.aftermath-panel')).toBeVisible();await snap(page,'aftermath');
  await page.reload();await expect(page.locator('.aftermath-panel')).toBeVisible();
  await page.getByRole('button',{name:'Back to the road',exact:true}).click();
 }
 // Deliberate manual slot remains distinct from recovery.
 await openMenu(page);await page.locator('#save-open-btn').click();
 await page.locator('.drawer').getByRole('button',{name:'Load',exact:true}).click();
 await expect(page.locator('.drawer')).toHaveCount(0);
 await page.getByRole('button',{name:'Camp',exact:true}).filter({visible:true}).last().click();
 await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','3');
 await snap(page,'camp');await page.getByRole('button',{name:/^Rest$/}).click();
 await expect(page.locator('.aftermath-panel')).toBeVisible();
 await page.getByRole('button',{name:'Back to the road',exact:true}).click();
 await page.getByRole('button',{name:'Camp',exact:true}).last().click();
 const rest=page.locator('.camp-action').filter({has:page.getByRole('button',{name:'Rest',exact:true})});
 await expect(rest.locator('.cooldown-status')).toContainText('Rest available in 2 completed days');
 await expect(rest.getByRole('button',{name:'Rest',exact:true})).toBeDisabled();
 expect(errors).toEqual([]);
});
