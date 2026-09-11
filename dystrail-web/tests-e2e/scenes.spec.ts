import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {baseline,importState,snap,openMenu} from './helpers';
import {atTown} from './geography';
const encounters=JSON.parse(readFileSync(join(__dirname,'../static/assets/data/game.json'),'utf8'));
test('encounters have matching settings and never inherit a road backdrop',async({page})=>{
 const state=await baseline(page);
 for(const [id,asset] of [['classic_media_training','enc-media-workshop'],['classic_mutual_aid','enc-community'],['clinic_triage','enc-clinic'],['overnight_briefing','enc-night-briefing']]) {
  state.current_encounter=encounters.find((e:{id:string})=>e.id===id);
  await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',asset);
  await expect(page.locator('.crew-van')).toHaveCount(0);
  await expect(page.locator('.game-clock')).toContainText('08:00');
  await snap(page,id);
 }
});
test('regional roads follow route geography, full crew van travels, heat haze and clock are synchronized',async({page})=>{
 const state=await baseline(page);
 for(const [town,asset] of [['Minneapolis','open-heartland-prairie'],['Madison','open-heartland-orchard'],['Chicago','open-rustbelt-foundry'],['Toledo','open-rustbelt-lakeside'],['Pittsburgh','open-appalachian-ridge'],['Frederick','open-beltway-suburbs']] as const) {
  atTown(state,town);state.route_services.stop=null;state.current_encounter=null;state.day=2;
  await importState(page,state);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',asset);
  state.day=3;await importState(page,state);await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene',asset);
  await expect(page.locator('.crew-van .van-body')).toHaveAttribute('src',/van-empty/);await snap(page,asset);
 }
 atTown(state,'Madison');state.route_services.stop=null;state.weather_state.today='HeatWave';await importState(page,state);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-weather','heat');
 expect(await page.locator('.weather-atmosphere').evaluate(el=>getComputedStyle(el).animationName)).toBe('heat-shimmer');await snap(page,'heat-wave');
 state.weather_state.today='Clear';await importState(page,state);
 await page.getByRole('button',{name:'Resume travel',exact:true}).click();
 await expect(page.locator('.scene-moving .crew-van')).toBeVisible();
 expect(await page.locator('.crew-van').evaluate(el=>getComputedStyle(el).animationName)).toBe('van-suspension');
 await expect(page.locator('.scene-moving')).toHaveAttribute('data-time','morning');
 await expect(page.locator('.game-clock')).toContainText('08:00');
 await snap(page,'travel-dusk');
});
test('menu controls, pointer cursors and help stay inside viewport at either edge',async({page})=>{
 await baseline(page);await page.getByRole('button',{name:'Assess conditions',exact:true}).click();await openMenu(page);
 await expect(page.getByRole('switch')).toBeVisible();await expect(page.getByRole('button',{name:'Save',exact:true})).toBeVisible();
 await page.keyboard.press('Escape');await expect(page.locator('#game-menu-panel')).toHaveCount(0);await expect(page.locator('#game-menu-button')).toBeFocused();
 for(const label of ['How this works: Supplies','How this works: Allies','How this works: Weather effects']){
  const trigger=page.getByRole('button',{name:label,exact:true});await trigger.click();
  const help=page.locator('.viewport-help');await expect(help).toBeVisible();
  const box=await help.boundingBox();const size=page.viewportSize()!;
  expect(box!.x).toBeGreaterThanOrEqual(10);expect(box!.y).toBeGreaterThanOrEqual(10);
  expect(box!.x+box!.width).toBeLessThanOrEqual(size.width-10);expect(box!.y+box!.height).toBeLessThanOrEqual(size.height-10);
  await page.screenshot({path:`test-results/help-${label.split(': ')[1]}-${test.info().project.name}.png`});
  await page.evaluate(()=>window.scrollTo(0,0));await expect(help).toBeVisible();
  await expect.poll(async()=>{const b=await help.boundingBox();return b!==null && b.y>=10 && b.y+b.height<=size.height-10;}).toBe(true);
  await page.keyboard.press('Escape');await expect(help).toHaveCount(0);
 }
 for(const selector of ['#game-menu-button','.setting-option strong','.help-trigger'])expect(await page.locator(selector).first().evaluate(el=>getComputedStyle(el).cursor)).toBe('pointer');
});
