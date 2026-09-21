import {test,expect} from '@playwright/test';
import {baseline,importState,openMenu,waitForLaunch} from './helpers';

test('billboards keep replay identity, readable localized text and clock lighting',async({page,context},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?320:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const state=await baseline(page);
 // Use a JS-safe seed: this helper parses saves through JavaScript numbers.
 state.seed=42;await importState(page,state);
 const ad=page.locator('.scene-art .road-billboard');
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
