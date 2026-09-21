import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
const catalog=JSON.parse(readFileSync('static/assets/data/town-conversations.json','utf8'));
const source=JSON.parse(readFileSync('../review/art-satire/source-refresh-20260914/complete-pack.json','utf8')).units;
const current=(page:any)=>page.evaluate(()=>{const c=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);c.state.inventory.tags.sort();return c;});
async function load(page:any,unit:string,lang='en',saved=true){
 const story=catalog.find((c:any)=>c.id===unit);
 const route=routes.find((r:any)=>r.stops.some((s:any)=>s.name===story.town));
 const stop=route.stops.find((s:any)=>s.name===story.town);
 const checkpoint=structuredClone(base),s=checkpoint.state;
 checkpoint.phase='Town';checkpoint.aftermath=null;checkpoint.journey_detail={tab:0,expanded:false};
 s.route_services.route_id=route.id;s.route_services.stop=stop.mile;s.route_services.talked_at=null;s.route_services.talk_reward=null;
 s.activities.local_word=null;s.day=3;s.clock_minutes=600;s.region=stop.region;
 s.miles_traveled_actual=stop.mile/route.total_miles*s.trail_distance;
 s.visual_content={edition:1,selections:saved?{[`${story.family_id}/town/${route.id}/${stop.mile}`]:unit}:{}};
 await page.evaluate(({checkpoint,lang}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(checkpoint));localStorage.setItem('dystrail.locale',lang);},{checkpoint,lang});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.town-arrival')).toBeVisible();
 const before=await current(page);await page.locator('.route-stop .action-button').click();await expect(page.locator('.local-conversation')).toHaveAttribute('data-town-unit',unit);
 return {story,before};
}

test('current town stories retain their matching facts sources and saved variants',async({page},info)=>{
 test.setTimeout(240000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('./');await waitForLaunch(page);
 for(const family of ['TOWN-01','TOWN-14','TOWN-19','TOWN-41'])for(const variant of ['A','B','C']){
  const unit=`${family}-${variant}`, {story,before}=await load(page,unit,'en',variant!=='A');
  const authored=source.find((u:any)=>u.id===unit);
  await expect(page.locator('.local-fact')).toHaveText(authored.fact);
  await expect(page.locator('#screen-title')).toHaveText(authored.title);
  await expect(page.locator('.resident-story > p')).toHaveText(story.setup.en);
  await expect(page.locator('.resident-remark')).toHaveText(story.comment.en);
  const links=await page.locator('.fact-source > a').evaluateAll(els=>els.map(e=>e.getAttribute('href')));
  expect(links).toEqual(authored.sources.map((s:any)=>s.url));
  const after=await current(page);expect(after.state.clock_minutes).toBe(630);expect(after.state.rng_bundle).toEqual(before.state.rng_bundle);
  expect(after.aftermath.scene).toEqual({TownConversation:{unit}});
  const sealed=Object.values(after.state.visual_content.selections);expect(sealed).toContain(unit);
  await page.reload();await waitForLaunch(page);expect(await current(page)).toEqual(after);
  await page.locator('.conversation-actions button').click();await page.locator('.route-stop .action-button').click();
  await expect(page.locator('.local-conversation')).toHaveAttribute('data-town-unit',unit);expect(await current(page)).toEqual(after);
 }
 for(const unit of ['TOWN-14-A','TOWN-19-A','TOWN-41-A'])for(const lang of ['es','it','ar']){
  const {story}=await load(page,unit,lang,false);
  await expect(page.locator('.local-fact')).toHaveText(story.text[lang]);
  await expect(page.locator('.resident-story > p')).toHaveText(story.setup[lang]);
  await expect(page.locator('#screen-title')).toHaveText(story.title[lang]);
  await expect(page.locator('.resident-remark')).toHaveText(story.comment[lang]);
  await expect(page.locator('.scene-speaker')).toHaveCount(0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(unit==='TOWN-41-A'){
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
   await page.screenshot({path:info.outputPath(`town-${lang}.png`),fullPage:true,animations:'disabled'});
  }
 }
 expect(errors).toEqual([]);
});
