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
for(const locale of ['en','es']) test(`each character keeps its ${locale} introduction through checkout and offline reload`,async({page,context},info)=>{
 const ui=JSON.parse(readFileSync(`i18n/${locale}.json`,'utf8'));
 const copy=ui.encounter_copy;
 for(const [unit,english] of Object.entries(JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy) as [string,any][]){
  if(!unit.startsWith('OPEN-'))continue;
  for(const field of ['name','desc','choice_0','log_0']){expect(copy[unit][field]).toBeTruthy();if(locale!=='en')expect(copy[unit][field]).not.toBe(english[field]);}
 }
 test.setTimeout(180000);
 for(const persona of ['Journalist','Organizer','Whistleblower','Lobbyist','Staffer','Satirist']){
  await page.goto('./');await waitForLaunch(page);
  await page.evaluate(l=>{localStorage.clear();localStorage.setItem('dystrail.locale',l);},locale);await page.reload();await waitForLaunch(page);
  await page.getByRole('button',{name:ui.persona.choose,exact:true}).click();
  await page.getByRole('radio',{name:ui.persona[persona.toLowerCase()].name,exact:true}).click();
  await page.getByRole('button',{name:ui.ui.continue,exact:true}).click();
  await page.getByLabel(ui.crew.player,{exact:true}).fill('Vanna Test');
  await page.getByLabel(ui.crew.name,{exact:true}).fill('The Receipts');
  await page.getByRole('button',{name:ui.ui.continue,exact:true}).click();
  await page.getByRole('button',{name:ui.play.review,exact:true}).filter({visible:true}).first().click();
  const intro=page.locator('.departure-intro');const unit=(await intro.getAttribute('data-departure-unit'))!;
  expect(unit).toMatch(new RegExp(`^OPEN-${persona.toUpperCase()}-[ABC]$`));
  await expect(intro).toContainText(copy[unit].desc);
  const origin=await page.locator('.endpoint-context').getAttribute('data-town');
  const fact=endpointFacts.find((f:any)=>f.town===origin);expect(fact).toBeTruthy();
  await page.locator('.endpoint-context .help-trigger').click();
  await expect(page.locator('.viewport-help .endpoint-fact')).toHaveText(fact.text[locale]||fact.text.en);
  await expect(page.locator('.viewport-help a')).toHaveAttribute('href',fact.source);
  await page.keyboard.press('Escape');
  if(persona==='Satirist'){
   for(const card of await page.locator('.store-card').all()){
    const heading=card.locator('h3');
    await heading.evaluate(el=>el.scrollIntoView({block:'center',behavior:'instant'}));
    expect(await heading.evaluate(el=>{const r=el.getBoundingClientRect();const hit=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return !!hit&&(el===hit||el.contains(hit));})).toBe(true);
    for(const control of await card.locator('button:enabled').all())await control.click({trial:true});
   }
   await page.screenshot({path:info.outputPath('checkout-scrolled-viewport.png')});
  }
  if(persona==='Satirist'){await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.waitForFunction(()=>window.scrollY===0);await page.screenshot({path:info.outputPath('departure-review.png'),fullPage:true});}
  await page.getByRole('button',{name:ui.play.depart,exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
  const check=async()=>{
   const state=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
   expect(state.visual_content.selections[`OPEN-${persona.toUpperCase()}/departure/0`]).toBe(unit);
   expect(state.journal.at(-1).title).toBe(copy[unit].name);
   expect(state.journal.at(-1).message).toBe(copy[unit].log_0);
   expect(state.party.members).toHaveLength(6);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  };
  await check();await context.setOffline(true);await page.reload();await waitForLaunch(page);await check();await context.setOffline(false);
  if(persona==='Satirist')await page.screenshot({path:info.outputPath('departure-report.png'),fullPage:true});
 }
});
