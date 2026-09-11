import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';import {join} from 'node:path';
import {baseline,importState,savedState,snap,settle} from './helpers';
import {atTown} from './geography';
const encounters=JSON.parse(readFileSync(join(__dirname,'../static/assets/data/game.json'),'utf8'));
test('numbered encounter, persistent aftermath, low sanity and visual results',async({page})=>{
 const state=await baseline(page,true);state.current_encounter=encounters.find((e:{id:string})=>e.id==='raw_milk');state.stats.hp=8;
 await importState(page,state);await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','10');await expect(page.locator('.game-clock')).toContainText('08:00');await snap(page,'milk-deep');
 await page.locator('body').click({position:{x:4,y:4}});await page.keyboard.press('1');await expect(page.locator('.aftermath-panel')).toBeVisible();
 const after=await savedState(page);await page.reload();await expect(page.locator('.aftermath-panel')).toBeVisible();expect((await savedState(page)).stats).toEqual(after.stats);
 state.current_encounter=null;state.stats.sanity=1;await importState(page,state);await expect(page.locator('.danger-notice')).toBeVisible();await snap(page,'danger');
 state.stats.pants=100;state.ending={type:'collapse',cause:'panic'};await importState(page,state);
 await expect(page.getByRole('heading',{name:'NATIONAL PANTS EMERGENCY'})).toBeVisible();await snap(page,'result');
 await page.getByRole('menuitem',{name:/Replay/i}).click();await expect(page.getByRole('radio',{name:'Journalist',exact:true})).toBeVisible();
});
test('repair consumes a pump and shows the cost',async({page})=>{
 const state=await baseline(page);state.breakdown={part:'FuelPump',day_started:0};state.day_state.travel_blocked=true;state.inventory.spares.pump=1;
 await importState(page,state);await page.getByRole('button',{name:'Fit your spare Fuel Pump',exact:true}).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
 const after=await savedState(page);expect(after.inventory.spares.pump).toBe(0);expect(after.breakdown).toBeNull();
 await expect(page.locator('.aftermath-panel')).toContainText('Fuel Pump');await expect(page.locator('.aftermath-panel')).toContainText('1 → 0 (-1)');await snap(page,'repair');
});
test('regions, weather details, final vote and terminal reload',async({page})=>{
 const state=await baseline(page,true);
 for(const [region,asset] of [['RustBelt','rustbelt'],['Beltway','beltway']]) {atTown(state,region==='RustBelt'?'Chicago':'Frederick');state.route_services.stop=null;await importState(page,state);await expect(page.locator('.journey-scene .scene-background').first()).toHaveAttribute('src',new RegExp(asset));await snap(page,asset+'-deep');}
 state.weather_state.today='ColdSnap';await importState(page,state);await expect(page.locator('.journey-scene')).toHaveAttribute('data-weather','cold');await page.getByRole('button',{name:'Assess conditions',exact:true}).click();await expect(page.locator('.weather-summary')).toContainText('Sanity -1');await expect(page.locator('.weather-summary')).not.toContainText(/weather\.|\{.*\}/);await snap(page,'cold');
 state.current_encounter=encounters.find((e:{id:string})=>e.id==='beltway_briefing');state.boss.ready=true;await importState(page,state);
 await expect(page.locator('.scene-atlas')).toHaveAttribute('data-cell','4');
 await page.locator('.encounter-choice button').first().click();await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','boss');await snap(page,'boss-deep');
 await page.getByRole('button',{name:'Resolve the final vote',exact:true}).dblclick();await expect(page.locator('#main')).toHaveAttribute('data-screen','result');await page.reload();await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
});
test('route store, one exchange and cooldown recovery',async({page})=>{
 const state=await baseline(page);atTown(state,'La Crosse');state.stats.supplies=10;state.camp.rest_cooldown=1;
 await importState(page,state);await snap(page,'route-stop');await page.getByRole('button',{name:/^Trade 4 supplies/}).click();
 const traded=await savedState(page);expect(traded.stats.supplies).toBe(6);expect(traded.inventory.spares.battery).toBe(state.inventory.spares.battery+1);
 await page.reload();await expect(page.getByRole('button',{name:'Community exchange used'})).toBeDisabled();
 await page.getByRole('button',{name:'Visit supply store'}).click();await page.getByRole('group',{name:'Rations Pack',exact:true}).getByRole('button',{name:'Add +1',exact:true}).click();
 await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();await page.getByRole('button',{name:'Buy supplies & return',exact:true}).click();
 const bought=await savedState(page);expect(bought.stats.supplies).toBe(9);expect(bought.budget_cents).toBe(state.budget_cents-500);expect(bought.day).toBe(state.day);
 await page.getByRole('button',{name:'Depart town',exact:true}).click();await page.getByRole('button',{name:'Resume travel',exact:true}).click();await settle(page);
 for(let attempts=0;attempts<8;attempts++) {
  const current=await savedState(page);if(current.day>state.day)break;
  if(await page.locator('.encounter-choice button').count()){await page.locator('.encounter-choice button').first().click();await page.getByRole('button',{name:'Back to the road',exact:true}).click();}
  await page.getByRole('button',{name:/^(Resume travel|Handle breakdown)$/}).click();await settle(page);
 }
 expect((await savedState(page)).camp.rest_cooldown).toBe(0);
});
