import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';

test('terminal road scene retains clock and crew while atmosphere changes',async({page,context},info)=>{
 test.setTimeout(120000);
 const base=await baseline(page);base.seed=42;
 base.ending={type:'collapse',cause:'hunger'};
 base.party.members[0].status='Departed';base.party.members[1].status='Dead';
 const appearances=new Set<string>();
 for(const [hour,profile] of [[8,'morning'],[12,'day'],[16,'afternoon'],[19,'dusk'],[23,'night']] as const){
  const state=structuredClone(base);state.clock_minutes=hour*60;
  await importState(page,state);
  const scene=page.locator('.result-art .journey-scene');
  await expect(scene).toHaveAttribute('data-time',profile);
  await expect(scene).toHaveAttribute('data-hour',String(hour));
  const appearance=await scene.locator('.scene-light').evaluate(el=>{
   const s=getComputedStyle(el);return `${s.backgroundImage}|${s.backgroundColor}`;
  });
  appearances.add(appearance);
  const before=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(scene).toHaveAttribute('data-time',profile);
  const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(after.clock_minutes).toBe(hour*60);expect(after.party).toEqual(state.party);
  expect(after.ending).toEqual(before.ending);expect(after.boss).toEqual(before.boss);
  await expect(scene.locator('.crew-van,.standing-member,.character-portrait,.result-profile')).toHaveCount(0);
  await expect(page.locator('.ending-crew [data-fate="crew.departed"]')).toHaveCount(1);
  await expect(page.locator('.ending-crew [data-fate="crew.dead"]')).toHaveCount(1);
  if(hour===12||hour===23) await scene.screenshot({path:info.outputPath(`ending-${profile}.png`)});
  await context.setOffline(false);
 }
 expect(appearances.size).toBe(5);
});

test('terminal scenes use recorded weather without applying another exposure cost',async({page})=>{
 const base=await baseline(page);base.seed=42;base.clock_minutes=720;
 base.ending={type:'collapse',cause:'hunger'};
 for(const [weather,attribute] of [['Clear','clear'],['ColdSnap','cold'],['HeatWave','heat'],['Smoke','smoke'],['Storm','storm']]) {
  const state=structuredClone(base);state.weather_state.today=weather;
  await importState(page,state);
  await expect(page.locator('.result-art .journey-scene')).toHaveAttribute('data-weather',attribute);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(saved.weather_state).toEqual(state.weather_state);
  expect(saved.stats).toEqual(state.stats);expect(saved.clock_minutes).toBe(720);
 }
});
