import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch,depart,importState} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8'));
const personas=JSON.parse(readFileSync('static/assets/data/personas.json','utf8'));
const store=JSON.parse(readFileSync('static/assets/data/store.json','utf8'));
test('approved persona and mode copy matches retained runtime catalogs',()=>{
 const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units;
 for(const unit of units.filter((u:any)=>u.category==='Persona descriptions')){
  const id=unit.id.replace('PERSONA-','').toLowerCase();expect(copy.persona[id].desc).toBe(unit.source_paragraphs[1]);expect(personas[id].desc).toBe(copy.persona[id].desc);
 }
 for(const [id,key] of [['MODE-C','classic_desc'],['MODE-D','deep_desc']])expect(copy.ux[key]).toBe(units.find((u:any)=>u.id===id).source_paragraphs[1]);
 for(const item of store.categories.flatMap((c:any)=>c.items))expect(item.desc).toBe(copy.store.items[item.id].desc);
});
test('mode themes, all persona descriptions and all shop descriptions fit the existing onboarding',async({page,context},info)=>{
 await page.goto('./');await waitForLaunch(page);
 await expect(page.getByText(copy.ux.classic_desc,{exact:true})).toBeVisible();await expect(page.getByText(copy.ux.deep_desc,{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 for(const [id,p] of Object.entries(personas) as [string,any][]){
  await page.getByRole('radio',{name:p.name,exact:true}).click();
  await expect(page.locator('#persona-preview .persona-preview-header p')).toHaveText(p.desc);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }
 await page.getByRole('radio',{name:'Journalist',exact:true}).click();await page.getByRole('button',{name:'Continue',exact:true}).click();
 await page.getByLabel('Your name',{exact:true}).fill('Vanna Test');await page.getByLabel('Crew name',{exact:true}).fill('The Receipts');await page.getByRole('button',{name:'Continue',exact:true}).click();
 for(const item of store.categories.flatMap((c:any)=>c.items))await expect(page.locator('.store-card').filter({has:page.locator(`#store-item-${item.id}`)}).locator('.store-item-detail > span').first()).toHaveText(item.desc);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath('shop-descriptions.png'),fullPage:true});
 await depart(page);const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
 saved.boss={ready:true,reached:true,attempted:false,victory:false,presentation:'Arrival',hearing:null};saved.current_encounter=null;saved.route_services.stop=null;
 await importState(page,saved);await expect(page.locator('.hearing')).toHaveAttribute('data-hearing-phase','Arrival');
 const fact=JSON.parse(readFileSync('static/assets/data/town-facts.json','utf8')).find((f:any)=>f.town==='D.C.');
 await page.locator('.endpoint-context .help-trigger').click();await expect(page.locator('.viewport-help .endpoint-fact')).toHaveText(fact.text.en);await expect(page.locator('.viewport-help a')).toHaveAttribute('href',fact.source);
 await page.screenshot({path:info.outputPath('arrival-context.png'),fullPage:true});await page.keyboard.press('Escape');
 await context.setOffline(true);await page.reload();await waitForLaunch(page);expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.stats)).toEqual(saved.stats);await context.setOffline(false);
});
