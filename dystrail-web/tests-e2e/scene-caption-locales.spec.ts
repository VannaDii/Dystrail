import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch,snap} from './helpers';
import {atTown} from './geography';

test('scene title stays bottom-left in LTR and RTL layouts',async({page})=>{
 const original=await baseline(page);
 for(const town of [false,true]) {
  await page.evaluate(()=>localStorage.setItem('dystrail.locale','en'));
  await page.reload();await waitForLaunch(page);
  const state=structuredClone(original);state.seed=42;
  if(town) atTown(state,'Spokane');
  await importState(page,state);
  for(const lang of ['en','es','ar']) {
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);
   await page.reload();await waitForLaunch(page);
   const layout=await page.locator('.world-view .scene-caption').evaluate(el=>{
    const h=el.querySelector('h1')!,box=el.getBoundingClientRect(),title=h.getBoundingClientRect();
    return {left:title.left-box.left,padding:parseFloat(getComputedStyle(el).paddingLeft),
      direction:getComputedStyle(h).direction,align:getComputedStyle(h).textAlign,
      bottom:title.bottom<=box.bottom,width:title.width<=box.width};
   });
   expect(layout.left).toBeCloseTo(layout.padding,0);
   expect(layout.direction).toBe(lang==='ar'?'rtl':'ltr');
   expect(layout.align).toBe('left');expect(layout.bottom).toBe(true);expect(layout.width).toBe(true);
   if(lang==='ar') await snap(page,`caption-ar-${town?'town':'road'}`);
  }
 }
});
