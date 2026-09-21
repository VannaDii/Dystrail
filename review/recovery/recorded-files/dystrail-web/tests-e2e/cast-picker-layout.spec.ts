import {test,expect} from '@playwright/test';
import {waitForLaunch} from './helpers';

for(const width of [320,390,1280])for(const lang of ['en','es','it','ar']){
 test(`cast picker at ${width}px in ${lang} keeps every portrait and profile clear of actions`,async({page},info)=>{
  await page.setViewportSize({width,height:900});await page.emulateMedia({reducedMotion:'reduce'});
  await page.goto('./');await waitForLaunch(page);
  await page.getByRole('button',{name:'Choose your character',exact:true}).click();
  await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);
  const tiles=page.locator('.persona-tile');await expect(tiles).toHaveCount(6);
  for(const scale of [1,2]){
   if(scale===2)await page.evaluate(()=>{
    const sizes=[...document.querySelectorAll('body *')].filter((e):e is HTMLElement=>e instanceof HTMLElement).map(e=>[e,parseFloat(getComputedStyle(e).fontSize)] as const);
    for(const [el,size]of sizes)el.style.fontSize=`${size*2}px`;
   });
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
   const layout=await page.evaluate(()=>{
    const grid=document.querySelector('.persona-grid')!.getBoundingClientRect();
    const profile=document.querySelector('.persona-preview-card')!.getBoundingClientRect();
    const actions=document.querySelector('.persona-actions')!.getBoundingClientRect();
    return {actionsAfterContent:actions.top>=Math.max(grid.bottom,profile.bottom)-1,overflow:document.documentElement.scrollWidth>innerWidth};
   });
   expect(layout).toEqual({actionsAfterContent:true,overflow:false});
   for(const tile of await tiles.all()){
    await tile.evaluate(el=>el.scrollIntoView({block:'center',behavior:'instant'}));
    await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    expect(await tile.evaluate(el=>{const b=el.getBoundingClientRect();return el.contains(document.elementFromPoint(b.x+b.width/2,b.y+b.height/2));})).toBe(true);
    await tile.focus();await page.keyboard.press('Space');await expect(tile).toHaveAttribute('aria-checked','true');
    const name=await tile.locator('.persona-name').textContent();await expect(page.locator('.persona-preview-header h3')).toHaveText(name!);
    const overflow=await page.locator('.persona-starting').evaluate(el=>{
     const issues:string[]=[];
     for(const cell of el.querySelectorAll('dt,dd')){
      const box=cell.getBoundingClientRect(),range=document.createRange();range.selectNodeContents(cell);
      for(const line of range.getClientRects())if(line.width&&(line.left<box.left-1||line.right>box.right+1||line.bottom>box.bottom+1))issues.push(cell.textContent||'');
     }
     return issues;
    });
    expect(overflow).toEqual([]);
   }
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
   await page.screenshot({path:info.outputPath(`cast-${width}-${lang}-${scale}.png`),fullPage:true,animations:'disabled'});
  }
  await page.locator('#persona-continue').focus();await page.keyboard.press('Enter');
  await expect(page.locator('#main')).toHaveAttribute('data-screen','crew');
 });
}
