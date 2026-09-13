import {test,expect,Page} from '@playwright/test';
import {baseline,importState,snap,fastMode,openMenu} from './helpers';
import {atTown,routes} from './geography';
const saved=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);

test('illustrated van and compact journal preserve every count and fit desktop and phone',async({page,isMobile})=>{
 const gs=await baseline(page);gs.inventory.spares={tire:2,battery:0,alt:1,pump:3};gs.inventory.tags=['plague_resist','permit'];
 const before={...gs.stats},after={...gs.stats,supplies:gs.stats.supplies-2,morale:gs.stats.morale+1};
 gs.journal=[{day:129,minute:540,pace:'steady',diet:'mixed',place:'At Iowa City',title:'A crew member needs help · Sage',message:'Sage receives care and can help the crew again.',action_kind:'care',before,after,resources:[],details:[['Sage','Traveling']]}];
 await importState(page,gs);await page.getByRole('tab',{name:'The Van',exact:true}).click();
 await expect(page.locator('.van-item')).toHaveCount(9);await expect(page.locator('.van-crew-member')).toHaveCount(6);
 await expect(page.getByRole('heading',{name:'Trip overview',exact:true})).toBeVisible();await expect(page.locator('.van-overview .stat-card-art')).toHaveCount(4);
 const crewBounds=await page.locator('.van-crew').boundingBox(),equipmentBounds=await page.locator('.van-equipment-grid').boundingBox();
 expect(crewBounds!.y+crewBounds!.height).toBeLessThan(equipmentBounds!.y);
 for(const [id,count] of Object.entries({spare_tire:'2',battery:'0',alternator:'1',fuel_pump:'3'}))await expect(page.locator(`[data-item=${id}] .van-item-count`)).toHaveText(count);
 await expect(page.locator('[data-item=masks]')).toContainText('Carried');await expect(page.locator('[data-item=coats]')).toContainText('Not carried');
 expect(await page.locator('.van-inventory img').evaluateAll(es=>es.every(e=>{const i=e as HTMLImageElement;return i.complete&&i.naturalWidth>0&&i.src.startsWith('blob:');}))).toBe(true);
 await snap(page,'illustrated-van');
 await page.getByRole('tab',{name:'Journal',exact:true}).click();const entry=page.locator('.journal-day'),daily=entry.locator(':scope > .journal-story');
 await expect(entry).toHaveCount(1);await expect(daily.getByRole('heading',{name:'Day 129',exact:true})).toBeVisible();
 await expect(daily.locator('.stat-card')).toHaveCount(2);await expect(daily.locator('[data-stat="ux.supplies"] small')).toHaveText(`${after.supplies} left`);
 await expect(daily).toContainText('SageTraveling');
 if(!isMobile){expect((await entry.boundingBox())!.height).toBeLessThan(220);const positions=await entry.evaluate(e=>{const a=e.querySelector('.journal-narrative')!.getBoundingClientRect(),b=e.querySelector('.resource-changes')!.getBoundingClientRect();return {a:a.right,b:b.left};});expect(positions.b).toBeGreaterThan(positions.a);}
 await snap(page,'compact-journal-desktop');expect((await saved(page)).stats).toEqual(gs.stats);
});

test('status icons, weather detail, help spacing, and selected pace and diet stay synchronized',async({page})=>{
 const gs=await baseline(page);gs.weather_state.today='HeatWave';gs.weather_impact={day:gs.day,weather:'HeatWave',supplies:-1,hp:-1,sanity:-2};gs.day_state.day_initialized=false;
 await importState(page,gs);await page.getByRole('tab',{name:'Conditions',exact:true}).click();
 await expect(page.locator('.setting-option .journey-icon')).toHaveCount(6);await expect(page.locator('.weather-indicator')).toHaveCount(1);
 await expect(page.locator('.conditions-hud [data-icon=clock]')).toHaveCount(1);
 await page.locator('.setting-option').filter({has:page.locator('[data-icon=blitz]')}).click();
 await page.locator('.setting-option').filter({has:page.locator('[data-icon=quiet]')}).click();
 await expect(page.locator('.hud-journey-settings')).toContainText('Blitz');await expect(page.locator('.hud-journey-settings')).toContainText('Quiet');
 await page.locator('.weather-indicator .help-trigger').click();await expect(page.locator('.weather-details')).toBeVisible();
 await expect(page.locator('.weather-details .stat-card')).toHaveCount(5);await expect(page.locator('.weather-details')).toContainText('after gear protection');
 await expect(page.locator('.weather-indicator .info-glyph')).toHaveText('i');await expect(page.locator('.help-trigger')).not.toContainText(['ⓘ']);
 const bounds=await page.locator('.viewport-help').boundingBox(),vp=page.viewportSize()!;expect(bounds!.x).toBeGreaterThanOrEqual(0);expect(bounds!.x+bounds!.width).toBeLessThanOrEqual(vp.width);expect(bounds!.y+bounds!.height).toBeLessThanOrEqual(vp.height);
 await snap(page,'structured-weather');await page.keyboard.press('Escape');
 await openMenu(page);const geometry=()=>page.locator('.world-view').evaluate(e=>[...e.querySelectorAll('.hud-context-row,.critical-stats,.scene-status')].map(e=>{const r=e.getBoundingClientRect();return [r.x,r.y,r.width,r.height];}));
 const before=await geometry();await page.getByRole('switch',{name:'Help & tips',exact:true}).click();expect(await geometry()).toEqual(before);
 await page.locator('#game-menu-button').click();
 const state=await saved(page);expect(state.stats).toEqual(gs.stats);expect(state.pace).toBe('blitz');expect(state.diet).toBe('quiet');
 await expect(page.locator('.leg-cash')).toContainText('Cash');await expect(page.locator('.leg-vehicle')).toContainText('Vehicle condition');
 await snap(page,'status-pictograms');
});

test('repair outcome and its journal use stat cards with remaining parts',async({page})=>{
 const gs=await baseline(page);gs.breakdown={part:'Battery',day_started:gs.day};gs.inventory.spares.battery=2;gs.vehicle.health=93.27;
 await importState(page,gs);await expect(page.locator('.journey-actions [data-icon=travel]')).toBeVisible();await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Fit your spare Battery',exact:true}).click();
 await expect(page.locator('#aftermath-title')).toHaveText('Outcome');
 const changes=page.locator('.aftermath-panel .resource-changes');await expect(changes.locator('[data-stat="play.vehicle"] strong')).toHaveText('+6.73%');
 await expect(changes.locator('[data-stat="store.items.battery.name"] strong')).toHaveText('-1');await expect(changes.locator('[data-stat="store.items.battery.name"] small')).toHaveText('1 left');
 await snap(page,'boxed-repair-outcome');await page.getByRole('button',{name:'Back to the road',exact:true}).click();
 await page.getByRole('tab',{name:'Journal',exact:true}).click();const repairDay=page.locator('.journal-day').first();
 await expect(repairDay.locator(':scope > .journal-story > .resource-changes [data-stat="play.vehicle"] strong')).toHaveText('+6.73%');
 await repairDay.locator(':scope > .journal-raw > summary').click();const repairEntry=repairDay.locator('.journal-raw-entry').last();
 await expect(repairEntry.locator('.stat-card')).toHaveCount(2);await expect(repairEntry.locator('[data-stat="store.items.battery.name"] strong')).toHaveText('-1');await expect(repairEntry.locator('[data-stat="store.items.battery.name"] small')).toHaveText('1 left');
});

test('local conversations offer all three benefits, are repeatable, and never award twice',async({page})=>{
 const original=await baseline(page);const route=routes.find((r:any)=>r.id===original.persona_id);
 for(const kind of [0,1,2]){
  const town=route.stops.find((s:any)=>Math.floor(s.mile/10)%3===kind&&s.name!=='D.C.');const gs=structuredClone(original);atTown(gs,town.name);gs.stats.credibility=10;gs.stats.allies=2;
  await importState(page,gs);const label=['Credibility','Receipts','Allies'][kind],button=page.getByRole('button',{name:'Talk to locals',exact:true});
  await expect(button).toContainText(`${label} +1`);await expect(button.locator('s')).toHaveCount(0);await button.click();
  const once=await saved(page);expect(once.stats.credibility-gs.stats.credibility).toBe(kind===0?1:0);expect(once.receipts.length-gs.receipts.length).toBe(kind===1?1:0);expect(once.stats.allies-gs.stats.allies).toBe(kind===2?1:0);
  await page.getByRole('button',{name:'Back to town',exact:true}).click();await expect(button.locator('s')).toHaveText(`${label} +1`);await expect(button).toBeEnabled();
  await page.reload();await button.click();const twice=await saved(page);twice.inventory.tags.sort();once.inventory.tags.sort();expect(twice).toEqual({...once,activities:{...once.activities,local_word:expect.any(Number)}});
  await page.getByRole('button',{name:'Back to town',exact:true}).click();
 }
});

test('one town action leaves town, normal maps last three seconds, Fast mode skips them',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Spokane');gs.day=6;gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;gs.weather_state.neutral_buffer=100;
 await importState(page,gs);await expect(page.getByRole('button',{name:'Leave town',exact:true})).toHaveCount(1);await expect(page.locator('.town-depart')).toHaveCount(0);
 await fastMode(page,false);await page.getByRole('button',{name:'Leave town',exact:true}).click();await expect(page.locator('.map-countdown')).toContainText('3');
 const started=Date.now();await expect(page.locator('.map-scene')).toHaveCount(0,{timeout:4200});const elapsed=Date.now()-started;expect(elapsed).toBeGreaterThan(1700);expect(elapsed).toBeLessThan(3900);
 await expect(page.getByRole('button',{name:'Pause travel',exact:true})).toBeVisible();await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await importState(page,gs);await fastMode(page,true);await page.getByRole('button',{name:'Visit the Store',exact:true}).click();
 await page.getByRole('spinbutton',{name:'Rations Pack Quantity',exact:true}).fill('1');await page.getByRole('button',{name:'Leave town',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');expect((await saved(page)).budget_cents).toBe(gs.budget_cents);await expect(page.locator('.map-scene')).toHaveCount(0);
 await page.waitForTimeout(1800);await expect(page.locator('.map-scene')).toHaveCount(0);
});

test('offline share creates a real image and editable post without sending it',async({page,context})=>{
 const gs=await baseline(page);gs.abandoned=true;await importState(page,gs);await context.setOffline(true);
 await page.locator('#result-share-open').click();const dialog=page.getByRole('dialog');await expect(dialog).toBeVisible();const image=dialog.locator('.share-preview img');await expect(image).toBeVisible();
 expect(await image.evaluate((img:HTMLImageElement)=>[img.naturalWidth,img.naturalHeight])).toEqual([1200,1200]);
 const content=page.locator('#share-post');await expect(content).toHaveValue(/Vanna Test/);await content.fill('My trip — 1234 & a story');await expect(dialog).toBeVisible();
 await expect(dialog.getByRole('link',{name:'Bluesky',exact:true})).toHaveAttribute('href',/text=My%20trip/);
 const [download]=await Promise.all([page.waitForEvent('download'),dialog.getByRole('link',{name:'Save image',exact:true}).click()]);expect(download.suggestedFilename()).toBe('dystopian-trail.png');await download.saveAs(test.info().outputPath('share.png'));
 await snap(page,'offline-share-composer');await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await expect(page.locator('#result-share-open')).toBeFocused();
});
