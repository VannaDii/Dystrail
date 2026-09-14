import { test, expect, Page } from '@playwright/test';
import { openMenu, waitForLaunch } from './helpers';

async function menuGeometry(page: Page) {
  return page.locator('#game-menu-panel').evaluate(element => {
    const bounds = element.getBoundingClientRect();
    return { x: bounds.x, y: bounds.y, width: bounds.width, height: bounds.height,
      content: element.scrollHeight, scroll: element.scrollTop, pageScroll: scrollY };
  });
}

async function expectFloatingList(page: Page) {
  await expect(page.getByRole('listbox')).toBeVisible();
  const geometry = await page.locator('.language-options').evaluate(element => {
    const list = element.getBoundingClientRect();
    const menu = document.querySelector('#game-menu-panel')!.getBoundingClientRect();
    const button = document.querySelector('.language-picker>button')!.getBoundingClientRect();
    return { topLayer: element.matches(':popover-open'), x: list.x, y: list.y,
      right: list.right, bottom: list.bottom, width: list.width,
      buttonWidth: button.width, below: Math.abs(list.top - button.bottom - 6) < 1,
      above: Math.abs(list.bottom - button.top + 6) < 1,
      outside: list.bottom > menu.bottom || list.top < menu.top,
      viewportWidth: innerWidth, viewportHeight: innerHeight };
  });
  expect(geometry.topLayer).toBe(true);
  expect(geometry.below || geometry.above).toBe(true);
  expect(geometry.width).toBeCloseTo(geometry.buttonWidth, 0);
  expect(geometry.x).toBeGreaterThanOrEqual(11);
  expect(geometry.y).toBeGreaterThanOrEqual(11);
  expect(geometry.right).toBeLessThanOrEqual(geometry.viewportWidth - 11);
  expect(geometry.bottom).toBeLessThanOrEqual(geometry.viewportHeight - 11);
  return geometry;
}

for (const deep of [false, true]) {
  test(`language list floats beyond the menu without resizing or double scrolling: ${deep ? 'deep' : 'classic'}`, async ({ page }, info) => {
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto('./');
    await waitForLaunch(page);
    if (deep) await page.getByRole('radio', { name: /The Deep End/ }).check();
    await openMenu(page);
    const before = await menuGeometry(page);
    const trigger = page.locator('.language-picker>button');
    await trigger.click();
    expect((await expectFloatingList(page)).outside).toBe(true);
    expect(await menuGeometry(page)).toEqual(before);
    await expect(page.getByRole('option')).toHaveCount(20);
    await expect(page.getByRole('option', { name: 'English', exact: true })).toBeFocused();
    await page.screenshot({ path: info.outputPath('language-floating.png') });

    // Every language must be visible and clickable when reached by keyboard.
    for (let index = 0; index < 20; index++) {
      const option = page.getByRole('option').nth(index);
      await expect(option).toBeFocused();
      expect(await option.evaluate(element => {
        const rect = element.getBoundingClientRect();
        const list = element.parentElement!.getBoundingClientRect();
        return rect.top >= list.top && rect.bottom <= list.bottom
          && element.contains(document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2));
      })).toBe(true);
      if (index < 19) await page.keyboard.press('ArrowDown');
    }
    expect(await menuGeometry(page)).toEqual(before);
    await page.keyboard.press('Home');
    await expect(page.getByRole('option', { name: 'English', exact: true })).toBeFocused();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('listbox')).toHaveCount(0);
    await expect(trigger).toBeFocused();
    await expect(page.locator('#game-menu-button')).toHaveAttribute('aria-expanded', 'true');
    await page.keyboard.press('Escape');
    await expect(page.locator('#game-menu-panel')).toHaveCount(0);
    await expect(page.locator('#game-menu-button')).toBeFocused();

    await openMenu(page);
    await trigger.click();
    await page.getByRole('option', { name: 'العربية', exact: true }).click();
    await expect(page.locator('html')).toHaveAttribute('dir', 'rtl');
    await expect(trigger).toBeFocused();
    await page.reload();
    await waitForLaunch(page);
    await expect(page.locator('html')).toHaveAttribute('lang', 'ar');
    await openMenu(page);
    await trigger.click();
    await expectFloatingList(page);

    // Resizing and a larger header must retain a usable list outside the scroll container.
    await page.setViewportSize({ width: page.viewportSize()!.width, height: 430 });
    await page.locator('.game-header').evaluate(element => (element as HTMLElement).style.minHeight = '140px');
    await expect(async () => { await expectFloatingList(page); }).toPass({ timeout: 5000 });
    const shortMenu = await menuGeometry(page);
    await page.keyboard.press('End');
    await expect(page.getByRole('option', { name: 'Türkçe', exact: true })).toBeFocused();
    expect(await menuGeometry(page)).toEqual(shortMenu);
    await page.screenshot({ path: info.outputPath('language-arabic-short.png') });
    await page.mouse.click(2, 2);
    await expect(page.getByRole('listbox')).toHaveCount(0);
    await expect(page.locator('#game-menu-panel')).toHaveCount(0);
    expect(errors).toEqual([]);
  });
}
