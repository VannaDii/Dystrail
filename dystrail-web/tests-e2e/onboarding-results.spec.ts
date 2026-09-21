import {test,expect} from '@playwright/test';
import {baseline,importState,snap,waitForLaunch} from './helpers';
import {atTown} from './geography';
import {readFileSync} from 'node:fs';

const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
test('start action shares the Deep End column and stacks on mobile',async({page,isMobile})=>{
 await page.goto('./');await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','setup');
 const geometry=await page.evaluate(()=>{const mode=document.querySelectorAll('.mode-option')[1].getBoundingClientRect(),code=document.querySelector('.seed-entry')!.getBoundingClientRect(),button=document.querySelector('.setup-start-row>button')!.getBoundingClientRect();return {button:{x:button.x,y:button.y,width:button.width},mode:{x:mode.x,width:mode.width},code:{x:code.x,bottom:code.bottom,width:code.width}};});
 if(isMobile){expect(geometry.button.y).toBeGreaterThanOrEqual(geometry.code.bottom);expect(Math.abs(geometry.button.width-geometry.code.width)).toBeLessThan(1);}else{expect(Math.abs(geometry.button.x-geometry.mode.x)).toBeLessThan(1);expect(geometry.button.x).toBeGreaterThan(geometry.code.x);}
 await snap(page,'setup-action-alignment');
});
test('random character persists, selection announces without duplicate footer text, and compact crew busts have no added frame or background',async({page})=>{
 await page.goto('./');await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 await expect(page.locator('.persona-tile[aria-checked=true]')).toHaveCount(1);const initial=(await checkpoint(page)).pending.persona_id;expect(initial).toBeTruthy();
 await expect(page.locator('#persona-continue')).toBeEnabled();await expect(page.locator('.journey-mission')).toContainText('public hearing in D.C.');await expect(page.locator('#persona-preview')).not.toContainText('public hearing');
 await page.reload();await waitForLaunch(page);await expect.poll(async()=>(await checkpoint(page)).pending.persona_id).toBe(initial);expect(await page.evaluate(()=>document.activeElement?.tagName)).toBe('BODY');
 for(const name of ['Journalist','Organizer','Whistleblower','Lobbyist','Staffer','Satirist']){
  await page.getByRole('radio',{name,exact:true}).click();await expect(page.locator('#persona-helper')).toContainText(name);
  await expect(page.locator('#persona-helper')).toHaveClass('sr-only');await expect(page.locator('.persona-actions #persona-helper')).toHaveCount(0);await expect(page.locator('.persona-actions button:visible')).toHaveCount(2);
 }
 await snap(page,'character-details');await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('.crew-name-card')).toHaveCount(6);
 for(const portrait of await page.locator('.crew-name-card .crew-portrait').all()){
  const g=await portrait.evaluate(e=>{const s=getComputedStyle(e),r=e.getBoundingClientRect(),card=e.parentElement!.getBoundingClientRect();return {w:r.width,h:r.height,bg:s.backgroundColor,border:s.borderWidth,card:card.width};});
  expect(g.bg).toBe('rgba(0, 0, 0, 0)');expect(g.border).toBe('0px');expect(g.w).toBeGreaterThan(54);expect(g.w).toBeLessThan(90);expect(g.h).toBeLessThan(100);
 }
 await expect(page.locator('.crew-intro')).not.toContainText('gender-neutral');await snap(page,'crew-compact-busts');
});
test('early illness ending matches location and primary player, shows the full scorecard and updates in place',async({page})=>{
 let gs=await baseline(page);
 if(process.env.PLAYER_RESULT_FIXTURE){gs=JSON.parse(readFileSync(process.env.PLAYER_RESULT_FIXTURE,'utf8'));}
 else {
  gs.persona_id='whistleblower';gs.party.leader='Willow';gs.party.members.find((m:any)=>m.persona==='whistleblower').name='Willow';gs.party.members.find((m:any)=>m.persona==='whistleblower').status='Dead';
  gs.trail_distance=2400;gs.day=37;gs.clock_minutes=540;gs.miles_traveled_actual=501.7949;gs.miles_traveled=502;gs.ending={type:'collapse',cause:'disease'};gs.stats.sanity=0;
  gs.route_services={...gs.route_services,route_id:'whistleblower',stop:null};gs.journal.push({day:37,minute:540,place:'Reno → Salt Lake City',title:'A crew member needs help · Willow',message:'Willow died after critical illness went untreated. The crew continues without them.',before:gs.stats,after:gs.stats,details:[]});
 }
 await importState(page,gs);await expect(page.locator('#result-title')).toHaveText('ILLNESS ENDS THE JOURNEY');await expect(page.locator('.result-location')).toContainText('Reno → Salt Lake City');await expect(page.locator('.result-location')).toContainText('21%');
 const player=gs.party.members.find((m:any)=>m.persona===gs.persona_id).name;
 await expect(page.locator('.result-art .result-profile')).toContainText(player);await expect(page.locator('.result-profile .character-art')).toHaveAttribute('data-expression','defeated');expect(await page.locator('.result-profile image').getAttribute('href')).toBe(await page.evaluate(()=>(window as any).dystrailAssetUrls['static/img/journey/occupant-whistleblower-expressions-v1.png']));await expect(page.locator('.result-art .scene-speaker')).toHaveCount(0);await expect(page.locator('.result-art .journey-scene')).toHaveAttribute('data-scene','ending-rest-area');
 await expect(page.locator('.ending-scorecard')).toBeVisible();await expect(page.locator('details.ending-scorecard')).toHaveCount(0);await expect(page.locator('.ending-moments')).not.toContainText('The crew continues without them');await expect(page.locator('.result-screen')).not.toContainText('steps');await snap(page,'illness-ending');
 const sameDay=structuredClone(gs);sameDay.party.members.find((m:any)=>m.persona===gs.persona_id).name='Reese';sameDay.continuity=undefined;sameDay.abandoned=true;
 await importState(page,sameDay);await expect(page.locator('.result-profile')).toContainText('Reese');await expect(page.locator('#result-title')).toHaveText('The trail ends here');
});
test('bottom result actions give visible feedback without a page jump',async({page,context})=>{
 await context.grantPermissions(['clipboard-read','clipboard-write']);
 const gs=await baseline(page);gs.abandoned=true;await importState(page,gs);
 const copy=page.getByRole('menuitem',{name:'5 Export Save',exact:true});await copy.scrollIntoViewIfNeeded();const before=await page.evaluate(()=>({y:scrollY,header:document.querySelector('.game-header')!.getBoundingClientRect().height}));
 await copy.click();await expect(page.locator('.action-feedback').filter({hasText:'Game export copied to clipboard.'})).toBeVisible();
 const after=await page.evaluate(()=>({y:scrollY,header:document.querySelector('.game-header')!.getBoundingClientRect().height,feedback:[...document.querySelectorAll('.action-feedback')].filter(e=>e.textContent).map(e=>{const r=e.getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight;})}));
 expect(Math.abs(after.y-before.y)).toBeLessThan(2);expect(after.header).toBe(before.header);expect(after.feedback.every(Boolean)).toBe(true);await snap(page,'result-copy-feedback');
});

test('a fatal player care decision reports the death and ends the run here',async({page})=>{
 const gs=await baseline(page);gs.stats.sanity=1;gs.crew_care.pending=gs.persona_id;gs.crew_care.strain={[gs.persona_id]:3};await importState(page,gs);
 await page.locator('.crew-incident .camp-actions>button').nth(2).click();await expect(page.locator('.aftermath-panel')).toContainText('Vanna Test dies. The journey ends.');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('#result-title')).toHaveText('ILLNESS ENDS THE JOURNEY');
 await expect(page.locator('.result-art .result-profile')).toContainText('Vanna Test');const saved=(await checkpoint(page)).state;expect(saved.ending).toEqual({type:'collapse',cause:'disease'});expect(saved.party.members.find((m:any)=>m.persona===gs.persona_id).status).toBe('Dead');
});

test('town names omit universal service icons while town actions remain accessible',async({page})=>{
 const gs=await baseline(page);await expect(page.locator('.service-indicators')).toHaveCount(0);
 const metrics=await page.locator('.conditions-hud').evaluate(el=>['.leg-destination','.leg-cash','.leg-vehicle'].map(selector=>{const r=el.querySelector(selector)!.getBoundingClientRect();return {top:r.top,height:r.height,left:r.left,right:r.right};}));
 expect(Math.max(...metrics.map(m=>m.height))-Math.min(...metrics.map(m=>m.height))).toBeLessThan(1);
 for(const m of metrics){expect(m.left).toBeGreaterThanOrEqual(0);expect(m.right).toBeLessThanOrEqual(page.viewportSize()!.width);}
 await expect(page.locator('.scene-status,.leg-turn')).toHaveCount(0);
 await snap(page,'plain-next-town');
 await page.getByRole('button',{name:'Route',exact:true}).click();await expect(page.locator('.map-scene-itinerary ol li')).toHaveCount(2);await expect(page.locator('.map-scene-itinerary button')).toHaveCount(0);await snap(page,'plain-map-towns');
 atTown(gs,'Spokane');await importState(page,gs);await expect(page.getByRole('button',{name:'Trade with locals',exact:true})).toBeEnabled();await expect(page.getByRole('button',{name:'Visit the Store',exact:true})).toBeEnabled();await expect(page.locator('.service-indicators')).toHaveCount(0);await snap(page,'plain-town-services');
});
