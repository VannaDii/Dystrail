import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,waitForLaunch} from './helpers';
const languages=[...readFileSync('src/i18n/locales.rs','utf8').matchAll(/code: "([a-z]+)"/g)].map(m=>m[1]);

test('localized action text stays inside each button',async({page},info)=>{
 test.setTimeout(300000);await baseline(page);
 for(const width of info.project.name==='mobile'?[320,390]:[1440]) {
  await page.setViewportSize({width,height:900});
  for(const lang of languages){
   await page.evaluate(l=>localStorage.setItem('dystrail.locale',l),lang);await page.reload();await waitForLaunch(page);
   await expect(page.locator('html')).toHaveAttribute('lang',lang);
   const violations=await page.locator('.journey-actions').evaluate(root=>{
    const errors:string[]=[];
    for(const button of root.querySelectorAll('button')){
     const b=button.getBoundingClientRect(),style=getComputedStyle(button);
     if(style.hyphens!=='auto') errors.push('hyphenation disabled');
     const walker=document.createTreeWalker(button,NodeFilter.SHOW_TEXT);
     while(walker.nextNode()){
      if(!walker.currentNode.textContent?.trim()) continue;
      const range=document.createRange();range.selectNodeContents(walker.currentNode);
      for(const r of range.getClientRects())if(r.left<b.left+1||r.right>b.right-1||r.top<b.top||r.bottom>b.bottom)errors.push(`${button.textContent}: text outside button`);
     }
     if(button.scrollWidth>button.clientWidth+1)errors.push(`${button.textContent}: scroll overflow`);
    }
    if(document.documentElement.scrollWidth>innerWidth)errors.push('page overflow');
    return errors;
   });
   expect(violations,`${lang} at ${width}px`).toEqual([]);
   if(lang==='es')await page.screenshot({path:info.outputPath(`spanish-actions-${width}.png`),fullPage:true});
  }
 }
});
