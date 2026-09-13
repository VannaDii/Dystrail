import {test,expect,Page} from '@playwright/test';
import {fastMode,baseline,importState,snap,openMenu} from './helpers';
const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
async function weatherDetails(page:Page){
 await expect(page.locator('.weather-indicator .weather-impact')).toHaveCount(0);
 await page.locator('.weather-indicator .help-trigger').click();
 const details=page.locator('.viewport-help .weather-details');await expect(details).toBeVisible();return details;
}
function quiet(gs:any){
 gs.seed=5;gs.rng_bundle=null;gs.region='Heartland';gs.current_encounter=null;gs.encounter_cooldown=100;gs.encounter_chance_today=0;
 gs.stats={...gs.stats,supplies:20,sanity:10,hp:10,morale:10,pants:0};
 gs.weather_state={today:'Clear',yesterday:'Clear',extreme_streak:0,heatwave_streak:0,coldsnap_streak:0,neutral_buffer:0};
 gs.day_state.day_initialized=false;gs.day_state.travel_blocked=false;gs.day=1;gs.weather_impact=null;
 gs.inventory.tags=['water_jugs'];return gs;
}
test('weather costs apply during continuous travel, notify inline, and survive reload once',async({page})=>{
 const gs=quiet(await baseline(page));await importState(page,gs);await fastMode(page,false);
 const count=gs.journal.length;
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect.poll(async()=>(await checkpoint(page)).state.weather_impact).toBeTruthy();
 const charged=(await checkpoint(page)).state;
 console.log('Weather travel fixture',charged.seed,charged.region,charged.weather_state.today,charged.weather_impact);
 expect(charged.weather_state.today).not.toBe('Clear');
 await expect(page.locator('.weather-indicator')).toHaveAttribute('data-weather',charged.weather_state.today,{timeout:6000});
 await expect(page.locator('.weather-indicator')).toHaveClass(/weather-changed/);
 const details=await weatherDetails(page);
 await expect(details.locator('[data-stat="ux.supplies"] strong')).toHaveText('-1');
 await expect(details.locator('[data-stat="ux.sanity"] strong')).toHaveText('-1');
 await page.keyboard.press('Escape');
 await expect(page.locator('.weather-notice')).toHaveCount(0);
 await expect(page.getByRole('button',{name:/Accept conditions|Carry on in these conditions/})).toHaveCount(0);
 await expect.poll(async()=>(await checkpoint(page)).state.journal.length,{timeout:6000}).toBeGreaterThanOrEqual(count+2);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
 const saved=await checkpoint(page);expect(saved.state.weather_impact).toBeTruthy();
 await page.reload();await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
 expect((await checkpoint(page)).state.stats).toEqual(saved.state.stats);
 expect((await checkpoint(page)).state.journal).toEqual(saved.state.journal);
 await snap(page,'weather-continuous');
});
test('weather receipt shows exposure, clear removes stale losses, and help stays in the viewport',async({page})=>{
 const gs=await baseline(page);gs.weather_state.today='HeatWave';gs.weather_impact={day:gs.day,weather:'HeatWave',supplies:-1,hp:-1,sanity:-2,pants:1};
 await importState(page,gs);const details=await weatherDetails(page);await expect(details.locator('[data-stat="ux.health"] strong')).toHaveText('-1');await expect(details.locator('[data-stat="ux.sanity"] strong')).toHaveText('-2');
 await expect(page.locator('.viewport-help')).toContainText('after gear protection');
 await expect(page.locator('.weather-details [data-stat="play.distance"] strong')).toHaveText('-10%');
 const box=await page.locator('.viewport-help').boundingBox();const vp=page.viewportSize()!;expect(box!.x).toBeGreaterThanOrEqual(0);expect(box!.x+box!.width).toBeLessThanOrEqual(vp.width);expect(box!.y+box!.height).toBeLessThanOrEqual(vp.height);
 await page.keyboard.press('Escape');await snap(page,'weather-heat-impact');
 gs.weather_state.today='Clear';gs.weather_impact={day:gs.day,weather:'Clear',supplies:0,hp:0,sanity:0,pants:0};await importState(page,gs);
 await expect(page.locator('.weather-indicator')).toHaveClass(/weather-changed/);const clear=await weatherDetails(page);await expect(clear.locator('section').first().locator('p').first()).toHaveText('No weather cost');await expect(clear.locator('[data-stat="ux.health"],[data-stat="ux.sanity"],[data-stat="ux.supplies"]')).toHaveCount(0);
 await snap(page,'weather-clear-impact');
});
test('old weather pause checkpoints resume without a weather decision or repeated costs',async({page})=>{
 const gs=await baseline(page);gs.weather_state.today='ColdSnap';delete gs.weather_impact;await importState(page,gs);
 await page.evaluate(()=>{const saved=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);saved.weather_notice=true;localStorage.setItem('dystrail.autosave.v1',JSON.stringify(saved));});
 await page.reload();await expect(page.locator('.weather-indicator')).toHaveAttribute('data-weather','ColdSnap');await expect(page.locator('.weather-notice')).toHaveCount(0);
 expect((await checkpoint(page)).state.stats).toEqual(gs.stats);await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeVisible();
 const details=await weatherDetails(page);await expect(details.locator('section').first().locator('p').first()).toHaveText('No weather cost recorded yet.');await expect(details.locator('.weather-explainer')).toHaveText('Applied once per travel day, after gear protection.');await page.keyboard.press('Escape');expect((await checkpoint(page)).state.stats).toEqual(gs.stats);
});
test('weather indicator respects reduced motion and Arabic layout',async({page})=>{
 await page.emulateMedia({reducedMotion:'reduce'});const gs=await baseline(page);gs.weather_state.today='ColdSnap';gs.weather_impact={day:gs.day,weather:'ColdSnap',supplies:0,hp:-1,sanity:-1,pants:0};
 await importState(page,gs);await expect(page.locator('.weather-indicator')).toHaveClass(/weather-changed/);
 expect(await page.locator('.weather-indicator').evaluate(e=>getComputedStyle(e).animationName)).toBe('none');
 await openMenu(page);await page.locator('.language-picker>button').click();await page.getByRole('option',{name:/العربية/}).click();await page.locator('.wordmark').click();
 await expect(page.locator('html')).toHaveAttribute('dir','rtl');const details=await weatherDetails(page);await expect(details).not.toContainText('weather.');await snap(page,'weather-arabic');
});

test('travel actions share one compact row and the scene gets the saved space',async({page})=>{
 await baseline(page);
 const buttons=page.locator('.journey-actions>button');await expect(buttons).toHaveCount(4);
 const boxes=await buttons.evaluateAll(elements=>elements.map(e=>({top:e.getBoundingClientRect().top,bottom:e.getBoundingClientRect().bottom})));
 expect(Math.max(...boxes.map(b=>b.top))-Math.min(...boxes.map(b=>b.top))).toBeLessThan(2);
 await expect(page.locator('.journey-actions').getByRole('button',{name:'Travel',exact:true})).toBeVisible();
 const tabs=page.locator('.journey-controls .travel-tabs');
 await expect(tabs.locator('button')).toHaveText(['The Trail','Conditions','The Van','Journal']);
 await expect(page.locator('.travel-controls .travel-tabs')).toHaveCount(0);
 await expect(page.locator('.speed-controls')).toHaveCount(0);
 await expect(page.getByRole('button',{name:/^(Normal|Step mode)$/})).toHaveCount(0);
 await expect(page.locator('.travel-command')).toHaveCount(0);
 await expect(page.locator('.travel-controls>.context-help')).toHaveCount(0);
 const scene=await page.locator('.journey-scene').boundingBox();expect(scene!.height).toBeGreaterThanOrEqual(440);
 const toolbar=await page.locator('.travel-toolbar').boundingBox();expect(toolbar!.height).toBeLessThanOrEqual(120);
 await snap(page,'travel-actions-compact');
});


test('Fast mode persists, legacy Step saves recover, and toolbar details do not spend a turn',async({page})=>{
 const before=await baseline(page);const toggle=page.getByRole('switch',{name:'Fast mode',exact:true});
 await expect(toggle).toHaveAttribute('aria-checked','false');await toggle.focus();await page.keyboard.press('Space');
 await expect(toggle).toHaveAttribute('aria-checked','true');await expect.poll(async()=>(await checkpoint(page)).travel_speed).toBe('Fast');
 await page.reload();await expect(toggle).toHaveAttribute('aria-checked','true');await expect(page.locator('.travel-screen')).toHaveCSS('--travel-duration','900ms');
 await page.getByRole('tab',{name:'Conditions',exact:true}).click();await expect(page.locator('.daily-settings')).toBeVisible();
 await page.getByRole('tab',{name:'The Van',exact:true}).click();await expect(page.locator('.van-crew')).toBeVisible();await expect(page.locator('.daily-settings')).toHaveCount(0);
 await page.getByRole('tab',{name:'Journal',exact:true}).click();await expect(page.locator('.trail-log')).toBeVisible();await expect(page.locator('.van-crew')).toHaveCount(0);
 await page.getByRole('tab',{name:'The Trail',exact:true}).click();await expect(page.locator('.trail-log')).toHaveCount(0);
 expect((await checkpoint(page)).state.stats).toEqual(before.stats);expect((await checkpoint(page)).state.journal).toEqual(before.journal);
 await page.evaluate(()=>{const saved=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);saved.travel_speed='Step';localStorage.setItem('dystrail.autosave.v1',JSON.stringify(saved));});
 await page.reload();await expect(toggle).toHaveAttribute('aria-checked','false');await expect(page.locator('.travel-screen')).toHaveCSS('--travel-duration','2600ms');
 await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');expect((await checkpoint(page)).state.stats).toEqual(before.stats);
 await expect.poll(async()=>(await checkpoint(page)).travel_speed).toBe('Normal');
 // Even a narrow phone keeps all four actions together without horizontal overflow.
 await page.setViewportSize({width:320,height:740});
 const tops=await page.locator('.journey-actions>button').evaluateAll(es=>es.map(e=>e.getBoundingClientRect().top));expect(Math.max(...tops)-Math.min(...tops)).toBeLessThan(2);
 await snap(page,'travel-toolbar-narrow');
});
