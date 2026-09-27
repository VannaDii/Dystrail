import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,openMenu,waitForLaunch} from './helpers';

test('billboards keep replay identity, readable localized text and clock lighting',async({page,context},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?320:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const state=await baseline(page);
 // Use a JS-safe seed: this helper parses saves through JavaScript numbers.
 state.seed=42;await importState(page,state);
 const ad=page.locator('.scene-art .road-billboard').first();
 const first=await ad.getAttribute('data-billboard');
 for(const [hour,light] of [[8,'morning'],[13,'day'],[16,'afternoon'],[19,'dusk'],[23,'night']] as const){
  state.clock_minutes=hour*60;
  await importState(page,state);
  await expect(page.locator('.journey-scene')).toHaveAttribute('data-time',light);
  await expect(ad).toHaveAttribute('data-billboard',first!);
 }
 const labels=[['English','Roadside advertisement'],['Español','Anuncio de carretera'],['Italiano','Pubblicità stradale'],['العربية','إعلان على الطريق']];
 for(const [language,label] of labels){
  await openMenu(page);await page.locator('.language-picker > button').click();
  await page.getByRole('option',{name:language,exact:true}).click();
  await page.locator('#game-menu-button').click();
  const headline=await ad.locator('strong:visible').innerText();expect(headline).not.toContain('road_ad.');
  const fits=await ad.locator('strong:visible').evaluate(el=>{
   const text=el.getBoundingClientRect(),panel=el.parentElement!.getBoundingClientRect();
   return {font:parseFloat(getComputedStyle(el).fontSize),fits:text.top>=panel.top&&text.bottom<=panel.bottom&&text.left>=panel.left&&text.right<=panel.right};
  });
  expect(fits.font).toBeGreaterThanOrEqual(12);expect(fits.fits,`${language} headline fits its sign`).toBe(true);
  const before=await page.evaluate(()=>localStorage.getItem('dystrail.autosave.v1'));
  await page.getByRole('button',{name:label,exact:true}).click();
  await expect(page.locator('.viewport-help:visible')).toContainText(await ad.locator('.billboard-headline-full').textContent() || '');
  await page.keyboard.press('Escape');
  expect(await page.evaluate(()=>localStorage.getItem('dystrail.autosave.v1'))).toBe(before);
 }
 await page.evaluate(()=>window.scrollTo(0,0));
 await page.screenshot({path:info.outputPath('billboard-night-ar.png'),fullPage:true});
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await expect(ad).toHaveAttribute('data-billboard',first!);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-time','night');
});


test('all billboard ads have localized copy and reusable art',async({page},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?320:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const state=await baseline(page);state.seed=42;state.clock_minutes=12*60;
 await openMenu(page);await page.locator('.language-picker > button').click();
 const languages=(await page.getByRole('option').allTextContents()).map(x=>x.trim());expect(languages).toHaveLength(20);
 await page.keyboard.press('Escape');await page.locator('#game-menu-button').click();
 const seen=new Set<string>();
 for(let step=0;step<12;step++) {
  state.continuity.driving_minutes_total=step*60;
  await importState(page,state);
  const ad=page.locator('.road-billboard').first();const id=(await ad.getAttribute('data-billboard'))!;seen.add(id);
  const illustrated=ad.locator('.billboard-art img');
  await expect(illustrated).toHaveJSProperty('complete',true);
  expect(await illustrated.evaluate((node:HTMLImageElement)=>node.naturalWidth),`${id} has reusable art`).toBeGreaterThan(0);
  for(const code of ['en','es','fr','it','ar','ta']) {
   const expected=JSON.parse(readFileSync(`i18n/${code}.json`,'utf8')).road_ad;
   expect(expected?.[id]?.headline,`${code}/${id} headline`).toBeTruthy();
   expect(expected?.[id]?.compact,`${code}/${id} compact`).toBeTruthy();
   expect(expected?.[id]?.copy,`${code}/${id} copy`).toBeTruthy();
  }
 }
 expect(seen.size).toBe(12);
 for(const code of ['de','pt','ru','ja','ko','zh','hi','bn','id','tr','jv','mr','pa','te']) {
  const expected=JSON.parse(readFileSync(`i18n/${code}.json`,'utf8')).road_ad;
  for(const id of seen) expect(expected?.[id]?.headline&&expected?.[id]?.compact&&expected?.[id]?.copy,`${code}/${id} translated`).toBeTruthy();
 }
});

test('each seeded sign passes the van without swapping while visible',async({page},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await baseline(page);
 const scene=page.locator('.scene-road');
 const track=scene.locator('.scene-art > .road-pan-track');
 const signs=track.locator('.road-billboard');
 await expect(signs).toHaveCount(1);
 const id=await signs.first().getAttribute('data-billboard');
 for(const sign of await signs.all()){
  const image=sign.locator('img');
  await expect(image).toHaveJSProperty('complete',true);
  expect(await image.evaluate((node:HTMLImageElement)=>node.naturalWidth)).toBeGreaterThan(0);
  await expect(sign.locator('.billboard-headline-full')).not.toBeEmpty();
 }
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(scene).toHaveClass(/scene-moving/);
 const positions=await track.evaluate(el=>{
  const animation=el.getAnimations()[0];
  const sign=el.querySelector('.road-billboard')!;
  const road=el.querySelector('.scene-background')!;
  const van=el.parentElement!.querySelector('.crew-van')!;
  const read=()=>({sign:sign.getBoundingClientRect().left,road:road.getBoundingClientRect().left,van:van.getBoundingClientRect().left});
  const duration=animation.effect!.getTiming().duration as number;
  animation.pause();animation.currentTime=0;const start=read();
  animation.currentTime=duration/2;const middle=read();
  animation.currentTime=duration-1;const end=read();
  animation.currentTime=0;const next=read();animation.play();
  return {start,middle,end,next,sceneLeft:el.parentElement!.getBoundingClientRect().left};
 });
 expect(positions.middle.sign).toBeLessThan(positions.start.sign);
 expect(positions.middle.sign-positions.start.sign).toBeCloseTo(positions.middle.road-positions.start.road,0);
  expect(Math.abs(positions.middle.van-positions.start.van)).toBeLessThan(2);
 expect(positions.end.sign).toBeLessThan(positions.middle.sign);
 expect(positions.end.sign).toBeLessThan(positions.sceneLeft);
 expect(positions.next.sign).toBeCloseTo(positions.start.sign,0);
 await expect(signs.first()).toHaveAttribute('data-billboard',id!);
 await page.screenshot({path:info.outputPath('illustrated-billboards.png'),fullPage:true});
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await expect(scene).not.toHaveClass(/scene-moving/);
 await expect(track).toHaveCSS('animation-play-state','paused');
 await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(scene).toHaveClass(/scene-moving/);
 await expect(track).toHaveCSS('animation-play-state','running');
});
