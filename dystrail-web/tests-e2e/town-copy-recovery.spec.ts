import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const catalog=JSON.parse(readFileSync('static/assets/data/town-conversations.json','utf8'));
const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
test('all 132 town conversations retain the approved story, matching sources and one-time reward',async({page,context},info)=>{
 test.setTimeout(360_000);const base=await baseline(page);
 for(const [index,story] of catalog.entries()){
  const route=routes.find((r:any)=>r.stops.some((s:any)=>s.name===story.town));
  const stop=route.stops.find((s:any)=>s.name===story.town);
  const s=structuredClone(base);s.seed=42;s.persona_id=route.id;s.day=6;s.clock_minutes=600;
  s.route_services={...s.route_services,route_id:route.id,stop:stop.mile,trading:false,talked_at:null,talk_reward:null};
  s.activities.local_word=null;s.miles_traveled_actual=(stop.mile+1)/route.total_miles*s.trail_distance;s.miles_traveled=Math.round(s.miles_traveled_actual);
  const key=`${story.family_id}/town/${stop.mile}`;s.visual_content={edition:1,selections:{[key]:story.id},outcomes:{},policy_bulletins:[]};
  await importState(page,s);await expect(page.locator('.town-arrival')).toBeVisible();
  await page.locator('.route-stop .action-button').click();
  await expect(page.locator('#screen-title')).toHaveText(story.title.en);
  await expect(page.locator('.local-fact')).toHaveText(story.text.en);
  await expect(page.locator('.resident-remark')).toHaveText(story.comment.en);
  await expect(page.locator('.resident-story > p')).toHaveText(story.setup.en);
  expect(await page.locator('.fact-source > a').evaluateAll(els=>els.map(e=>e.getAttribute('href')))).toEqual(story.sources.map((s:any)=>s.url));
  const after=await checkpoint(page);expect(after.clock_minutes).toBe(630);expect(after.visual_content.selections[key]).toBe(story.id);
  if(index%3===0){
   await page.reload();await waitForLaunch(page);await expect(page.locator('#screen-title')).toHaveText(story.title.en);
   await page.locator('.conversation-actions button').click();await page.locator('.route-stop .action-button').click();
   const repeated=await checkpoint(page);expect(repeated.stats).toEqual(after.stats);expect(repeated.clock_minutes).toBe(after.clock_minutes);expect(repeated.journal).toEqual(after.journal);
  }
  if(['TOWN-11-C','TOWN-40-B','TOWN-41-A'].includes(story.id)){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.local-fact')).toHaveText(story.text.en);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   await page.evaluate(()=>window.scrollTo(0,0));await page.screenshot({path:info.outputPath(`${story.id}.png`),fullPage:true});
   await context.setOffline(false);
  }
 }
});

test('retained Toledo scenes seat only surviving travelers',async({page,context},info)=>{
 test.setTimeout(120000);const base=await baseline(page);
 for(const variant of ['B','C'])for(const survivors of [0,1,4,6]){
  const story=catalog.find((c:any)=>c.id===`TOWN-41-${variant}`);
  const route=routes.find((r:any)=>r.stops.some((s:any)=>s.name===story.town)),stop=route.stops.find((s:any)=>s.name===story.town);
  const s=structuredClone(base);s.seed=42;s.persona_id=route.id;s.day=6;s.clock_minutes=600;s.turn_journal_start=null;
  s.party.members.forEach((m:any,i:number)=>m.status=i<survivors?'Active':i%2?'Departed':'Dead');
  s.route_services={...s.route_services,route_id:route.id,stop:stop.mile,trading:false,talked_at:null,talk_reward:null};s.activities.local_word=null;
  s.miles_traveled_actual=(stop.mile+1)/route.total_miles*s.trail_distance;s.miles_traveled=Math.round(s.miles_traveled_actual);
  const key=`${story.family_id}/town/${stop.mile}`;s.visual_content={edition:1,selections:{[key]:story.id},outcomes:{},policy_bulletins:[]};
  await importState(page,s);await page.locator('.route-stop .action-button').click();
  await expect(page.locator('[data-town-unit]')).toHaveAttribute('data-town-unit',story.id);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',String(variant==='B'));
  expect(await page.locator('[data-seated-traveler]').evaluateAll(els=>els.map(e=>e.getAttribute('data-seated-traveler')))).toEqual(s.party.members.filter((m:any)=>m.status==='Active').slice(0,2).map((m:any)=>m.persona));
  const after=await checkpoint(page);expect(after.party).toEqual(s.party);expect(after.clock_minutes).toBe(630);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(page.locator('[data-town-unit]')).toHaveAttribute('data-town-unit',story.id);
  await expect(page.locator('.resident-remark')).toHaveText(story.comment.en);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(survivors===6){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`town-${variant}.png`),fullPage:true});}
  await page.locator('.conversation-actions button').click();await page.locator('.route-stop .action-button').click();
  const repeated=await checkpoint(page);expect(repeated.stats).toEqual(after.stats);expect(repeated.clock_minutes).toBe(630);expect(repeated.journal).toEqual(after.journal);
  await context.setOffline(false);
 }
});

test('indoor cafe conversations use the shared native setting',async({page,context},info)=>{
 test.setTimeout(120000);const base=await baseline(page);
 const units=['TOWN-02-B','TOWN-16-A','TOWN-17-C','TOWN-18-A','TOWN-19-C','TOWN-20-C','TOWN-23-B','TOWN-25-A','TOWN-26-C','TOWN-36-A'];
 for(const id of units){
  const story=catalog.find((c:any)=>c.id===id),route=routes.find((r:any)=>r.stops.some((s:any)=>s.name===story.town)),stop=route.stops.find((s:any)=>s.name===story.town);
  const s=structuredClone(base);s.seed=42;s.persona_id=route.id;s.day=6;s.clock_minutes=600;s.turn_journal_start=null;
  s.route_services={...s.route_services,route_id:route.id,stop:stop.mile,trading:false,talked_at:null,talk_reward:null};s.activities.local_word=null;
  s.miles_traveled_actual=(stop.mile+1)/route.total_miles*s.trail_distance;s.miles_traveled=Math.round(s.miles_traveled_actual);
  const key=`${story.family_id}/town/${stop.mile}`;s.visual_content={edition:1,selections:{[key]:id},outcomes:{},policy_bulletins:[]};
  await importState(page,s);await page.locator('.route-stop .action-button').click();
  const outdoors=['TOWN-18-A','TOWN-36-A'].includes(id);
  if(outdoors){
   await expect(page.locator('[data-town-context]')).toHaveCount(0);
   await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','false');
  }else{
  await expect(page.locator('[data-town-context]')).toHaveAttribute('data-town-context','cafe');
  await expect(page.locator('[data-town-unit]')).toHaveAttribute('data-town-unit',id);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','true');
  const box=await page.locator('.journey-scene > .scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(16/9,1);
  }
  await expect(page.locator('.resident-story > p')).toHaveText(story.setup.en);
  const after=await checkpoint(page);expect(after.party).toEqual(s.party);expect(after.clock_minutes).toBe(630);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  if(outdoors) await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors','false');
  else await expect(page.locator('[data-town-unit]')).toHaveAttribute('data-town-unit',id);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(['TOWN-02-B','TOWN-16-A','TOWN-18-A','TOWN-36-A'].includes(id)){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath(`${id}.png`),fullPage:true});}
  await context.setOffline(false);
 }
});
