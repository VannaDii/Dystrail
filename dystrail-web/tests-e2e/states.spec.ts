import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';import {join} from 'node:path';
import { baseline,importState,savedState,snap,fastMode, waitForLaunch } from './helpers';
import {atTown} from './geography';
const encounters=JSON.parse(readFileSync(join(__dirname,'../static/assets/data/game.json'),'utf8'));
test('numbered encounter, persistent aftermath, low sanity and visual results',async({page})=>{
 const state=await baseline(page,true);state.current_encounter=encounters.find((e:{id:string})=>e.id==='raw_milk');state.stats.hp=8;
 await importState(page,state);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','enc-community');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-atlas','encounter-settings-v2');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','1');await expect(page.locator('.game-clock')).toContainText('08:00');await snap(page,'milk-deep');
 await page.locator('body').click({position:{x:4,y:4}});await page.keyboard.press('1');await expect(page.locator('.aftermath-panel')).toBeVisible();
 const after=await savedState(page);await page.reload();await waitForLaunch(page);await expect(page.locator('.aftermath-panel')).toBeVisible();expect((await savedState(page)).stats).toEqual(after.stats);
 state.current_encounter=null;state.stats.sanity=1;await importState(page,state);await expect(page.locator('.danger-notice')).toHaveCount(0);await expect(page.locator('.hud-stat.critical')).toBeVisible();await snap(page,'danger');
 state.stats.sanity=0;state.ending={type:'collapse',cause:'panic'};await importState(page,state);
 await expect(page.getByRole('heading',{name:'Nothing to See, Too Much to Hear',exact:true})).toBeVisible();await expect(page.locator('.result-screen')).not.toContainText(/Pants|steps/);await snap(page,'result');
 await page.getByRole('menuitem',{name:/Replay/i}).click();await expect(page.getByRole('radio',{name:'Journalist',exact:true})).toBeVisible();
});
test('repair consumes a pump and shows the cost',async({page})=>{
 const state=await baseline(page);state.breakdown={part:'FuelPump',day_started:0};state.day_state.travel_blocked=true;state.inventory.spares.pump=1;state.vehicle.health=75;
 await importState(page,state);await page.getByRole('button',{name:'Fit your spare Fuel Pump',exact:true}).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
 const after=await savedState(page);expect(after.inventory.spares.pump).toBe(0);expect(after.breakdown).toBeNull();expect(after.vehicle.health).toBe(83);
 await expect(page.locator('.resource-changes .change').filter({hasText:'Fuel Pump'})).toContainText('-1');
 await expect(page.locator('.resource-changes .change').filter({hasText:'Vehicle condition'})).toContainText('+8.00%');await snap(page,'repair');
});
test('regions, weather details, final vote and terminal reload',async({page})=>{
 const state=await baseline(page,true);
 for(const [region,asset] of [['RustBelt','rustbelt'],['Beltway','beltway']]) {atTown(state,region==='RustBelt'?'Chicago':'Frederick');state.route_services.stop=null;await importState(page,state);await expect(page.locator('.journey-scene .scene-background').first()).toHaveAttribute('src',new RegExp(asset));await snap(page,asset+'-deep');}
 // This checkpoint represents the day after its automatic cold-weather charge.
 state.weather_state.today='ColdSnap';state.stats.sanity-=1;state.weather_impact={day:state.day,weather:'ColdSnap',supplies:0,hp:0,sanity:-1};
 await importState(page,state);await expect(page.locator('.journey-scene')).toHaveAttribute('data-weather','cold');await page.getByRole('tab',{name:'Conditions',exact:true}).click();await expect(page.locator('.weather-indicator .weather-impact')).toHaveCount(0);await page.locator('.weather-indicator .help-trigger').click();await expect(page.locator('.viewport-help .weather-details [data-stat="ux.sanity"] strong')).toHaveText('-1');await expect(page.locator('.viewport-help .weather-details')).not.toContainText(/weather\.|\{.*\}/);await snap(page,'cold');await page.keyboard.press('Escape');
 state.current_encounter=encounters.find((e:{id:string})=>e.id==='beltway_briefing');state.boss.ready=true;await importState(page,state);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','enc-service-counter');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-atlas','encounter-settings-v3');await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','5');
 await page.locator('.encounter-choice button').first().click();await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','boss');await snap(page,'boss-deep');
 await page.getByRole('button',{name:'Resolve the final vote',exact:true}).dblclick();await expect(page.locator('#main')).toHaveAttribute('data-screen','result');await page.reload();await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
});
test('route store, one exchange and cooldown recovery',async({page})=>{
 const state=await baseline(page);atTown(state,'La Crosse');state.prev_miles_traveled=state.miles_traveled_actual;state.clock_minutes=660;state.stats.supplies=10;state.camp.rest_cooldown=1;
 state.seed=42;state.rng_bundle=null;state.encounter_cooldown=100;state.weather_state.neutral_buffer=100;state.day_state.day_initialized=true;state.encounter_chance_today=0;
 await importState(page,state);await snap(page,'route-stop');await page.getByRole('button',{name:'Trade with locals',exact:true}).click();
 await expect(page.locator('.town-trading .action-grid .action-button')).toHaveCount(3);await page.getByRole('button',{name:'Get a battery',exact:true}).click();
 const traded=await savedState(page);expect(traded.stats.supplies).toBe(6);expect(traded.inventory.spares.battery).toBe(state.inventory.spares.battery+1);expect(traded.clock_minutes).toBe(state.clock_minutes+30);
 await page.reload();await waitForLaunch(page);await expect(page.locator('.town-trading')).toBeVisible();
 await expect(page.locator('.town-trading')).toContainText('Community exchange used');
 for(const offer of await page.locator('.town-trading .action-grid .action-button').all())await expect(offer).toBeDisabled();
 await page.getByRole('button',{name:'Back to town',exact:true}).click();
 await page.getByRole('button',{name:'Visit the Store',exact:true}).click();await page.getByRole('group',{name:'Rations Pack',exact:true}).getByRole('button',{name:'Add +1',exact:true}).click();
 await page.getByRole('button',{name:'Buy supplies & return',exact:true}).click();
 const bought=await savedState(page);expect(bought.stats.supplies).toBe(9);expect(bought.budget_cents).toBe(state.budget_cents-500);expect(bought.day).toBe(state.day);expect(bought.clock_minutes).toBe(state.clock_minutes+60);
 await fastMode(page,true);await page.getByRole('button',{name:'Leave town',exact:true}).click();
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.day),{timeout:10000}).toBeGreaterThan(state.day);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 expect((await savedState(page)).camp.rest_cooldown).toBe(0);
});
