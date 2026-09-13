import { expect, Page, test } from '@playwright/test';
export async function setup(page:Page,deep=false) {
  await page.goto('./');
  if(deep) await page.getByRole('radio',{name:/The Deep End/}).check();
  await page.getByRole('button',{name:'Choose your character',exact:true}).click();
  await page.getByRole('radio',{name:'Journalist',exact:true}).click();
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','crew');
  await page.getByLabel('Your name',{exact:true}).fill('Vanna Test');
  await page.getByLabel('Crew name',{exact:true}).fill('The Receipts');
  await page.getByRole('button',{name:'Continue',exact:true}).click();
  await expect(page.locator('.store-card').first()).toBeVisible();
}
export async function depart(page:Page) {
  await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();
  await page.getByRole('button',{name:'Start the journey',exact:true}).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');
}
export async function savedState(page:Page) {
  await openMenu(page);
  await page.getByRole('button',{name:'Save',exact:true}).click();
  return page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.save.default')!));
}
export async function baseline(page:Page,deep=false) {await setup(page,deep);await depart(page);return savedState(page);}
export async function importState(page:Page,state:object) {
  await openMenu(page);
  await page.locator('#save-open-btn').click();
  const backupText = page.locator('.save-text-backup');
  if (await backupText.getAttribute('open') === null) await backupText.locator('summary').click();
  await page.locator('#import-json').fill(JSON.stringify(state));
  await page.getByRole('button',{name:'Import',exact:true}).click();
  await expect(page.locator('.drawer')).toHaveCount(0);
}
export async function snap(page:Page,name:string) {
  for(const img of await page.locator('img:visible').all()) await img.evaluate((img:HTMLImageElement)=>img.decode());
  await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
  await page.screenshot({path:`test-results/${name}-${test.info().project.name}.png`,fullPage:true,animations:'disabled'});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
}
export async function settle(page:Page) {
  await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');
  await page.getByRole('button',{name:'Pause travel',exact:true}).click();
  await expect(page.locator('#main')).not.toHaveAttribute('data-screen','traveling');
  if(await page.locator('.map-scene').count()){const automatic=await page.locator('.map-scene').getAttribute('data-automatic')==='true';await page.getByRole('button',{name:automatic?'Resume travel':'Close map',exact:true}).click();}
}

export async function openMenu(page:Page) {const menu=page.locator("#game-menu-button");if(await menu.getAttribute("aria-expanded")!=="true")await menu.click();}

export async function fastMode(page:Page, enabled:boolean) {
 const toggle=page.getByRole('switch',{name:'Fast mode',exact:true});
 if(await toggle.getAttribute('aria-checked')!==String(enabled)) await toggle.click();
 await expect(toggle).toHaveAttribute('aria-checked',String(enabled));
}
