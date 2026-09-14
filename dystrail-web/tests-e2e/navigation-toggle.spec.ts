import {test,expect,Page} from '@playwright/test';
import {baseline,importState,snap,waitForLaunch} from './helpers';
import {atTown} from './geography';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));
const comparable=(state:any)=>({...state,inventory:{...state.inventory,tags:[...state.inventory.tags].sort()}});

for(const screen of ['travel','camp','town']) test(`manual Route closes back to ${screen} and never starts travel`,async({page})=>{
 const base=await baseline(page);
 const gs=structuredClone(base);if(screen==='town')atTown(gs,'Spokane');await importState(page,gs);
 if(screen==='camp')await page.getByRole('button',{name:'Camp',exact:true}).click();
 await page.getByRole('tab',{name:'Journal',exact:true}).click();
 const before=await checkpoint(page);
 await page.getByRole('button',{name:'Route',exact:true}).click();
 await expect(page.locator('.map-scene')).toHaveAttribute('data-automatic','false');
 await expect(page.getByRole('button',{name:'Close map',exact:true})).toBeVisible();
 await expect(page.locator('.map-countdown')).toHaveCount(0);
 await page.reload();await waitForLaunch(page);
 await page.getByRole('button',{name:'Close map',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen',screen);
 await expect(page.getByRole('tab',{name:'Journal',exact:true})).toHaveAttribute('aria-selected','true');
 await expect(page.getByRole('tabpanel',{name:'Journal',exact:true})).toBeVisible();
 await page.waitForTimeout(3300);
 const after=await checkpoint(page);expect(comparable(after.state)).toEqual(comparable(before.state));expect(after.aftermath).toEqual(before.aftermath);
 await expect(page.getByRole('button',{name:'Pause travel',exact:true})).toHaveCount(0);
 await snap(page,`manual-map-return-to-${screen}`);
});

test('every selected tab toggles its content without spending a turn and remains keyboard accessible',async({page})=>{
 await baseline(page);const before=await checkpoint(page);
 for(const name of ['The Trail','Conditions','The Van','Journal']){
  const tab=page.getByRole('tab',{name,exact:true});
  if(await tab.getAttribute('aria-selected')!=='true')await tab.click();
  await expect(page.getByRole('tabpanel',{name,exact:true})).toBeVisible();
  await tab.click();await expect(page.getByRole('tabpanel')).toHaveCount(0);await expect(page.locator('.travel-tabs [aria-selected=true]')).toHaveCount(0);
  await expect(tab).toHaveAttribute('aria-expanded','false');await expect(tab).toHaveAttribute('tabindex','0');
  await tab.focus();await page.keyboard.press('Enter');await expect(tab).toHaveAttribute('aria-selected','true');await expect(page.getByRole('tabpanel',{name,exact:true})).toBeVisible();
 }
 const journal=page.getByRole('tab',{name:'Journal',exact:true});await journal.click();await page.reload();await waitForLaunch(page);
 await expect(page.getByRole('tabpanel')).toHaveCount(0);await expect(journal).toHaveAttribute('tabindex','0');
 await journal.focus();await page.keyboard.press('ArrowLeft');await expect(page.getByRole('tab',{name:'The Van',exact:true})).toBeFocused();await expect(page.getByRole('tabpanel',{name:'The Van',exact:true})).toBeVisible();
 const after=await checkpoint(page);expect(comparable(after.state)).toEqual(comparable(before.state));
 await page.getByRole('tab',{name:'The Van',exact:true}).click();await snap(page,'collapsed-journey-tabs');
});
