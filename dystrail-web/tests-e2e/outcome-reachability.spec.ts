import {test,expect,Page} from '@playwright/test';
import {baseline,importState,openMenu,snap} from './helpers';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));

test('outcome exit stays reachable at every scroll position and with a short viewport',async({page})=>{
 const gs=await baseline(page);gs.breakdown={part:'Battery',day_started:gs.day};gs.vehicle.health=93.27;gs.inventory.spares.battery=1;
 await importState(page,gs);await page.getByRole('button',{name:'Fit your spare Battery',exact:true}).click();
 await expect(page.locator('.aftermath-panel')).toBeVisible();
 const resolved=await checkpoint(page);
 // A saved, long report exercises the same outcome view without changing the completed action.
 await page.evaluate(()=>{
  const saved=JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);
  saved.aftermath.message=Array(10).fill('The battery is fitted. Everyone checks the van and prepares for the next stretch of road.').join(' ');
  localStorage.setItem('dystrail.autosave.v1',JSON.stringify(saved));
 });
 await page.reload();
 const restored=await checkpoint(page);
 // Inventory tags are a set; their serialized order may change on reload.
 resolved.state.inventory.tags.sort();restored.state.inventory.tags.sort();
 expect(restored.state).toEqual(resolved.state);
 const button=page.getByRole('button',{name:'Back to the road',exact:true});
 const width=page.viewportSize()!.width;
 for(const height of [800,600,420]){
  await page.setViewportSize({width,height});
  for(const fraction of [0,.5,1]){
   await page.evaluate(f=>scrollTo(0,(document.documentElement.scrollHeight-innerHeight)*f),fraction);
   await expect(button).toBeInViewport({ratio:1});
   expect(await button.evaluate(e=>{const r=e.getBoundingClientRect();return document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.closest('button')===e;})).toBe(true);
  }
 }
 await snap(page,'reachable-long-outcome');
 await page.setViewportSize({width,height:800});await page.evaluate(()=>scrollTo(0,0));
 await openMenu(page);await expect(page.getByRole('button',{name:'Save',exact:true})).toBeVisible();await page.locator('#game-menu-button').click();
 await button.focus();await page.keyboard.press('Enter');
 await expect(page.locator('.outcome-screen')).toHaveCount(0);
 const continued=await checkpoint(page);continued.state.inventory.tags.sort();expect(continued.state).toEqual(resolved.state);expect(continued.aftermath).toBeNull();
 await expect(page.getByRole('button',{name:'Travel',exact:true})).toBeEnabled();
});
