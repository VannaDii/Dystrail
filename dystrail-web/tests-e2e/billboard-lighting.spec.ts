import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
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


test('all billboard ads are localized and fit in every supported language',async({page,context},info)=>{
 test.setTimeout(300000);
 await page.setViewportSize({width:info.project.name==='mobile'?320:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 const state=await baseline(page);state.seed=42;state.clock_minutes=12*60;
 await openMenu(page);await page.locator('.language-picker > button').click();
 const languages=(await page.getByRole('option').allTextContents()).map(x=>x.trim());expect(languages).toHaveLength(20);
 await page.keyboard.press('Escape');await page.locator('#game-menu-button').click();
 const changeLanguage=async(language:string)=>{
  await openMenu(page);await page.locator('.language-picker > button').click();
  await page.getByRole('option',{name:language,exact:true}).click();await page.locator('#game-menu-button').click();
 };
 const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
 const regions=['PacificCoast','MountainWest','Southwest','Heartland','RustBelt','Beltway'];
 const seen=new Set<string>();
 for(const region of regions) for(const day of [1,2]) {
  const route=routes.find((r:any)=>r.stops.some((s:any)=>s.region===region));
  const stop=route.stops.find((s:any)=>s.region===region);
  state.route_services.route_id=route.id;
  state.miles_traveled_actual=(stop.mile+1)*state.trail_distance/route.total_miles;
  state.region=region;state.day=day;await importState(page,state);
  await expect(page.locator('.world-view')).toHaveAttribute('data-region',region);
  const ad=page.locator('.road-billboard');const id=(await ad.getAttribute('data-billboard'))!;seen.add(id);
  for(const language of languages) {
   await changeLanguage(language);
   const lang=(await page.locator('html').getAttribute('lang'))!;
   const expected=JSON.parse(readFileSync(`i18n/${lang}.json`,'utf8')).road_ad;
   expect(expected?.[id],`${lang}/${id} has its own translations`).toBeTruthy();
   await expect(ad.locator('.billboard-headline-full')).toHaveText(expected[id].headline);
   await expect(ad.locator('.billboard-headline-compact')).toHaveText(expected[id].compact);
   await expect(ad.locator('.billboard-copy')).toHaveText(expected[id].copy);
   const fit=await ad.locator('strong:visible').evaluate(el=>{
    const a=el.getBoundingClientRect(),b=el.parentElement!.getBoundingClientRect();
    return {font:parseFloat(getComputedStyle(el).fontSize),fits:a.top>=b.top-1&&a.bottom<=b.bottom+1&&a.left>=b.left-1&&a.right<=b.right+1};
   });
   expect(fit.font,`${lang}/${id}`).toBeGreaterThanOrEqual(12);expect(fit.fits,`${lang}/${id} headline fits`).toBe(true);
   if(id==='public_land') {
    await page.getByRole('button',{name:expected.label,exact:true}).click();
    await expect(page.locator('.viewport-help:visible')).toContainText(expected[id].copy);await page.keyboard.press('Escape');
   }
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`${lang}/${id}`).toBe(true);
  }
  await changeLanguage('English');
 }
 expect(seen.size).toBe(12);
 await changeLanguage('தமிழ்');
 await page.locator('.world-view').screenshot({path:info.outputPath('billboard-tamil.png')});
 const lastId=(await page.locator('.road-billboard').getAttribute('data-billboard'))!;
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await expect(page.locator('.billboard-headline-compact')).toHaveText(JSON.parse(readFileSync('i18n/ta.json','utf8')).road_ad[lastId].compact);
});
