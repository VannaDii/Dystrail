import {test,expect,Page} from '@playwright/test';
import {baseline,importState,savedState,openMenu,snap,fastMode} from './helpers';
import {atTown,routes} from './geography';
const recovery=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
async function quietMap(page:Page) {
 const gs=await baseline(page);gs.day=6;gs.seed=42;gs.rng_bundle=null;gs.encounter_cooldown=100;gs.encounter_chance_today=0;gs.weather_state.neutral_buffer=100;gs.day_state.day_initialized=true;
 gs.stats={...gs.stats,supplies:20,hp:10,sanity:10,morale:10};gs.crew_care.last_check_day=100;gs.route_services.map_reviewed=null;
 await importState(page,gs);return gs;
}
test('the top HUD includes travel resources and repair details remain compact',async({page})=>{
 const gs=await baseline(page);const view=page.locator('.world-view');
 const geometry=await view.evaluate(e=>{const scene=e.querySelector('.journey-scene')!.getBoundingClientRect(),hud=e.querySelector('.resource-hud')!.getBoundingClientRect();return {top:scene.top-hud.bottom};});
 expect(Math.abs(geometry.top)).toBeLessThan(1);await expect(page.locator('.scene-status,.leg-turn')).toHaveCount(0);
 await expect(page.locator('.conditions-hud .leg-cash')).toContainText('Cash');await expect(page.locator('.conditions-hud .leg-vehicle')).toContainText('Vehicle condition');
 await expect(page.locator('.journey-scene .critical-stats')).toHaveCount(0);await expect(page.locator('.journey-scene .conditions-hud')).toBeVisible();
 gs.breakdown={part:'Battery',day_started:gs.day};gs.inventory.spares.battery=1;gs.vehicle.health=93.27;
 await importState(page,gs);await page.getByRole('button',{name:'Fit your spare Battery',exact:true}).click();
 const details=page.locator('.aftermath-panel .resource-changes');await expect(details).toContainText('+6.73%');await expect(details).toContainText('-1');
 const layout=await details.evaluate(e=>{const r=e.getBoundingClientRect(),copy=document.querySelector('.outcome-summary')!.getBoundingClientRect(),panel=e.parentElement!.getBoundingClientRect();return {aside:r.left>=copy.right,below:r.top>=copy.bottom,contained:r.right<=panel.right};});
 expect(layout.aside||layout.below).toBe(true);expect(layout.contained).toBe(true);
 await expect(page.getByRole('button',{name:'Back to the road',exact:true})).toBeInViewport({ratio:1});
 await snap(page,'compact-repair');
});
test('The Trail is exclusive tab content, keyboard accessible, with equal action widths',async({page})=>{
 const gs=await baseline(page);const actions=page.locator('.journey-actions>button');
 const widths=await actions.evaluateAll(es=>es.map(e=>e.getBoundingClientRect().width));expect(Math.max(...widths)-Math.min(...widths)).toBeLessThan(1);expect(Math.max(...widths)).toBeLessThanOrEqual(144);
 await expect(page.getByRole('tabpanel',{name:'The Trail',exact:true})).toContainText(gs.journal.at(-1).message);
 await page.getByRole('tab',{name:'The Trail',exact:true}).focus();await page.keyboard.press('ArrowRight');
 await expect(page.getByRole('tab',{name:'Conditions',exact:true})).toBeFocused();await expect(page.locator('.daily-settings')).toBeVisible();await expect(page.locator('.turn-receipt')).toHaveCount(0);
 await page.keyboard.press('ArrowRight');await expect(page.locator('.van-crew')).toBeVisible();await page.keyboard.press('End');await expect(page.locator('.trail-log')).toBeVisible();
 await page.keyboard.press('Home');await expect(page.locator('.turn-receipt')).toBeVisible();expect((await recovery(page)).state.stats).toEqual(gs.stats);
 await page.evaluate(()=>{const s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);s.state.journal.at(-1).message='The next encounter is ready. You made progress along the route. Encounter!';localStorage.setItem('dystrail.autosave.v1',JSON.stringify(s));});await page.reload();await expect(page.locator('.turn-receipt')).not.toContainText('The next encounter');await expect(page.locator('.turn-receipt')).not.toContainText('Encounter!');await snap(page,'trail-report-tabs');
});
test('automatic map keeps moving and closes in three seconds without a second click',async({page})=>{
 const gs=await quietMap(page);await fastMode(page,false);await page.getByRole('button',{name:'Travel',exact:true}).click();
 const map=page.locator('.map-scene');await expect(map).toHaveAttribute('data-running','true');const opened=await recovery(page);expect(opened.map_automatic).toBe(true);
 await expect.poll(async()=>(await recovery(page)).state.journal.length,{timeout:4000}).toBeGreaterThan(opened.state.journal.length);
 await expect(map).toHaveCount(0,{timeout:4000});await expect(page.getByRole('button',{name:'Pause travel',exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');const end=await recovery(page);expect(end.state.journal.length).toBeGreaterThan(gs.journal.length+2);expect(end.state.route_services.map_reviewed).not.toBeNull();
 await page.reload();await expect(map).toHaveCount(0);expect((await recovery(page)).state.journal).toEqual(end.state.journal);
});
test('automatic map can stay open, survives reload, and resumes with one click',async({page})=>{
 await quietMap(page);await fastMode(page,false);await page.getByRole('button',{name:'Travel',exact:true}).click();await expect(page.locator('.map-countdown')).toBeVisible();
 await page.getByRole('button',{name:'Stay on map',exact:true}).click();await expect(page.locator('.map-countdown')).toHaveCount(0);await page.waitForTimeout(1300);const parked=await recovery(page);
 await page.waitForTimeout(5200);expect((await recovery(page)).state.journal).toEqual(parked.state.journal);await expect(page.locator('.map-scene')).toBeVisible();await snap(page,'map-stay');
 await page.reload();await expect(page.locator('.map-scene')).toHaveAttribute('data-running','false');
 await page.getByRole('button',{name:'Resume travel',exact:true}).click();await expect(page.getByRole('button',{name:'Pause travel',exact:true})).toBeVisible();await expect(page.locator('.map-scene')).toHaveCount(0);
 await expect.poll(async()=>(await recovery(page)).state.journal.length).toBeGreaterThan(parked.state.journal.length);
});
test('manual map stays open and resumed travel stops at the next town',async({page})=>{
 const gs=await quietMap(page);await page.getByRole('button',{name:'Route',exact:true}).click();await expect(page.locator('.map-countdown')).toHaveCount(0);await page.waitForTimeout(5300);await expect(page.locator('.map-scene')).toHaveAttribute('data-automatic','false');expect((await recovery(page)).state.journal).toEqual(gs.journal);
 await page.getByRole('button',{name:'Close map',exact:true}).click();await expect(page.getByRole('button',{name:'Pause travel',exact:true})).toHaveCount(0);await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeEnabled();expect((await recovery(page)).state.journal).toEqual(gs.journal);
 // Put the running map just before a known real town; town arrival must interrupt it.
 atTown(gs,'La Crosse');gs.miles_traveled_actual-=1.1/routes.find((r:any)=>r.id==='journalist')!.total_miles*gs.trail_distance;gs.miles_traveled=Math.round(gs.miles_traveled_actual);gs.route_services.stop=null;gs.route_services.map_reviewed=null;gs.prev_miles_traveled=gs.miles_traveled_actual;gs.day_state.day_initialized=false;
 // The two earlier crossings are already behind this imported position.
 gs.crossings_completed=2;
 await importState(page,gs);await page.evaluate(()=>{const s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);s.phase='Map';s.map_automatic=true;localStorage.setItem('dystrail.autosave.v1',JSON.stringify(s));});await page.reload();await page.getByRole('button',{name:'Resume travel',exact:true}).click();
 await expect(page.locator('.town-arrival')).toBeVisible({timeout:12000});await expect(page.locator('.map-scene')).toHaveCount(0);
});
test('camp gathering uses cooldown cards, persists and becomes available again',async({page})=>{
 const gs=await baseline(page);gs.stats.supplies=8;await importState(page,gs);await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toHaveCount(0);await page.getByRole('button',{name:'Camp',exact:true}).click();for(const name of ['Rest','Forage with a local guide','Help glean a nearby farm'])await expect(page.locator('.camp-actions').getByRole('button',{name,exact:true})).toHaveCount(1);await expect(page.locator('.camp-actions').getByRole('button',{name:'Forage',exact:true})).toHaveCount(0);await expect(page.locator('.camp-gather .cooldown-status').first()).toContainText('Ready');
 await page.getByRole('button',{name:'Help glean a nearby farm',exact:true}).click();const used=await savedState(page);expect(used.stats.supplies).toBe(12);expect(used.stats.hp).toBe(gs.stats.hp-1);expect(used.clock_minutes).toBe(gs.clock_minutes+120);
 await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('.camp-gather button').first()).toBeDisabled();await page.reload();await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toBeDisabled();await expect(page.locator('.camp-gather').first()).toContainText('3 travel days');await snap(page,'camp-cooldowns');
 used.day+=2;await importState(page,used);await page.getByRole('button',{name:'Camp',exact:true}).click();await expect(page.locator('.camp-gather').first()).toContainText('1 travel days');await expect(page.locator('.camp-gather progress').first()).toHaveAttribute('value','2');
 used.day+=1;await importState(page,used);await page.getByRole('button',{name:'Camp',exact:true}).click();await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toBeEnabled();
 atTown(used,'La Crosse');await importState(page,used);await expect(page.getByRole('button',{name:'Take a paid unloading shift',exact:true})).toBeVisible();await expect(page.getByRole('button',{name:'Forage with a local guide',exact:true})).toHaveCount(0);
});
test('help preference hides only optional tips, persists offline, and keeps information usable',async({page,context})=>{
 await baseline(page);await openMenu(page);const toggle=page.getByRole('switch',{name:'Help & tips',exact:true});await expect(toggle).toHaveAttribute('aria-checked','true');
 const layout=()=>page.evaluate(()=>({scroll:scrollY,boxes:[...document.querySelectorAll('.survival-hud, .hud-stat dt, .hud-stat dd, .journey-scene, .scene-status, .travel-toolbar, #main>footer, #game-menu-panel, .offline-status')].map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};})}));
 const before=await layout();
 await toggle.click();await expect(page.locator('[data-help-kind=tip]:visible')).toHaveCount(0);expect(await layout()).toEqual(before);await expect(page.getByRole('button',{name:'How this works: Supplies',exact:true})).toHaveCount(0);await expect(page.locator('[data-help-kind=info]')).not.toHaveCount(0);await expect(page.getByRole('button',{name:'How to install',exact:true})).toBeVisible();await snap(page,'help-off');
 await page.locator('.wordmark').click();await page.locator('.weather-indicator button').click();await expect(page.locator('.viewport-help')).toBeVisible();await page.keyboard.press('Escape');
 await context.setOffline(true);await page.reload();await expect(page.locator('#main')).toBeVisible();await expect(page.locator('[data-help-kind=tip]:visible')).toHaveCount(0);await openMenu(page);await expect(toggle).toHaveAttribute('aria-checked','false');await toggle.focus();const offlineBefore=await layout();await page.keyboard.press('Space');await expect(toggle).toHaveAttribute('aria-checked','true');await expect(page.locator('[data-help-kind=tip]:visible')).not.toHaveCount(0);expect(await layout()).toEqual(offlineBefore);
});

test('critical sanity stays in the stat bar with a bounded pulse and no popup',async({page})=>{
 const gs=await baseline(page);gs.stats.sanity=2;await importState(page,gs);await expect(page.locator('.danger-notice')).toHaveCount(0);
 const stat=page.locator('.hud-stat').filter({has:page.getByRole('button',{name:'How this works: Sanity',exact:true})});await expect(stat).toHaveClass(/critical/);await expect(stat).toHaveCSS('animation-duration','1s');await expect(stat).toHaveCSS('animation-iteration-count','3');await snap(page,'critical-stat-only');
 await page.emulateMedia({reducedMotion:'reduce'});await expect(stat).toHaveCSS('animation-name','none');
});

test('a queued crew decision waits for the automatic map without advancing past it',async({page})=>{
 const gs=await quietMap(page);gs.day=7;gs.clock_minutes=660;gs.crew_care.last_check_day=3;await importState(page,gs);await fastMode(page,false);
 await page.getByRole('button',{name:'Travel',exact:true}).click();await expect(page.locator('.map-countdown')).toBeVisible();
 await expect.poll(async()=>(await recovery(page)).state.crew_care.pending,{timeout:3500}).not.toBeNull();const queued=await recovery(page);await expect(page.locator('.map-scene')).toBeVisible();await page.waitForTimeout(1000);expect((await recovery(page)).state.journal).toEqual(queued.state.journal);await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care',{timeout:6500});await expect(page.locator('.map-scene')).toHaveCount(0);await expect(page.getByRole('button',{name:'Treat them',exact:true})).toBeVisible();
});
