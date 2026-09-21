import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
import {atTown} from './geography';
test('inspect overnight work',async({page})=>{
 page.on('console',m=>{if(m.type()==='error')console.log('console-error',m.text())});
 page.on('pageerror',e=>console.log('page-error',e.message));
 const gs=await baseline(page);atTown(gs,'La Crosse');
 gs.clock_minutes=770;gs.stats.supplies=8;gs.stats.hp=8;gs.stats.sanity=8;
 gs.day_state.day_initialized=true;gs.activities.worked_at=null;
 gs.exec_order_cooldown=100;gs.disease_cooldown=100;gs.weather_state.today='Storm';gs.weather_state.neutral_buffer=100;
 await importState(page,gs);
 const button=page.locator('.action-option:has([data-unit^="ACT-FOODWORK-"]) .action-button');
 console.log('button',await button.textContent());await button.click();
 await page.waitForTimeout(250);
 console.log('after',await page.evaluate(()=>({screen:document.querySelector('#main')?.getAttribute('data-screen'),checkpoint:JSON.parse(localStorage.getItem('dystrail.autosave.v1')!)})));
 await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
});
