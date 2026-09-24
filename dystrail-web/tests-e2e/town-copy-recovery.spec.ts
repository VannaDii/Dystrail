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
