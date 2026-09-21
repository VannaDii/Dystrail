import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/town-toledo.json','utf8'));
const catalog=JSON.parse(readFileSync('static/assets/data/town-conversations.json','utf8'));
const state=(page:any)=>page.evaluate(()=>{const s=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state;s.inventory.tags.sort();return s;});
async function load(page:any,variant:string,lang='en',role='journalist',survivors:string[]|null=null,hour=10){
 const save=structuredClone(base),s=save.state;const unit=`TOWN-41-${variant}`;s.persona_id=role;
 s.visual_content.selections={'TOWN-41/town/journalist/2423':unit};s.clock_minutes=hour*60;
 if(survivors)for(const m of s.party.members)m.status=survivors.includes(m.persona)?'Active':m.persona===role?'Dead':'Departed';
 await page.evaluate(({save,lang}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(save));localStorage.setItem('dystrail.locale',lang);},{save,lang});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.town-setting')).toHaveAttribute('data-town-setting',unit);
 return {unit,story:catalog.find((c:any)=>c.id===unit)};
}
async function capture(page:any,path:string){
 await page.evaluate(()=>{window.scrollTo({top:0,behavior:'instant'});if(document.activeElement instanceof HTMLElement)document.activeElement.blur();});
 await page.screenshot({path,fullPage:true,animations:'disabled'});
}
test('town compositions keep active travelers source props and localized context together',async({page},info)=>{
 test.setTimeout(180000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('./');await waitForLaunch(page);
 for(const variant of ['A','B','C'])for(const lang of ['en','es','it','ar']){
  const {unit,story}=await load(page,variant,lang);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-unit',unit);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-indoors',String(variant==='B'));
  await expect(page.locator('#screen-title')).toHaveText(story.title[lang]);
  await expect(page.locator('.local-fact')).toHaveText(story.text[lang]);
  await expect(page.locator('.resident-remark')).toHaveText(story.comment[lang]);
  await expect(page.locator('.town-traveler')).toHaveCount(2);await expect(page.locator('.scene-speaker')).toHaveCount(0);
  const art=await page.locator('.scene-art').boundingBox(),caption=await page.locator('.scene-caption').boundingBox();
  expect(art!.width/art!.height).toBeCloseTo(1.5,2);expect(caption!.y+caption!.height).toBeCloseTo(art!.y+art!.height,0);
  await page.locator('.fact-source .help-trigger').click();await expect(page.locator('.help-popover')).toContainText(story.sources[0].notes[lang]);await page.locator('.help-dismiss').click();
  const before=await state(page);await page.reload();await waitForLaunch(page);expect(await state(page)).toEqual(before);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(['en','ar'].includes(lang))await capture(page,info.outputPath(`${unit}-${lang}.png`));
 }
 for(const [i,role] of ['journalist','organizer','staffer','lobbyist','whistleblower','satirist'].entries()){
  const variant=['A','B','C'][i%3];await load(page,variant,'en',role,[role],i%2?19:6);
  await expect(page.locator('.town-traveler')).toHaveCount(1);await expect(page.locator('.town-traveler')).toHaveAttribute('data-member',role);
  await capture(page,info.outputPath(`solo-${role}.png`));
 }
 await load(page,'A','en','journalist',['organizer']);await expect(page.locator('.town-traveler')).toHaveAttribute('data-member','organizer');
 expect(errors).toEqual([]);
});
