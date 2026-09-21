import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));
const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
async function show(page:any,base:any,variant:string,lang:string,clock=600){
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 const gs=structuredClone(base);gs.seed=42;gs.rng_bundle=null;
 Object.assign(gs.stats,{supplies:8,hp:8,sanity:5,credibility:5});
 gs.current_encounter=bank.find((e:any)=>e.id==='classic_civic_potluck');
 gs.last_encounter_driving_minutes=300;gs.driving_minutes_total=300;gs.clock_minutes=clock;
 gs.weather_state.today='Storm';gs.weather_state.neutral_buffer=100;
 gs.scene_subject=gs.party.members[1].persona;gs.party.members[1].status='Departed';
 gs.visual_content.selections={'ENC-C02/road/300':`ENC-C02-${variant}`};await importState(page,gs);
 if(lang!=='en'){await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);}
 return gs;
}
async function capture(page:any,testInfo:any,name:string){
 await page.evaluate(()=>{window.scrollTo(0,0);(document.activeElement as HTMLElement)?.blur();});
 await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 await page.screenshot({path:testInfo.outputPath(`${name}.png`),fullPage:true});
}
test('pantry variants retain actual effects, labels and selected outcomes across languages and offline reload',async({page,context},testInfo)=>{
 test.setTimeout(300000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 const base=await baseline(page);const outcomes=new Map<number,any>();
 for(const variant of ['A','B','C'])for(const lang of ['en','es','it','ar'])for(const choice of [0,1,2]){
  const gs=await show(page,base,variant,lang),unit=`ENC-C02-${variant}`,row='ABC'.indexOf(variant),text=copy(lang).visual_copy[unit];
  const scene=page.locator('.world-view .journey-scene');
  await expect(scene).toHaveAttribute('data-unit',unit);await expect(scene).toHaveAttribute('data-choice','offered');
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-atlas','road-c02-offer-meal-20260914');
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-cell',String(row*2));
  const box=await scene.locator('.scene-art').boundingBox();expect(box!.width/box!.height).toBeCloseTo(760/[366,308,326][row],2);
  await expect(page.locator('.encounter-desc')).toContainText(text.desc);
  await expect(scene.locator(`[data-subject="${gs.scene_subject}"]`)).toHaveCount(0);
  await expect(scene.locator('.scene-npc')).toHaveCount(0);
  await expect(scene.locator('.road-prop-description')).toHaveText(text.overlay_0);
  const before=await checkpoint(page);
  if(choice===0&&(lang==='en'||lang==='ar'))await capture(page,testInfo,`pantry-${variant}-${lang}-offer`);
  await page.locator('.encounter-choice button').nth(choice).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  await expect(scene).toHaveAttribute('data-choice',String(choice));
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-atlas',choice===0?'road-c02-offer-meal-20260914':'road-c02-record-donate-20260914');
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-cell',String(row*2+(choice===1?0:1)));
  await expect(page.locator('.outcome-copy')).toContainText(text[`log_${choice}`]);
  expect(await scene.locator('.road-prop-lettering text').evaluateAll(nodes=>nodes.every(n=>{const r=n.getBoundingClientRect(),a=n.closest('.scene-art')!.getBoundingClientRect();return r.width>30&&r.left>=a.left&&r.right<=a.right&&r.top>=a.top&&r.bottom<=a.bottom;}))).toBe(true);
  const after=await checkpoint(page);
  expect(after.aftermath.scene).toEqual({EncounterOutcome:{unit,choice}});
  expect(after.state.stats.supplies).toBe(before.state.stats.supplies+[3,2,-1][choice]);
  expect(after.state.stats.sanity).toBe(before.state.stats.sanity+(choice===0?1:0));
  expect(after.state.stats.credibility).toBe(before.state.stats.credibility+[0,1,2][choice]);
  expect(after.state.stats.hp).toBe(before.state.stats.hp);
  expect(after.state.receipts).toEqual(before.state.receipts);
  expect(after.state.clock_minutes).toBe(before.state.clock_minutes+30);expect(after.state.day).toBe(before.state.day);
  expect(after.state.current_encounter).toBeNull();expect(after.state.vehicle).toEqual(before.state.vehicle);
  expect(after.state.budget_cents).toBe(before.state.budget_cents);
  await expect(page.locator('.outcome-copy')).not.toContainText(`${gs.party.members[1].name}:`);
  const result={stats:after.state.stats,rng:after.state.rng_bundle,receipts:after.state.receipts,day:after.state.day,clock:after.state.clock_minutes,vehicle:after.state.vehicle};
  if(outcomes.has(choice))expect(result).toEqual(outcomes.get(choice));else outcomes.set(choice,result);
  if(lang==='en'||lang==='ar')await capture(page,testInfo,`pantry-${variant}-${lang}-${choice}`);
  if(lang==='en'&&choice===2){mkdirSync('../review/art-satire/client-fixtures',{recursive:true});writeFileSync(`../review/art-satire/client-fixtures/pantry-${variant}-donated.json`,JSON.stringify(after));}
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(scene).toHaveAttribute('data-choice',String(choice));expect((await checkpoint(page)).aftermath).toEqual(after.aftermath);
  expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);await context.setOffline(false);
  await page.locator('#outcome-continue').click();expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);
 }
 expect(errors).toEqual([]);
});
test('pantry day settlement, duplicate input and legacy checkpoints preserve committed state',async({page},testInfo)=>{
 test.setTimeout(120000);const base=await baseline(page);const results=new Map<number,any>();
 for(const choice of [0,1,2])for(const variant of ['A','B','C']){
  const gs=await show(page,base,variant,'en',1140);await page.locator('.encounter-choice button').nth(choice).dblclick();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');const after=await checkpoint(page);
  expect(after.state.day).toBe(gs.day+1);expect(after.state.clock_minutes).toBe(510);
  expect(after.aftermath.scene).toEqual({EncounterOutcome:{unit:`ENC-C02-${variant}`,choice}});
  const invariant={stats:after.state.stats,rng:after.state.rng_bundle,receipts:after.state.receipts,vehicle:after.state.vehicle};
  if(results.has(choice))expect(invariant).toEqual(results.get(choice));else results.set(choice,invariant);
  if(variant==='B'&&choice===2){
   await capture(page,testInfo,'pantry-after-night');
   const old=structuredClone(after);old.aftermath.scene={Encounter:'ENC-C02-B'};
   await page.evaluate(value=>localStorage.setItem('dystrail.autosave.v1',JSON.stringify(value)),old);await page.reload();await waitForLaunch(page);
   await expect(page.locator('.journey-scene')).toHaveAttribute('data-choice','offered');
   expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);
   await expect(page.locator('.outcome-copy')).toContainText(copy('en').visual_copy['ENC-C02-B'].log_2);
   await importState(page,after.state);expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);
  }
 }
});
