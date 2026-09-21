import {test, expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const checkpoint=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const current=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
async function load(page:any, stop:number, locale='en', capped=false) {
 const s=structuredClone(checkpoint);s.phase='Town';s.aftermath=null;s.journey_detail={tab:0,expanded:false};
 s.state.route_services.stop=stop;s.state.route_services.talked_at=null;s.state.route_services.talk_reward=null;
 s.state.activities.local_word=null;s.state.clock_minutes=600;s.state.day=3;
 s.state.miles_traveled_actual=stop/2921*s.state.trail_distance;s.state.region=stop<400?'PacificCoast':stop<800?'MountainWest':'Heartland';
 if(capped){s.state.stats.credibility=20;s.state.stats.allies=50;}
 await page.evaluate(({s,locale}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(s));localStorage.setItem('dystrail.locale',locale);},{s,locale});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.town-arrival')).toBeVisible();
 return await current(page);
}
test('town forecasts are qualitative and replay retains one resident and one reward',async({page},info)=>{
 test.setTimeout(180000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('./');await waitForLaunch(page);
 for(const stop of [280,478,1140]){
  let expectedResident:number|undefined;
  for(const locale of ['en','es','it','ar']){
   const before=await load(page,stop,locale);const button=page.locator('.route-stop .action-button');
   await expect(button).toBeVisible();await expect(button.locator('.action-detail')).not.toContainText('+1');
   expect(await button.getAttribute('aria-description')).not.toMatch(/[+−]1/);
   await button.click();await expect(page.locator('.local-conversation')).toBeVisible();
   const after=await current(page);expect(after.clock_minutes).toBe(630);
   expect(after.rng_bundle).toEqual(before.rng_bundle);
   expect(after.stats.credibility-before.stats.credibility).toBe(stop===1140?1:0);
   expect(after.stats.allies-before.stats.allies).toBe(stop===478?1:0);
   expect(after.receipts.length-before.receipts.length).toBe(stop===280?1:0);
   if(expectedResident===undefined)expectedResident=after.activities.local_word;
   expect(after.activities.local_word).toBe(expectedResident);
   await page.reload();await waitForLaunch(page);expect(await current(page)).toEqual(after);
   await page.locator('.conversation-actions button').click();
   await expect(page.locator('.town-arrival')).toBeVisible();await page.locator('.route-stop .action-button').click();
   await expect(page.locator('.local-conversation')).toBeVisible();expect(await current(page)).toEqual(after);
   if(stop===280 && ['en','ar'].includes(locale))await page.screenshot({path:info.outputPath(`town-${locale}.png`),fullPage:true,animations:'disabled'});
  }
 }
 // Capped reward selection comes from the engine; presentation must still describe
 // its receipt fallback, with no invented cap or extra grant.
 for(const stop of [478,1140]){
  const before=await load(page,stop,'en',true);await expect(page.locator('.route-stop .action-detail')).toContainText('Receipt');
  await page.locator('.route-stop .action-button').click();const after=await current(page);
  expect(after.stats.credibility).toBe(20);expect(after.stats.allies).toBe(50);expect(after.receipts.length).toBe(before.receipts.length+1);
 }
 expect(errors).toEqual([]);
});
test('mode selection explains the approved themes in translated and fallback copy',async({page},info)=>{
 const expected:any={en:'attacks on trans rights',es:'derechos trans',it:'diritti trans',ar:'حقوق العابرين',de:'attacks on trans rights'};
 for(const locale of Object.keys(expected)){
  await page.goto('./');await waitForLaunch(page);
  await page.evaluate(locale=>{localStorage.removeItem('dystrail.autosave.v1');localStorage.removeItem('dystrail.save.default');localStorage.setItem('dystrail.locale',locale);},locale);
  await page.reload();await waitForLaunch(page);
  const option=page.locator('.mode-option').nth(1);await expect(option).toContainText(expected[locale]);
  await option.locator('input').check();await expect(page.locator('#run-code')).toHaveValue(/^DP-/);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(['en','ar'].includes(locale))await page.screenshot({path:info.outputPath(`mode-${locale}.png`),fullPage:true,animations:'disabled'});
 }
});
