import { expect, Page, test } from '@playwright/test';
import { openMenu, snap, waitForLaunch } from './helpers';

const crewNames = (page: Page) => page.locator('.crew-name-form input').evaluateAll(inputs =>
  Object.fromEntries(inputs.map(input => [(input as HTMLInputElement).id, (input as HTMLInputElement).value])));

async function checkActionOrder(page: Page, selector = '.crew-actions', rtl = false) {
  const actions = page.locator(selector);
  const buttons = actions.locator('button:visible');
  const back = actions.locator('.retro-btn-secondary:visible');
  const next = actions.locator('.retro-btn-primary');
  const mobile = page.viewportSize()!.width <= 700;
  await expect(buttons).toHaveCount(2);
  await expect(buttons.nth(mobile ? 0 : 1)).toHaveClass(/retro-btn-primary/);
  const backBox = (await back.boundingBox())!;
  const nextBox = (await next.boundingBox())!;
  if (mobile) {
    expect(nextBox.y + nextBox.height).toBeLessThanOrEqual(backBox.y);
    expect(Math.abs(nextBox.x - backBox.x)).toBeLessThan(1);
    expect(Math.abs(nextBox.width - backBox.width)).toBeLessThan(1);
  } else {
    expect(Math.abs(nextBox.y - backBox.y)).toBeLessThan(1);
    if (rtl) expect(nextBox.x + nextBox.width).toBeLessThanOrEqual(backBox.x);
    else expect(backBox.x + backBox.width).toBeLessThanOrEqual(nextBox.x);
    const edges = await actions.evaluate(element => {
      const box = element.getBoundingClientRect(), style = getComputedStyle(element);
      return { left: box.left + parseFloat(style.paddingLeft), right: box.right - parseFloat(style.paddingRight) };
    });
    const left = rtl ? nextBox : backBox, right = rtl ? backBox : nextBox;
    expect(Math.abs(left.x - edges.left)).toBeLessThan(1);
    expect(Math.abs(right.x + right.width - edges.right)).toBeLessThan(1);
    expect(Math.abs(nextBox.width - backBox.width)).toBeLessThan(1);
  }
  await page.locator(selector === '.crew-actions' ? '.crew-name-card input' : '.persona-tile').last().focus();
  await page.keyboard.press('Tab');
  await expect(buttons.nth(0)).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(buttons.nth(1)).toBeFocused();
}

test('setup Back preserves mode, character, and crew with desktop, phone, and RTL keyboard order', async ({ page }) => {
  await page.goto('./');
  await waitForLaunch(page);
  await page.getByRole('radio', { name: /The Deep End/ }).check();
  const code = await page.locator('#run-code').inputValue();
  await page.getByRole('button', { name: 'Choose your character', exact: true }).click();
  await page.getByRole('radio', { name: 'Journalist', exact: true }).click();
  await expect(page.locator('#persona-helper')).toHaveClass('sr-only');
  await expect(page.locator('#persona-helper')).toHaveAttribute('aria-live', 'polite');
  await expect(page.locator('.persona-actions #persona-helper')).toHaveCount(0);
  await checkActionOrder(page, '.persona-actions');
  await snap(page, 'persona-navigation');
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'setup');
  await expect(page.getByRole('radio', { name: /The Deep End/ })).toBeChecked();
  await expect(page.locator('#run-code')).toHaveValue(code);
  await page.reload();
  await waitForLaunch(page);
  await expect(page.getByRole('radio', { name: /The Deep End/ })).toBeChecked();
  await page.getByRole('button', { name: 'Choose your character', exact: true }).click();
  await expect(page.getByRole('radio', { name: 'Journalist', exact: true })).toHaveAttribute('aria-checked', 'true');
  await page.getByRole('button', { name: 'Continue', exact: true }).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'crew');
  const back = page.getByRole('button', { name: 'Back', exact: true });
  const next = page.getByRole('button', { name: 'Continue', exact: true });
  await expect(next).toBeDisabled();
  await expect(back).toBeEnabled();
  await back.click();
  await expect(page).toHaveURL(/\/play\/persona$/);
  await expect(page.getByRole('radio', { name: 'Journalist', exact: true })).toHaveAttribute('aria-checked', 'true');
  await next.click();
  await page.getByLabel('Your name', { exact: true }).fill('Vanna Full Name');
  await page.getByLabel('Crew name', { exact: true }).fill('The Good Trouble');
  await page.getByLabel('Organizer', { exact: true }).fill('Alex Morgan');
  const names = await crewNames(page);
  await checkActionOrder(page);
  await snap(page, 'crew-navigation');
  await back.focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/play\/persona$/);
  await back.click();
  await expect(page.locator('#run-code')).toHaveValue(code);
  await page.getByRole('button', { name: 'Choose your character', exact: true }).click();
  await expect(page.getByRole('radio', { name: 'Journalist', exact: true })).toHaveAttribute('aria-checked', 'true');
  await next.click();
  await expect(page).toHaveURL(/\/play\/crew$/);
  expect(await crewNames(page)).toEqual(names);

  await back.click();
  await page.getByRole('radio', { name: 'Organizer', exact: true }).click();
  await next.click();
  await expect(page.getByLabel('Your name', { exact: true })).toHaveValue('Alex Morgan');
  expect(await crewNames(page)).toEqual(names);
  await expect.poll(() => page.evaluate(() =>
    JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).pending.party.leader)).toBe('Alex Morgan');
  await page.reload();
  await waitForLaunch(page);
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'crew');
  expect(await crewNames(page)).toEqual(names);

  await openMenu(page);
  await page.locator('.language-picker>button').click();
  await page.getByRole('option', { name: 'العربية', exact: true }).click();
  await page.locator('#game-menu-button').click();
  await expect(page.locator('html')).toHaveAttribute('dir', 'rtl');
  await expect(page.locator('.crew-actions')).not.toContainText(/store\.menu\.back|ui\.continue|Back|Continue/);
  await checkActionOrder(page, '.crew-actions', true);
  await snap(page, 'crew-navigation-arabic');
  await page.locator('.crew-actions .retro-btn-secondary:visible').click();
  await checkActionOrder(page, '.persona-actions', true);
  await snap(page, 'persona-navigation-arabic');
  await page.locator('#persona-continue').click();
  await page.locator('.crew-actions .retro-btn-primary').click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'outfitting');
});
