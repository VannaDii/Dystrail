import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {waitForLaunch} from './helpers';
test('scene captions overlay artwork without busts on roads and authored scenes',async({page},testInfo)=>{
 test.setTimeout(180000);
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('./');await waitForLaunch(page);
 for(const locale of ['en','ar'])for(const fixture of ['mutual-A-work','mutual-B-offer','teachin-B-rehearsal','road','rest-completed','barter-completed']){
  const checkpoint=readFileSync(`../review/art-satire/client-fixtures/${fixture}.json`,'utf8');
  await page.evaluate(({checkpoint,locale})=>{localStorage.setItem('dystrail.autosave.v1',checkpoint);localStorage.setItem('dystrail.locale',locale);},{checkpoint,locale});
  await page.reload();await waitForLaunch(page);
  const scene=page.locator('.world-view .journey-scene');await expect(scene).toBeVisible();
  await expect(scene.locator('.scene-speaker,.scene-npc')).toHaveCount(0);
  const art=await scene.locator('.scene-art').boundingBox(),caption=await scene.locator('.scene-caption').boundingBox();
  expect(caption!.y).toBeGreaterThanOrEqual(art!.y);
  expect(caption!.y+caption!.height).toBeCloseTo(art!.y+art!.height,0);
  expect(caption!.x).toBeCloseTo(art!.x,0);expect(caption!.width).toBeCloseTo(art!.width,0);
  expect(await scene.locator('.scene-caption').evaluate(el=>el.scrollWidth<=el.clientWidth)).toBe(true);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.locator('#screen-title').focus();await expect(page.locator('#screen-title')).toBeFocused();
  await page.evaluate(()=>{(document.activeElement as HTMLElement)?.blur();scrollTo(0,0);});
  await page.screenshot({path:testInfo.outputPath(`${fixture}-${locale}.png`),fullPage:true,animations:'disabled'});
 }
 expect(errors).toEqual([]);
});
