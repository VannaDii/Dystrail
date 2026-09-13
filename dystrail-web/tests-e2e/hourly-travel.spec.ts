import {test,expect,Page} from '@playwright/test';
import {baseline,importState,fastMode,snap} from './helpers';

type Pace = 'steady' | 'heated' | 'blitz';
type SavedState = Awaited<ReturnType<typeof baseline>>;

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
const mileageCard=(page:Page)=>page.locator('.turn-receipt [data-stat="play.miles"]');

function clearRoad(source:SavedState,pace:Pace='steady') {
 const state=structuredClone(source);
 // Isolate the driving clock from scheduled towns, encounters and daily hazards.
 // This route uses road miles directly, as the engine's uninterrupted-road fixture does.
 state.route_services.route_id='uninterrupted-test-road';
 state.day_state.day_initialized=true;
 state.weather_state.today='Clear';state.weather_state.neutral_buffer=100;
 state.weather_travel_multiplier=1;state.exec_travel_multiplier=1;
 state.exec_breakdown_bonus=0;state.illness_travel_penalty=1;
 state.vehicle.breakdown_cooldown=100;
 // Keep the exposure-based encounter clock outside this isolated five-hour drive.
 state.last_encounter_driving_minutes=600;state.crew_care.last_check_day=100;
 state.stats.hp=10;state.stats.sanity=10;state.stats.supplies=16;
 state.pace=pace;state.driving_minutes_total=0;
 return state;
}

async function driveOneHour(page:Page) {
 const before=await checkpoint(page);
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 // The autosave includes the pending result before its travel animation finishes.
 // Pause after that result exists, then wait for the visible state to catch up.
 await page.waitForFunction(length=>{
  const saved=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);
  return saved.state.journal.length>length;
 },before.journal.length);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
 await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeEnabled();
 const after=await checkpoint(page);
 expect(after.driving_minutes_total-before.driving_minutes_total).toBe(60);
 await expect(mileageCard(page).locator('small')).toHaveText('1 h driving');
 return after;
}

for(const [pace,mph] of [['steady',60],['heated',70],['blitz',80]] as const) {
 test(`${pace} covers ${mph} miles during one clear, healthy driving hour`,async({page})=>{
  const state=clearRoad(await baseline(page),pace);
  await importState(page,state);await fastMode(page,true);
  const after=await driveOneHour(page);
  expect(after.day).toBe(state.day);
  expect(after.clock_minutes-state.clock_minutes).toBe(60);
  expect(after.miles_traveled_actual-state.miles_traveled_actual).toBeCloseTo(mph,4);
  await expect(page.locator('.conditions-hud .hud-context')).toHaveText('Day 1 · 09:00');
  await expect(mileageCard(page).locator('strong')).toHaveText(`+${mph}.0`);
 });
}

test('five driving hours cover 300 miles and overnight rollover adds no driving time',async({page})=>{
 test.setTimeout(60000);
 const state=clearRoad(await baseline(page));
 await importState(page,state);await fastMode(page,true);
 let after=state;
 for(let hour=1;hour<=5;hour++) {
  after=await driveOneHour(page);
  expect(after.miles_traveled_actual-state.miles_traveled_actual).toBeCloseTo(hour*60,4);
 }
 expect(after.day).toBe(state.day+1);
 expect(after.clock_minutes).toBe(480);
 expect(after.driving_minutes_total-state.driving_minutes_total).toBe(300);
 await expect(page.locator('.conditions-hud .hud-context')).toHaveText('Day 2 · 08:00');
 await expect(page.locator('.turn-receipt [data-stat="play.elapsed"] strong')).toHaveText('+1');
 await expect(mileageCard(page).locator('strong')).toHaveText('+60.0');
 await expect(mileageCard(page).locator('small')).toHaveText('1 h driving');
 await snap(page,'hourly-day-rollover');
});

test('two hours of foraging leave three driving hours and 180 miles that day',async({page})=>{
 test.setTimeout(60000);
 const state=clearRoad(await baseline(page));
 await importState(page,state);await fastMode(page,true);
 await page.getByRole('button',{name:'Camp',exact:true}).click();
 await page.getByRole('button',{name:'Forage with a local guide',exact:true}).click();
 await expect(page.locator('#outcome-continue')).toBeVisible();
 const gathered=await checkpoint(page);
 expect(gathered.day).toBe(state.day);
 expect(gathered.clock_minutes-state.clock_minutes).toBe(120);
 expect(gathered.driving_minutes_total).toBe(state.driving_minutes_total);
 expect(gathered.miles_traveled_actual).toBe(state.miles_traveled_actual);
 expect(gathered.stats.supplies-state.stats.supplies).toBe(2);
 await expect(page.locator('.conditions-hud .hud-context')).toHaveText('Day 1 · 10:00');
 await page.locator('#outcome-continue').click();
 await page.getByRole('button',{name:'Camp',exact:true}).click();
 let after=gathered;
 for(let hour=1;hour<=3;hour++) {
  after=await driveOneHour(page);
  expect(after.miles_traveled_actual-state.miles_traveled_actual).toBeCloseTo(hour*60,4);
 }
 expect(after.day).toBe(state.day+1);
 expect(after.clock_minutes).toBe(480);
 expect(after.driving_minutes_total-state.driving_minutes_total).toBe(180);
 expect(after.miles_traveled_actual-state.miles_traveled_actual).toBeCloseTo(180,4);
 await expect(page.locator('.conditions-hud .hud-context')).toHaveText('Day 2 · 08:00');
 await snap(page,'hourly-foraging-cost');
});
