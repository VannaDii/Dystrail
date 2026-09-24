import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
const endpointFacts=JSON.parse(readFileSync('static/assets/data/town-facts.json','utf8'));
test('all departure copy matches the approved source',()=>{
 const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.filter((u:any)=>u.category==='Character departure introductions');
 expect(units).toHaveLength(18);
 for(const unit of units){
  expect(copy[unit.id].name).toBe(unit.title);
  expect(copy[unit.id].desc).toBe(unit.source_paragraphs[1]);
  expect(copy[unit.id].log_0).toBe(unit.source_paragraphs[2].split(' → ')[1]);
 }
});
test('each character keeps its introduction through checkout and offline reload',async({page,context},info)=>{
 test.setTimeout(180000);
 for(const persona of ['Journalist','Organizer','Whistleblower','Lobbyist','Staffer','Satirist']){
  await page.goto('./');await waitForLaunch(page);
  await page.evaluate(()=>localStorage.clear());await page.reload();await waitForLaunch(page);
  await page.getByRole('button',{name:'Choose your character',exact:true}).click();
  await page.getByRole('radio',{name:persona,exact:true}).click();
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await page.getByLabel('Your name',{exact:true}).fill('Vanna Test');
  await page.getByLabel('Crew name',{exact:true}).fill('The Receipts');
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
  const intro=page.locator('.departure-intro');const unit=(await intro.getAttribute('data-departure-unit'))!;
  expect(unit).toMatch(new RegExp(`^OPEN-${persona.toUpperCase()}-[ABC]$`));
  await expect(intro).toContainText(copy[unit].desc);
  const origin=await page.locator('.endpoint-context').getAttribute('data-town');
  const fact=endpointFacts.find((f:any)=>f.town===origin);expect(fact).toBeTruthy();
  await page.locator('.endpoint-context .help-trigger').click();
  await expect(page.locator('.viewport-help .endpoint-fact')).toHaveText(fact.text.en);
  await expect(page.locator('.viewport-help a')).toHaveAttribute('href',fact.source);
  await page.keyboard.press('Escape');
  if(persona==='Satirist'){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.waitForFunction(()=>window.scrollY===0);await page.screenshot({path:info.outputPath('departure-review.png'),fullPage:true});}
  await page.getByRole('button',{name:'Start the journey',exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
  const check=async()=>{
   const state=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   expect(state.visual_content.selections[`OPEN-${persona.toUpperCase()}/departure/0`]).toBe(unit);
   expect(state.journal.at(-1).title).toBe(copy[unit].name);
   expect(state.journal.at(-1).message).toBe(copy[unit].log_0);
   expect(state.party.members).toHaveLength(6);
  };
  await check();await context.setOffline(true);await page.reload();await waitForLaunch(page);await check();await context.setOffline(false);
  if(persona==='Satirist')await page.screenshot({path:info.outputPath('departure-report.png'),fullPage:true});
 }
});
