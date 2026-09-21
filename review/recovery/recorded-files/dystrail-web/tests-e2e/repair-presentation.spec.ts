import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const base=JSON.parse(readFileSync('../review/art-satire/client-fixtures/road.json','utf8'));
const copy:any=Object.fromEntries(['en','es','it','ar'].map(lang=>[lang,JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).visual_copy]));
const parts:any={Tire:'TIRE',Battery:'BATTERY',Alternator:'ALTERNATOR',FuelPump:'FUELPUMP'};
const state=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
async function load(page:any,part:string,variant:string,locale='en',town=false,broke=false){
 const save=structuredClone(base);save.phase='Travel';save.aftermath=null;const s=save.state;
 s.day=3;s.clock_minutes=480;s.driving_minutes_total=180;s.breakdown={part,day_started:3};s.last_breakdown_part=part;
 s.vehicle.health=60;s.vehicle.wear=30;s.budget_cents=broke?0:10000;s.budget=broke?0:100;
 s.inventory.spares={tire:broke?0:1,battery:broke?0:1,alt:broke?0:1,pump:broke?0:1};s.stats.supplies=broke?0:12;s.stats.sanity=10;s.stats.morale=8;
 s.route_services.stop=town?280:null;s.visual_content={edition:1,selections:{[`REPAIR-${parts[part]}/repair/3/180`]:`REPAIR-${parts[part]}-${variant}`}};
 await page.evaluate(({save,locale}:any)=>{localStorage.setItem('dystrail.autosave.v1',JSON.stringify(save));localStorage.setItem('dystrail.locale',locale);},{save,locale});
 await page.reload();await waitForLaunch(page);await expect(page.locator('.roadside-options')).toBeVisible();return await state(page);
}
async function capture(page:any,path:string){await page.evaluate(()=>{window.scrollTo({top:0,behavior:'instant'});if(document.activeElement instanceof HTMLElement)document.activeElement.blur();});await page.screenshot({path,fullPage:true,animations:'disabled'});}
test('repair variants retain exact completed costs and recover without duplicate actions',async({page},info)=>{
 test.setTimeout(180000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('./');await waitForLaunch(page);
 for(const part of Object.keys(parts))for(const variant of ['A','B','C'])for(let choice=0;choice<4;choice++){
  const town=choice===1&&variant==='B';const before=await load(page,part,variant,'en',town);const unit=`REPAIR-${parts[part]}-${variant}`;
  await expect(page.locator('#screen-title')).toHaveText(copy.en[unit].title);await expect(page.locator('.scene-narrative')).toContainText(copy.en[unit].setup);
  await expect(page.locator('.roadside-options .action-detail').nth(2)).not.toContainText('−4');await expect(page.locator('.roadside-options .action-detail').nth(3)).not.toMatchText?.(/−[12]/);
  await page.locator('.roadside-options .action-button').nth(choice).click();await expect(page.locator('.aftermath-panel')).toBeVisible();const after=await state(page);
  const field=choice===1&&!town?'roadside_outcome':'outcome';await expect(page.locator('.outcome-copy')).toHaveText(copy.en[unit].choices[`c${choice}`][field]);
  expect(after.state.breakdown).toBeNull();expect(after.state.clock_minutes-before.state.clock_minutes).toBe([60,90,120,240][choice]);expect(after.state.rng_bundle).toEqual(before.state.rng_bundle);
  expect(after.aftermath.scene).toEqual({Repair:{unit,choice}});expect(after.state.stats.supplies).toBe(before.state.stats.supplies-(choice===2?4:0));expect(after.state.stats.sanity).toBe(before.state.stats.sanity-(choice===3?2:0));
  await page.reload();await waitForLaunch(page);await expect(page.locator('.aftermath-panel')).toBeVisible();expect(await state(page)).toEqual(after);
  if(part==='Tire'&&variant==='B'&&choice===1)await capture(page,info.outputPath('paid-repair.png'));
 }
 expect(errors).toEqual([]);
});
test('translated repair offers are readable and cashless repair remains usable',async({page},info)=>{
 await page.goto('./');await waitForLaunch(page);const unit='REPAIR-BATTERY-C';
 for(const lang of ['en','es','it','ar']){
  const before=await load(page,'Battery','C',lang,false,true);await expect(page.locator('#screen-title')).toHaveText(copy[lang][unit].title);
  const buttons=page.locator('.roadside-options .action-button');for(let i=0;i<3;i++)await expect(buttons.nth(i)).toBeDisabled();await expect(buttons.nth(3)).toBeEnabled();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await capture(page,info.outputPath(`offer-${lang}.png`));
  await buttons.nth(3).click();await expect(page.locator('.outcome-copy')).toHaveText(copy[lang][unit].choices.c3.outcome);const after=await state(page);expect(after.state.budget_cents).toBe(0);expect(after.state.stats.supplies).toBe(0);expect(after.state.stats.sanity).toBe(before.state.stats.sanity-2);
 }
});
