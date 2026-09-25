import {test,expect,Page} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import { fastMode,baseline,importState,savedState,openMenu,snap,setup, waitForLaunch } from './helpers';
import {routes} from './geography';
const encounters=JSON.parse(readFileSync(join(__dirname,'../static/assets/data/game.json'),'utf8'));
const recovery=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));

test('abandon is cancelable, saves a crew ending, and leaves the manual slot intact',async({page})=>{
 const before=await baseline(page);
 await openMenu(page);await page.getByRole('button',{name:'Abandon trail',exact:true}).click();
 await expect(page.getByRole('dialog',{name:'Abandon this trail?'})).toBeVisible();
 await expect(page.getByRole('button',{name:'Keep traveling',exact:true})).toBeFocused();
 await page.keyboard.press('Escape');await expect(page.locator('.abandon-dialog')).toHaveCount(0);
 expect((await recovery(page)).state.stats).toEqual(before.stats);
 await openMenu(page);await page.getByRole('button',{name:'Abandon trail',exact:true}).click();
 await page.getByRole('button',{name:'End this journey',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
 await expect(page.locator('#main')).toContainText('The trail ends here');
 await expect(page.locator('.ending-crew li')).toHaveCount(6);
 const ended=(await recovery(page)).state;expect(ended.abandoned).toBe(true);expect(ended.party).toEqual(before.party);
 await page.reload();await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
 expect((await recovery(page)).state.journal).toEqual(ended.journal);await snap(page,'abandoned');
 await openMenu(page);await page.locator('#save-open-btn').click();await page.locator('.drawer').getByRole('button',{name:'Load',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
 expect((await recovery(page)).state.abandoned).toBe(false);
});

test('stationary rest records actual changes and survives manual loading',async({page})=>{
 const state=await baseline(page);state.stats.sanity=9;state.clock_minutes=870;await importState(page,state);
 await expect(page.locator('.game-clock')).toContainText('14:30');
 await page.getByRole('button',{name:'Camp',exact:true}).click();
 await expect(page.locator('.game-clock')).toContainText('14:30');
 await expect(page.getByRole('button',{name:/^(Take a day to rest|Rest for the day|Rest and mute the phones)$/})).toContainText('Improves Sanity');
 await page.getByRole('button',{name:/^(Take a day to rest|Rest for the day|Rest and mute the phones)$/}).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
 const rested=await savedState(page);expect(rested.stats.sanity).toBe(10);expect(rested.day).toBe(state.day+1);expect(rested.clock_minutes).toBe(480);
 expect(rested.miles_traveled_actual).toBe(state.miles_traveled_actual);expect(rested.journal.at(-1).after).toEqual(rested.stats);
 await importState(page,rested);const latest=page.locator('.turn-receipt').first();await expect(latest).toHaveAttribute('data-action','camp');await expect(latest.locator('.receipt-heading')).toHaveText(rested.journal.at(-1).message);
 await page.getByRole('tab',{name:'Journal',exact:true}).click();await expect(page.getByRole('tabpanel',{name:'Journal',exact:true})).toContainText(rested.journal.at(-1).message);
 await page.reload();await waitForLaunch(page);expect((await savedState(page)).journal).toEqual(rested.journal);
});

test('late-route camp stays put and never triggers the hearing',async({page})=>{
 const state=await baseline(page);state.stats.sanity=3;state.clock_minutes=1140;
 state.miles_traveled_actual=state.trail_distance-2;state.miles_traveled=state.miles_traveled_actual;
 state.endgame.active=true;state.endgame.stop_cap_max_full=0;
 await importState(page,state);
 await page.getByRole('button',{name:'Camp',exact:true}).click();
 await page.getByRole('button',{name:/^(Take a day to rest|Rest for the day|Rest and mute the phones)$/}).click();
 await expect(page.locator('.aftermath-panel')).toBeVisible();
 const rested=(await recovery(page)).state;
 expect(rested.day).toBe(state.day+1);expect(rested.clock_minutes).toBe(480);
 expect(rested.miles_traveled_actual).toBe(state.miles_traveled_actual);
 expect(rested.boss.ready).toBe(false);
 expect(rested.day_records.at(-1).kind).toBe('non_travel');
 expect(rested.day_records.at(-1).miles).toBe(0);
 await page.reload();await waitForLaunch(page);expect((await recovery(page)).state.miles_traveled_actual).toBe(state.miles_traveled_actual);
 await snap(page,'late-route-stationary-camp');
});

test('a named crew decision precedes map review and absence persists in later scenes',async({page})=>{
 const state=await baseline(page);state.day=10;state.crew_care={strain:{organizer:2},pending:'organizer',last_check_day:10};
 state.current_encounter=encounters.find((e:{id:string})=>e.id==='classic_mutual_aid');state.scene_subject='organizer';
 await importState(page,state);await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
 await expect(page.locator('.scene-caption')).toContainText(state.party.members.find((m:any)=>m.persona==='organizer').name);
 await expect(page.locator('.scene-speaker')).toHaveCount(0);
 await page.reload();await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
 await page.getByRole('button',{name:'Leave a companion with the medics',exact:true}).click();await expect(page.locator('.aftermath-panel')).toBeVisible();
 const departed=await savedState(page);expect(departed.party.members.find((m:{persona:string})=>m.persona==='organizer').status).toBe('Departed');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','encounter');
 await expect(page.locator('.scene-speaker[data-subject=organizer]')).toHaveCount(0);
 await expect(page.locator('.map-scene')).toHaveCount(0);await snap(page,'crew-care-continuity');
});

test('each persona map always fits its own traveled route',async({page})=>{
 const state=await baseline(page);const views=new Set<string>();
 for(const route of routes){
  state.persona_id=route.id;state.route_services={route_id:route.id,stop:null,traded_at:null,map_reviewed:null};
  await importState(page,state);await page.getByRole('button',{name:'Route',exact:true}).click();
  const view=await page.locator('.us-route-map').getAttribute('viewBox');expect(view).not.toBe('0 0 1000 660');views.add(view!);
  await expect(page.locator('.us-route-map')).toHaveAttribute('data-route',route.id);
  await expect(page.getByRole('button',{name:/Zoom|Overview/})).toHaveCount(0);
  await snap(page,`map-${route.id}`);
 }
 expect(views.size).toBe(6);
});

test('one Resume carries two quiet travel actions; pausing and reload never replay them',async({page})=>{
 const state=await baseline(page);state.seed=42;state.rng_bundle=null;state.encounter_cooldown=100;
 state.weather_state.neutral_buffer=100;state.day_state.day_initialized=true;state.encounter_chance_today=0;
 await importState(page,state);await fastMode(page,true);
 const count=state.journal.length;
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect.poll(async()=>(await recovery(page)).state.journal.length,{timeout:7000}).toBeGreaterThanOrEqual(count+2);
 if(await page.getByRole('button',{name:'Pause travel',exact:true}).count()) await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 const paused=await recovery(page);await page.reload();await waitForLaunch(page);await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
 expect((await recovery(page)).state.journal).toEqual(paused.state.journal);
});


test('visibility loss pauses automatic travel and does not resume it on return',async({page})=>{
 const state=await baseline(page);state.seed=42;state.rng_bundle=null;state.encounter_cooldown=100;
 state.weather_state.neutral_buffer=100;state.day_state.day_initialized=true;state.encounter_chance_today=0;
 await importState(page,state);await fastMode(page,false);
 await page.getByRole('button',{name:'Travel',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');
 // Exercise the browser visibility handler deterministically in headless Chromium.
 await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
 const interrupted=(await recovery(page)).state.journal;
 await page.waitForTimeout(3100);
 expect((await recovery(page)).state.journal).toEqual(interrupted);
 await page.evaluate(()=>{delete (document as any).hidden;document.dispatchEvent(new Event('visibilitychange'));});
 await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeVisible();
});

test('outfitting cannot charge for supplies beyond capacity',async({page})=>{
 await setup(page);
 await page.getByRole('group',{name:'Rations Pack',exact:true}).getByRole('button',{name:'Add +1',exact:true}).click();
 await page.getByRole('group',{name:'Rations Pack',exact:true}).getByRole('button',{name:'Add +1',exact:true}).click();
 await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
 await expect(page.locator('.capacity-warning')).toBeVisible();
 await expect(page.getByRole('button',{name:'Start the journey',exact:true})).toBeDisabled();
});

test('encounter forecasts and the journal agree at stat caps',async({page})=>{
 const state=await baseline(page);state.stats.sanity=10;state.stats.credibility=19;
 state.current_encounter=encounters.find((e:{id:string})=>e.id==='classic_media_training');await importState(page,state);
 await expect(page.locator('.choice-effects').first()).toContainText('Sanity +0');await expect(page.locator('.choice-effects').first()).toContainText('Credibility +1');
 await page.locator('.encounter-choice button').first().click();const after=await savedState(page);
 expect(after.stats.sanity).toBe(10);expect(after.stats.credibility).toBe(20);expect(after.journal.at(-1).after).toEqual(after.stats);
});
