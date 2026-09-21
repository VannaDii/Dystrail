import {test,expect,Page} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
async function enlarge(page:Page){
 // Text-only enlargement, measured before assigning sizes to avoid nested compounding.
 await page.evaluate(()=>{
  const sizes=[...document.querySelectorAll('body *')].filter((e):e is HTMLElement=>e instanceof HTMLElement).map(e=>[e,parseFloat(getComputedStyle(e).fontSize)] as const);
  for(const [el,size] of sizes)el.style.fontSize=`${size*2}px`;
 });
}
async function readable(page:Page){
 const result=await page.evaluate(()=>{
  const issues:string[]=[];
  const selector='.hud-stat,.turn-receipt .stat-card,.hud-primary-row,.journey-actions>button,.travel-tabs button,.conditions-hud .weather-name,.resource-hud .help-trigger,.conditions-hud .help-trigger,.scene-caption,.billboard-panel,.road-prop-description,.encounter-choice button';
  for(const el of document.querySelectorAll(selector)){
   const box=el.getBoundingClientRect(),style=getComputedStyle(el);
   if(!box.width||style.display==='contents'||style.visibility==='hidden')continue;
   const walker=document.createTreeWalker(el,NodeFilter.SHOW_TEXT);
   while(walker.nextNode()){
    const node=walker.currentNode;
    if(!node.textContent?.trim()||node.parentElement?.closest('.sr-only,.help-popover,.context-help-hidden'))continue;
    const range=document.createRange();range.selectNodeContents(node);
    for(const rect of range.getClientRects())if(rect.width&&(rect.left<box.left-1||rect.right>box.right+1||rect.top<box.top-1||rect.bottom>box.bottom+1))issues.push(`${el.className}: ${node.textContent}`);
   }
  }
  return {issues,overflow:document.documentElement.scrollWidth>innerWidth};
 });
 expect(result).toEqual({issues:[],overflow:false});
}
async function capture(page:Page,path:string){
 await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
 await page.screenshot({path,fullPage:true,animations:'disabled'});
}
for(const width of [320,390,1280])for(const lang of ['en','ar']){
 test(`road text reflows at ${width}px in ${lang} without covering art or changing state`,async({page},info)=>{
  await page.setViewportSize({width,height:900});await page.emulateMedia({reducedMotion:'reduce'});
  await baseline(page);
  await page.evaluate(lang=>localStorage.setItem('dystrail.locale',lang),lang);await page.reload();await waitForLaunch(page);
  await expect(page.locator('.turn-receipt .stat-card').first()).toBeVisible();
  const before=await checkpoint(page);
  for(const scale of [1,2]){
   if(scale===2)await enlarge(page);
   await readable(page);
   const scene=page.locator('.world-view .scene-road');
   const [hud,art,footer,sign]=await Promise.all(['.conditions-hud','.scene-art','.scene-footer','.road-billboard'].map(s=>scene.locator(s).boundingBox()));
   expect(hud!.y+hud!.height).toBeLessThanOrEqual(art!.y+1);
   expect(art!.y+art!.height).toBeLessThanOrEqual(footer!.y+1);
   expect(sign!.y).toBeGreaterThanOrEqual(art!.y);
   expect(sign!.y+sign!.height).toBeLessThanOrEqual(art!.y+art!.height);
   for(const member of await scene.locator('.standing-member').all()){
    const box=await member.boundingBox();
    expect(sign!.y+sign!.height<=box!.y||sign!.x+sign!.width<=box!.x||box!.x+box!.width<=sign!.x).toBe(true);
   }
   await capture(page,info.outputPath(`road-${width}-${lang}-${scale}.png`));
  }
  const help=page.locator('.conditions-hud .context-help:not(.context-help-hidden) .help-trigger').first();
  await help.focus();await page.keyboard.press('Enter');await expect(page.locator('.viewport-help')).toBeVisible();
  const popup=await page.locator('.viewport-help').boundingBox();expect(popup!.x).toBeGreaterThanOrEqual(0);expect(popup!.x+popup!.width).toBeLessThanOrEqual(width);
  await page.keyboard.press('Escape');await expect(page.locator('.viewport-help')).toHaveCount(0);await expect(help).toBeFocused();
  expect(await checkpoint(page)).toEqual(before);
 });
}
test('enlarged Arabic fundraiser HUD and receipt remain readable through a keyboard choice',async({page},info)=>{
 await page.setViewportSize({width:390,height:900});await page.emulateMedia({reducedMotion:'reduce'});
 const gs=await baseline(page);
 gs.current_encounter=JSON.parse(readFileSync('static/assets/data/game.json','utf8')).find((e:any)=>e.id==='classic_crossing_block_party');
 gs.driving_minutes_total=300;gs.last_encounter_driving_minutes=300;gs.clock_minutes=600;
 gs.visual_content.selections={'ENC-C03/road/300':'ENC-C03-C'};
 await importState(page,gs);await page.evaluate(()=>localStorage.setItem('dystrail.locale','ar'));await page.reload();await waitForLaunch(page);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-unit','ENC-C03-C');
 await enlarge(page);await readable(page);await capture(page,info.outputPath('fundraiser-ar-2-offer.png'));
 const choice=page.locator('.encounter-choice button').nth(2);await choice.focus();await page.keyboard.press('Enter');
 await expect(page.locator('#main')).toHaveAttribute('data-screen','aftermath');
 const result=await checkpoint(page);await page.reload();await waitForLaunch(page);await enlarge(page);
 await readable(page);await capture(page,info.outputPath('fundraiser-ar-2-outcome.png'));
 expect(await checkpoint(page)).toEqual(result);
});
