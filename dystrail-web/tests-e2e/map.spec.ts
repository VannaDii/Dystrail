import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';import {join} from 'node:path';
import {setup,depart,baseline,importState,savedState,snap,openMenu} from './helpers';import {atTown,routes} from './geography';
const encounters=JSON.parse(readFileSync(join(__dirname,'../static/assets/data/game.json'),'utf8'));
test('map is a periodic full scene with real geography and exact recovery',async({page})=>{
 await setup(page);await depart(page);await expect(page.locator('.map-scene')).toHaveCount(0);
 await page.getByRole('button',{name:'Review route',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','map');await expect(page.locator('.world-view')).toHaveCount(0);
 await expect(page.locator('.us-route-map')).toHaveAttribute('data-route','journalist');await expect(page.locator('.map-scene-itinerary')).toContainText('La Crosse');
 await expect(page.locator('.route-traced')).toHaveAttribute('points',/\d/);await expect(page.getByRole('button',{name:'Zoom to route',exact:true})).toHaveCount(0);expect(await page.locator('.us-route-map').getAttribute('viewBox')).not.toBe('0 0 1000 660');
 await snap(page,'map-us');await page.reload();await expect(page.locator('#main')).toHaveAttribute('data-screen','map');
 await snap(page,'map-route');
 await page.getByRole('button',{name:'Continue the journey',exact:true}).click();await expect(page.locator('.map-scene')).toHaveCount(0);
 const dismissed=await savedState(page);expect(dismissed.route_services.map_reviewed).not.toBeNull();
 await page.reload();await expect(page.locator('.map-scene')).toHaveCount(0);

});
test('real towns, regional scenes and encounter geography agree',async({page})=>{
 const state=await baseline(page);atTown(state,'Chicago');await importState(page,state);
 await expect(page.locator('.world-view')).toHaveAttribute('data-region','RustBelt');await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','town-arrival');
 await expect(page.locator('.route-stop h2')).toHaveText('Chicago');await expect(page.locator('.scene-location')).toHaveText('At Chicago');
 state.current_encounter=encounters.find((e:{id:string})=>e.id==='classic_mutual_aid');await importState(page,state);
 await expect(page.locator('.scene-location')).toHaveText('At Chicago');await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','enc-community');await snap(page,'geographic-encounter');
 state.current_encounter=null;atTown(state,'Pittsburgh');state.route_services.stop=null;await importState(page,state);
 await expect(page.locator('.scene-location')).toHaveText('Pittsburgh → Cumberland');await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','open-appalachian-ridge');
 atTown(state,'Frederick');await importState(page,state);await expect(page.locator('.world-view')).toHaveAttribute('data-region','Beltway');await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','town-arrival');await snap(page,'geographic-beltway');
});
test('every persona previews and saves a distinct geographic origin',async({page})=>{
 await page.goto('./');await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 for(const route of routes){const name=route.id[0].toUpperCase()+route.id.slice(1);await page.getByRole('radio',{name,exact:true}).click();await expect(page.locator('.persona-origin')).toHaveText(`Starts in ${route.stops[0].name} · Bound for D.C.`);}
 await page.getByRole('button',{name:'Continue',exact:true}).click();await page.getByLabel('Your name',{exact:true}).fill('Alex');await page.getByLabel('Crew name',{exact:true}).fill('Paper Tigers');await page.getByRole('button',{name:'Continue',exact:true}).click();await depart(page);
 const state=await savedState(page);expect(state.route_services.route_id).toBe('satirist');await page.reload();expect((await savedState(page)).route_services.route_id).toBe('satirist');await expect(page.locator('.scene-location')).toContainText('Austin');
});

test('geographic scene remains readable in Arabic and reduced motion',async({page})=>{
 await page.emulateMedia({reducedMotion:'reduce'});await baseline(page);
 await page.getByRole('button',{name:'Review route',exact:true}).click();
 await openMenu(page);await page.locator('.language-picker>button').click();await page.getByRole('option',{name:'العربية',exact:true}).click();await page.locator('.wordmark').click();
 await expect(page.locator('html')).toHaveAttribute('dir','rtl');await expect(page.locator('.us-route-map')).toHaveAttribute('direction','ltr');
 await expect(page.locator('.map-scene')).not.toContainText(/route\.|\{day\}/);expect(await page.locator('.route-traced').evaluate(el=>getComputedStyle(el).animationName)).toBe('none');
 await snap(page,'map-rtl');await page.locator('.map-scene-actions button').click();await expect(page.locator('.map-scene')).toHaveCount(0);
});
