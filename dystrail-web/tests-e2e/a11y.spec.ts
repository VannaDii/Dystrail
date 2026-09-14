import {test,expect} from '@playwright/test';
import {snap,baseline,openMenu,waitForLaunch} from './helpers';
test('focus stays still, keyboard selection, rich locale, contrast and outside dismissal',async({page})=>{
 await page.goto('./');await waitForLaunch(page);await expect(page.locator('#main')).toHaveAttribute('data-screen','setup');
 expect(await page.evaluate(()=>document.activeElement?.tagName)).toBe('BODY');expect(await page.evaluate(()=>scrollY)).toBe(0);
 await expect(page.locator('#game-menu-button')).toBeVisible();
 await openMenu(page);await page.locator('.language-picker>button').click();await expect(page.getByRole('listbox')).toBeVisible();
 await page.locator('.wordmark').click();await expect(page.getByRole('listbox')).toHaveCount(0);
 await openMenu(page);await page.locator('.language-picker>button').click();await page.getByRole('option',{name:'العربية',exact:true}).click();
 await expect(page.locator('html')).toHaveAttribute('dir','rtl');await page.reload();await waitForLaunch(page);await expect(page.locator('html')).toHaveAttribute('lang','ar');await snap(page,'rtl');
 await openMenu(page);await page.locator('.language-picker>button').click();await page.getByRole('option',{name:'English',exact:true}).click();
 await page.getByRole('switch',{name:'High contrast',exact:true}).click();await expect(page.getByRole('switch',{name:'High contrast',exact:true})).toHaveAttribute('aria-checked','true');await expect(page.locator('html')).toHaveClass(/hc/);
 await page.locator('#save-open-btn').click();await expect(page.locator('.drawer')).toBeVisible();await page.locator('.drawer').click({position:{x:2,y:2}});await expect(page.locator('.drawer')).toHaveCount(0);
 await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 await page.locator('body').click({position:{x:2,y:2}});await page.keyboard.press('1');await expect(page.getByRole('radio',{name:'Journalist',exact:true})).toHaveAttribute('aria-checked','true');
 await page.keyboard.press('ArrowRight');await expect(page.getByRole('radio',{name:'Organizer',exact:true})).toBeFocused();
 await page.reload();await waitForLaunch(page);await expect(page.getByRole('radio',{name:'Organizer',exact:true})).toHaveAttribute('aria-checked','true');
 expect(await page.evaluate(()=>document.activeElement?.tagName)).toBe('BODY');
});
test('context help and reduced motion',async({page})=>{
 await page.emulateMedia({reducedMotion:'reduce'});await baseline(page);
 for(const label of await page.locator('.hud-stat dt').all()) expect(await label.evaluate(el=>el.scrollWidth<=el.clientWidth+1)).toBe(true);
 await page.getByRole('button',{name:'How this works: Credibility',exact:true}).click();await expect(page.locator('.help-popover')).toBeVisible();
 await page.locator('.wordmark').click();await expect(page.locator('.help-popover')).toHaveCount(0);
 await page.getByRole('button',{name:'Travel',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','traveling');
 expect(await page.locator('.journey-scene .scene-background').first().evaluate(el=>getComputedStyle(el).animationName)).toBe('none');
});
