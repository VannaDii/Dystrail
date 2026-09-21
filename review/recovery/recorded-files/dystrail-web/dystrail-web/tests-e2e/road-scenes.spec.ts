import {test,expect} from '@playwright/test';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {baseline,importState,snap,waitForLaunch} from './helpers';
const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
const copy=(lang:string)=>JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8'));
const checkpoint=(page:any)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
async function show(page:any,base:any,variant:string,lang:string){
 await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));await page.reload();await waitForLaunch(page);
 const gs=structuredClone(base);gs.seed=42;gs.rng_bundle=null;gs.stats.supplies=8;gs.stats.hp=8;gs.stats.credibility=5;
 gs.current_encounter=bank.find((e:any)=>e.id==='classic_bridge_crews');gs.last_encounter_driving_minutes=300;gs.driving_minutes_total=300;
 gs.clock_minutes=variant==='B'?1140:780;gs.weather_state.today='Storm';gs.weather_state.neutral_buffer=100;
 gs.scene_subject=gs.party.members[1].persona;gs.party.members[1].status='Departed';
 gs.visual_content.selections={'ENC-C01/road/300':`ENC-C01-${variant}`};
 await importState(page,gs);
 if(lang!=='en'){await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);}
 return gs;
}

test('road choices show their actual work or unchanged photographed site in four languages',async({page,context})=>{
 test.setTimeout(300000);const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));const base=await baseline(page);
 for(const variant of ['A','B','C'])for(const lang of ['en','es','it','ar'])for(const choice of [0,1]){
  const gs=await show(page,base,variant,lang),unit=`ENC-C01-${variant}`,row='ABC'.indexOf(variant);
  const locale=copy(lang),text=variant==='A'?locale.encounter_copy.classic_bridge_crews:locale.visual_copy[unit];
  const scene=page.locator('.world-view .journey-scene');
  await expect(scene).toHaveAttribute('data-unit',unit);await expect(scene).toHaveAttribute('data-choice','offered');
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-atlas','road-c01-20260914');
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-cell',String(row*2));
  await expect(scene).toHaveAttribute('data-indoors',String(variant==='B'));
  await expect(scene).toHaveAttribute('data-weather','storm');
  await expect(scene.locator('.scene-npc')).toHaveCount(0);
  await expect(scene.locator(`[data-subject="${gs.scene_subject}"]`)).toHaveCount(0);
  await expect(page.locator('.encounter-desc')).toContainText(text.desc);
  if(variant!=='A'){
   await expect(scene.locator('.road-prop-description')).toHaveText(variant==='B'?text.overlay_0:`${text.overlay_0} · ${text.overlay_1}`);
   expect(await scene.locator('.road-prop-lettering text').evaluateAll(nodes=>nodes.every(n=>{const r=n.getBoundingClientRect();const a=n.closest('.scene-art')!.getBoundingClientRect();return r.width>30&&r.left>=a.left&&r.right<=a.right&&r.top>=a.top&&r.bottom<=a.bottom;}))).toBe(true);
  }
  const before=await checkpoint(page);
  if(choice===0&&(lang==='en'||lang==='ar'))await snap(page,`road-${unit}-${lang}-offered`);
  await page.locator('.encounter-choice button').nth(choice).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
  await expect(scene).toHaveAttribute('data-choice',String(choice));
  await expect(scene.locator('.scene-atlas')).toHaveAttribute('data-cell',String(row*2+(choice===0?1:0)));
  await expect(page.locator('.outcome-copy')).toContainText(text[`log_${choice}`]);
  const after=await checkpoint(page);
  expect(after.aftermath.scene).toEqual({EncounterOutcome:{unit,choice}});
  expect(after.state.stats.supplies).toBe(before.state.stats.supplies-(choice===0?1:0));
  expect(after.state.stats.hp).toBe(before.state.stats.hp-(choice===0?1:0));
  expect(after.state.stats.credibility).toBe(before.state.stats.credibility+(choice===0?2:1));
  expect(after.state.receipts.length).toBe(before.state.receipts.length+(choice===1?1:0));
  expect(after.state.clock_minutes).toBe(before.state.clock_minutes);
  expect(after.state.current_encounter).toBeNull();
  expect(after.state.vehicle).toEqual(before.state.vehicle);
  expect(after.state.budget_cents).toBe(before.state.budget_cents);
  if(lang==='en'||lang==='ar')await snap(page,`road-${unit}-${lang}-choice-${choice}`);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(scene).toHaveAttribute('data-choice',String(choice));
  expect((await checkpoint(page)).aftermath).toEqual(after.aftermath);
  expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);
  await context.setOffline(false);
  if(variant==='C'&&choice===0&&lang==='en'){
   mkdirSync('../review/art-satire/client-fixtures',{recursive:true});writeFileSync('../review/art-satire/client-fixtures/road-completed.json',JSON.stringify(after));
  }
  await page.locator('#outcome-continue').click();
  expect((await checkpoint(page)).state.stats).toEqual(after.state.stats);
 }
 expect(errors).toEqual([]);
});

test('road outcome survives checkpoint import; legacy outcome remains recoverable without invented work',async({page})=>{
 const base=await baseline(page);await show(page,base,'B','en');
 await page.locator('.encounter-choice button').first().dblclick();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
 const saved=await checkpoint(page);expect(saved.state.stats.supplies).toBe(7);expect(saved.state.stats.hp).toBe(7);
 await importState(page,saved);await expect(page.locator('.journey-scene')).toHaveAttribute('data-choice','0');
 expect((await checkpoint(page)).state.stats).toEqual(saved.state.stats);
 // This is the previously supported serialized scene shape, without a choice.
 saved.aftermath.scene={Encounter:'ENC-C01-B'};
 await importState(page,saved);await expect(page.locator('.journey-scene')).toHaveAttribute('data-choice','offered');
 await expect(page.locator('.outcome-copy')).toContainText(copy('en').visual_copy['ENC-C01-B'].log_0);
 expect((await checkpoint(page)).state.stats).toEqual(saved.state.stats);
 await page.locator('#outcome-continue').click();
 expect((await checkpoint(page)).state.stats).toEqual(saved.state.stats);
});
