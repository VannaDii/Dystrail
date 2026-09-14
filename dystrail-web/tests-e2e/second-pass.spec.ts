import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import { fastMode,baseline,depart,importState,savedState,setup,snap,openMenu, waitForLaunch } from './helpers';
import {atTown} from './geography';
const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
test('starter allocation is paid, editable, emptyable and persisted',async({page})=>{
 await setup(page);await expect(page.locator('.loadout-panel')).toContainText('$71');
 await page.getByRole('button',{name:'Empty the van',exact:true}).click();
 await expect(page.locator('.store-item-selected')).toHaveCount(0);await page.reload();await waitForLaunch(page);await expect(page.locator('.store-item-selected')).toHaveCount(0);for(const input of await page.locator('.store-qty').all())await expect(input).toHaveValue('0');
 await depart(page);const gs=await savedState(page);
 expect(gs.stats.supplies).toBe(0);expect(gs.inventory.spares).toEqual({tire:0,battery:0,alt:0,pump:0});expect(gs.budget_cents).toBe(12000);await snap(page,'empty-departure');
});
test('ending uses the name of the selected character',async({page})=>{
 const gs=await baseline(page);gs.party.leader='STALE';gs.party.members.find((m:any)=>m.persona===gs.persona_id).name='River Primary';
 await importState(page,gs);await openMenu(page);await page.getByRole('button',{name:'Abandon trail',exact:true}).click();await page.getByRole('button',{name:'End this journey',exact:true}).click();
 await expect(page.locator('.result-art-caption')).toContainText('River Primary');await expect(page.locator('.result-art-caption')).not.toContainText('STALE');
});
test('cashless breakdown has an escape with explicit costs across midnight',async({page})=>{
 const gs=await baseline(page);gs.breakdown={part:'Alternator',day_started:gs.day};gs.budget_cents=0;gs.stats.supplies=0;gs.stats.sanity=6;gs.stats.morale=6;gs.clock_minutes=1380;gs.vehicle.health=60;gs.inventory.spares={tire:0,battery:0,alt:0,pump:0};
 await importState(page,gs);await expect(page.getByRole('button',{name:'Fit your spare Alternator',exact:true})).toBeDisabled();await expect(page.getByRole('button',{name:'Order a replacement Alternator',exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Call local radio; work in exchange for repairs',exact:true}).click();
 const after=await savedState(page);expect(after.breakdown).toBeNull();expect(after.budget_cents).toBe(0);expect(after.stats.sanity).toBe(gs.stats.sanity-2+2);expect(after.stats.morale).toBe(5);expect(after.day).toBe(gs.day+1);expect(after.clock_minutes).toBe(720);expect(after.clock_minutes-480).toBe(240);expect(after.driving_minutes_total).toBe(gs.driving_minutes_total);expect(after.miles_traveled_actual).toBe(gs.miles_traveled_actual);await snap(page,'radio-repair');
});
test('foraging and paid town work apply costs and persist their limits',async({page})=>{
 const gs=await baseline(page);gs.stats.supplies=8;gs.stats.sanity=7;await importState(page,gs);
 await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toHaveCount(0);await page.getByRole('button',{name:'Camp',exact:true}).click();await page.getByRole('button',{name:'Forage with a local guide',exact:true}).click();const foraged=await savedState(page);expect(foraged.stats.supplies).toBe(10);expect(foraged.clock_minutes).toBe(gs.clock_minutes+120);
 await page.getByRole('button',{name:'Continue',exact:true}).click();await page.reload();await waitForLaunch(page);await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toBeDisabled();await expect(page.locator('.camp-gather').first()).toContainText('3 travel days');
 atTown(foraged,'La Crosse');await importState(page,foraged);await page.getByRole('button',{name:'Take a paid unloading shift',exact:true}).click();const worked=await savedState(page);expect(worked.budget_cents).toBe(foraged.budget_cents+1800);expect(worked.stats.sanity).toBe(foraged.stats.sanity-1);
 await page.getByRole('button',{name:'Back to town',exact:true}).click();await page.reload();await waitForLaunch(page);await expect(page.getByRole('button',{name:'Take a paid unloading shift',exact:true})).toBeDisabled();
});
test('town facts and player and NPC portraits survive reload',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Madison');await importState(page,gs);
 await page.getByRole('button',{name:/^Talk to locals/}).click();await expect(page.locator('.local-fact')).toContainText('UW–Madison');
 await expect(page.locator('.conversation-player')).toHaveAttribute('data-subject','journalist');await expect(page.locator('.conversation-player')).toContainText('Vanna Test');
 const npc=await page.locator('.scene-npc').getAttribute('data-npc');
 const fact=await page.locator('.local-fact').textContent(),remark=await page.locator('.resident-remark').textContent();
 await expect(page.locator('.local-fact')).toContainText('federal awards');await expect(page.locator('.resident-remark')).toContainText('bridge');
 await page.reload();await waitForLaunch(page);await expect(page.locator('.scene-npc')).toHaveAttribute('data-npc',npc!);
 await expect(page.locator('.local-fact')).toHaveText(fact!);await expect(page.locator('.resident-remark')).toHaveText(remark!);await snap(page,'town-conversation');
});
test('a new political encounter pays cash and exposes its factual hook',async({page})=>{
 const gs=await baseline(page);gs.current_encounter=bank.find((e:any)=>e.id==='sat_corn_bullets');await importState(page,gs);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','enc-farm-office');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-atlas','encounter-settings-v3');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','4');await page.getByRole('button',{name:'How this works: Behind the joke',exact:true}).click();await expect(page.locator('.viewport-help')).toContainText('five weekly');await expect(page.locator('.viewport-help a')).toHaveAttribute('href',/^https:/);
 await expect(page.locator('.source-explanation time')).toHaveAttribute('datetime','2025-02-28');
 await expect(page.locator('.source-explanation time')).toHaveText('2/28/2025');
 await expect(page.locator('.source-explanation > p')).toHaveCount(2);
 await expect(page.locator('.source-explanation > p').first()).toHaveAttribute('lang','en');
 await page.keyboard.press('Escape');
 await page.locator('.encounter-choice button').nth(1).click();const after=await savedState(page);expect(after.budget_cents).toBe(gs.budget_cents+800);await expect(page.locator('.aftermath-panel')).toContainText('Cash');await snap(page,'paid-satire');
});
test('road and progress elements survive consecutive automatic turns',async({page})=>{
 const gs=await baseline(page);gs.seed=42;gs.rng_bundle=null;gs.encounter_cooldown=100;gs.weather_state.neutral_buffer=100;gs.day_state.day_initialized=true;gs.encounter_chance_today=0;
 await importState(page,gs);await fastMode(page,true);
 await page.evaluate(()=>{(window as any).roadNode=document.querySelector('.road-pan-track');(window as any).progressNode=document.querySelector('.route-progress');});
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.journal.length),{timeout:7000}).toBeGreaterThanOrEqual(gs.journal.length+2);
 expect(await page.evaluate(()=>(window as any).roadNode===document.querySelector('.road-pan-track'))).toBe(true);expect(await page.evaluate(()=>(window as any).progressNode===document.querySelector('.route-progress'))).toBe(true);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();await snap(page,'continuous-road');
});
