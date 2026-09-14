import {test,expect,Page} from '@playwright/test';
import { setup,depart,baseline,importState,savedState,openMenu,snap, waitForLaunch } from './helpers';
import {atTown} from './geography';

async function checkoutFits(page:Page) {
 const bounds=await page.locator('.loadout-panel').evaluate(footer=>{
  const r=footer.getBoundingClientRect();
  return [...footer.querySelectorAll('button')].map(button=>{
   const b=button.getBoundingClientRect();
   const target=document.elementFromPoint(b.x+b.width/2,b.y+b.height/2);
   return {contained:b.left>=r.left&&b.right<=r.right,receivesPointer:!!target&&(button===target||button.contains(target))};
  });
 });
 for(const button of bounds){expect(button.contained).toBe(true);expect(button.receivesPointer).toBe(true);}
}

test('illustrated outfitting shows every item without filters and supports exact direct purchases',async({page},info)=>{
 await setup(page);
 await expect(page.locator('.store-card')).toHaveCount(11);
 await expect(page.locator('.category-tabs')).toHaveCount(0);
 await expect(page.locator('.store-item-art')).toHaveCount(11);
 expect(await page.locator('.store-item-art').evaluateAll(images=>images.every(e=>{const i=e as HTMLImageElement;return i.complete&&i.naturalWidth>0&&i.src.startsWith('blob:')}))).toBe(true);
 expect(await page.locator('.store-item-art').evaluateAll(images=>new Set(images.map(e=>(e as HTMLImageElement).src)).size)).toBe(11);
 const height=await page.locator('.outfit-workspace').evaluate(e=>e.getBoundingClientRect().height);
 expect(height).toBeLessThan(info.project.name==='mobile'?1700:1200);
 const rations=page.getByRole('spinbutton',{name:'Rations Pack Quantity',exact:true});
 await rations.fill('2');await expect(rations).toHaveValue('2');
 await expect(page.locator('.cart-total')).toHaveText('$61');await expect(page.locator('.cart-cash')).toHaveText('$59');
 await rations.press('ArrowUp');await expect(rations).toHaveValue('3');
 await expect(page.locator('.outfit-category')).toHaveCount(4);
 await page.reload();await waitForLaunch(page);await expect(rations).toHaveValue('3');
 await snap(page,'compact-outfitting');
 await checkoutFits(page);
 await depart(page);const gs=await savedState(page);
 expect(gs.budget_cents).toBe(5400);expect(gs.stats.supplies).toBe(13);
 expect(gs.inventory.spares).toEqual({tire:1,battery:1,alt:0,pump:0});
});

test('quantity entry cannot overspend and clearing a selected item restores its funds',async({page})=>{
 await setup(page);await page.getByRole('button',{name:'Empty the van',exact:true}).click();
 const rations=page.getByRole('group',{name:'Rations Pack',exact:true}),quantity=rations.getByRole('spinbutton');
 await quantity.fill('99');await expect(quantity).toHaveValue('24');await expect(page.locator('.cart-cash')).toHaveText('$0');
 await expect(rations.getByRole('button',{name:'Add +1',exact:true})).toBeDisabled();
 await expect(page.locator('.capacity-warning')).toBeVisible();
 await quantity.fill('0');await expect(page.locator('.cart-total')).toHaveText('$0');await expect(page.locator('.cart-cash')).toHaveText('$120');
 await expect(page.locator('.store-item-selected')).toHaveCount(0);
 await page.getByRole('spinbutton',{name:'Press Pass Quantity',exact:true}).fill('50');
 await expect(page.getByRole('spinbutton',{name:'Press Pass Quantity',exact:true})).toHaveValue('1');
});

test('town store shows carried parts, buys once with the shown total, and keeps inventory available',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Spokane');gs.budget_cents=6000;gs.stats.supplies=8;
 gs.inventory.spares={tire:2,battery:0,alt:0,pump:1};await importState(page,gs);
 await expect(page.locator('.town-vitals')).toContainText('Population estimate (2025)');
 await expect(page.locator('.town-vitals a')).toHaveText('230,783');
 await expect(page.locator('.town-vitals a')).toHaveAttribute('href',/\/PST045225$/);
 await expect(page.getByRole('button',{name:'Leave town',exact:true})).toHaveCount(1);
 await expect(page.getByRole('button',{name:'Leave town',exact:true})).toBeEnabled();
 await expect(page.locator('.town-depart')).toHaveCount(0);
 await snap(page,'town-single-departure');
 await page.getByRole('button',{name:'Visit the Store',exact:true}).click();
 await expect(page.getByRole('tab',{name:'The Van',exact:true})).toBeVisible();
 await expect(page.locator('.outfit-workspace>.leg-summary')).toHaveCount(0);
 await expect(page.getByRole('group',{name:'Spare Tire',exact:true})).toContainText('In van: 2');
 await expect(page.getByRole('group',{name:'Battery',exact:true})).toContainText('In van: 0');
 await expect(page.getByRole('button',{name:'Buy supplies & return',exact:true})).toBeDisabled();
 await page.getByRole('spinbutton',{name:'Battery Quantity',exact:true}).fill('1');
 await page.getByRole('spinbutton',{name:'Rations Pack Quantity',exact:true}).fill('1');
 await page.getByRole('spinbutton',{name:'Water Jugs Quantity',exact:true}).fill('2');
 await expect(page.locator('.cart-total')).toHaveText('$35');await expect(page.locator('.cart-cash')).toHaveText('$25');
 await expect(page.locator('.store-cart-summary')).toContainText('15 / 20');
 await snap(page,'compact-town-store');
 await checkoutFits(page);
 await page.getByRole('button',{name:'Buy supplies & return',exact:true}).click();
 await expect(page.locator('.town-arrival')).toBeVisible();
 const bought=await savedState(page);expect(bought.budget_cents).toBe(2500);expect(bought.stats.supplies).toBe(15);
 expect(bought.inventory.spares).toEqual({tire:2,battery:1,alt:0,pump:1});expect(bought.clock_minutes).toBe(gs.clock_minutes+30);
 await page.reload();await waitForLaunch(page);expect((await savedState(page)).budget_cents).toBe(2500);
 await page.getByRole('button',{name:'Visit the Store',exact:true}).click();
 await expect(page.getByRole('group',{name:'Battery',exact:true})).toContainText('In van: 1');
 await expect(page.locator('.cart-total')).toHaveText('$0');
});

test('shopping remains readable in Arabic and Help & tips never shifts item controls',async({page})=>{
 await setup(page);
 await page.getByRole('spinbutton',{name:'Rations Pack Quantity',exact:true}).fill('2');
 await expect(page.locator('.cart-total')).toHaveText('$61');
 await openMenu(page);
 const layout=()=>page.locator('.outfit-workspace').evaluate(e=>[...e.querySelectorAll('.store-card, .store-qty-row, .outfit-heading, .loadout-panel')].map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};}));
 const before=await layout();await page.getByRole('switch',{name:'Help & tips',exact:true}).click();expect(await layout()).toEqual(before);
 await page.getByRole('button',{name:'Language',exact:true}).click();await page.getByRole('option',{name:'العربية',exact:true}).click();
 await page.locator('#game-menu-button').click();
 await expect(page.locator('html')).toHaveAttribute('dir','rtl');
 await expect(page.locator('.store-cart-summary')).toContainText('النقد بعد الشراء');
 await expect(page.locator('.store-price').first()).toContainText('للقطعة');
 await expect(page.locator('input[aria-labelledby^="store-item-rations "]')).toHaveValue('2');
 const localizedTotal=await page.evaluate(()=>new Intl.NumberFormat('ar',{style:'currency',currency:'USD',minimumFractionDigits:0,maximumFractionDigits:0}).format(61));
 await expect(page.locator('.cart-total')).toHaveText(localizedTotal);
 await expect(page.locator('.outfit-workspace')).not.toContainText('shopping.');
 const row=await page.locator('.store-card').first().evaluate(e=>{const a=e.querySelector('.store-card-body')!.getBoundingClientRect(),b=e.querySelector('.store-qty-row')!.getBoundingClientRect();return {nameLeft:a.left,nameTop:a.top,nameBottom:a.bottom,controlsRight:b.right,controlsTop:b.top,controlsBottom:b.bottom};});
 const sameRow=Math.min(row.nameBottom,row.controlsBottom)>Math.max(row.nameTop,row.controlsTop);
 if(sameRow)expect(row.controlsRight).toBeLessThanOrEqual(row.nameLeft);
 else expect(row.controlsTop).toBeGreaterThanOrEqual(row.nameBottom);
 await snap(page,'compact-store-arabic');
});
