import {test, expect} from '@playwright/test';
import {baseline, importState, waitForLaunch} from './helpers';
import {readFileSync} from 'node:fs';

test('camp uses preserved symbol-free atlas online and offline', async ({page, context}, info) => {
  await page.setViewportSize({width:info.project.name === 'mobile' ? 390 : 1440,height:1000});
  await page.emulateMedia({reducedMotion:'reduce'});
  await baseline(page);
  await page.getByRole('button',{name:'Camp',exact:true}).click();
  const scene=page.locator('.scene-atlas[data-atlas="journey-settings-v1"][data-cell="3"]');
  await expect(scene).toBeVisible();
  const hash=await scene.locator('image').evaluate(async element=>{
    const url=element.getAttribute('href')!;
    const bytes=await (await fetch(url)).arrayBuffer();
    return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),n=>n.toString(16).padStart(2,'0')).join('');
  });
  expect(hash).toBe('813fedcd913da2356e6364561b217490fb550691bb73c0751bb3806b743f1de7');
  await page.evaluate(()=>{(document.activeElement as HTMLElement)?.blur();window.scrollTo(0,0);});
  await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
  await page.screenshot({path:info.outputPath('corrected-camp.png'),fullPage:true});
  await context.setOffline(true);
  await page.reload(); await waitForLaunch(page);
  // Camp is transient presentation; reopen it if restoring the saved travel state.
  if(!(await scene.count())) await page.getByRole('button',{name:'Camp',exact:true}).click();
  await expect(scene).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});


test('shared encounter scenes use the corrected retained atlas offline', async ({page,context}, info) => {
  test.setTimeout(120000);
  await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
  await page.emulateMedia({reducedMotion:'reduce'});
  const base=await baseline(page);base.seed=42;base.visual_content={edition:0,selections:{}};
  const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
  const cases=[['classic_media_training',0],['classic_civic_potluck',1],['classic_bridge_crews',2],['classic_service_station',3],['town_hall_drift',4],['sat_straw_inspection',5],['clinic_triage',6],['classic_radio_phonebank',7],['classic_neighborhood_watch',8],['deep_rustbelt_convoy',9],['overnight_briefing',11]] as const;
  for(const [id,cell] of cases) {
    const state=structuredClone(base);state.current_encounter=bank.find((e:any)=>e.id===id);state.clock_minutes=cell===11?22*60:12*60;
    await importState(page,state);
    const scene=page.locator('.scene-atlas[data-atlas="encounter-settings-v2"]');
    await expect(scene).toHaveAttribute('data-cell',String(cell));
    await expect(scene.locator('image')).toHaveAttribute('href',/encounter-settings-v2.png$/);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    if([1,6,8].includes(cell)) await page.locator('.world-view').screenshot({path:info.outputPath(`corrected-encounter-${cell}.png`)});
  }
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  const scene=page.locator('.scene-atlas[data-atlas="encounter-settings-v2"]');
  await expect(scene).toHaveAttribute('data-cell','11');
  const hash=await scene.locator('image').evaluate(async element=>{
    const bytes=await(await fetch(element.getAttribute('href')!)).arrayBuffer();
    return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),n=>n.toString(16).padStart(2,'0')).join('');
  });
  expect(hash).toBe('f68bceb4a4644feff2823bc903fe32e12e0f7f2e09c28c0d8bd290fa4f12837c');
});
