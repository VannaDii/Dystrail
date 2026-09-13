import { test, expect, Page } from '@playwright/test';
import { baseline, importState, openMenu, snap } from './helpers';
import { atTown } from './geography';

const state = (page: Page) => page.evaluate(() => JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);

async function containedHelp(page: Page) {
  const popup = page.locator('.viewport-help');
  await expect(popup).toBeVisible();
  const size = page.viewportSize()!;
  const bounds = await popup.boundingBox();
  expect(bounds!.x).toBeGreaterThanOrEqual(11);
  expect(bounds!.y).toBeGreaterThanOrEqual(11);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(size.width - 11);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(size.height - 11);
  expect(await popup.evaluate(e => e.scrollWidth <= e.clientWidth)).toBe(true);
  await expect(popup.getByRole('button', { name: 'Close', exact: true })).toBeVisible();
}

test('phone HUD stays compact and every policy has structured, read-only impact help', async ({ page }) => {
  const gs = await baseline(page);
  gs.persona_id = 'whistleblower';
  atTown(gs, 'Salt Lake City');
  gs.day = 3; gs.clock_minutes = 708; gs.stats.morale = 6;
  gs.weather_state.today = 'ColdSnap';
  gs.weather_impact = { day: 3, weather: 'ColdSnap', supplies: 0, hp: 0, sanity: -1 };
  gs.exec_order_days_remaining = 1;
  gs.inventory.tags = gs.inventory.tags.filter((tag: string) => tag !== 'legal_fund');
  for (const order of ['BookPanic', 'Shutdown', 'TravelBanLite', 'TariffTsunami', 'DoEEliminated', 'WarDeptReorg']) {
    gs.current_order = order;
    await importState(page, gs);
    const before = await state(page);
    const trigger = page.locator('.policy-indicator .help-trigger');
    const readout = page.locator('.policy-indicator > .weather-name + .sr-only');
    await expect(readout).toContainText('1 day left');
    expect((await readout.boundingBox())!.width).toBeLessThanOrEqual(1);
    await expect(page.locator('.conditions-hud .weather-impact,.conditions-hud .effect-duration')).toHaveCount(0);
    await trigger.click();
    await containedHelp(page);
    await expect(page.locator('.policy-details .effect-schedule')).toContainText('Time remaining');
    if (order === 'BookPanic') {
      await expect(page.locator('.policy-details [data-stat="ux.sanity"] strong')).toHaveText('-1');
      await expect(page.locator('.policy-details [data-stat="play.morale"] strong')).toHaveText('6');
      await snap(page, 'mobile-policy-book-panic');
      if (test.info().project.name === 'mobile') await page.screenshot({ path: 'test-results/mobile-impact-review.png', scale: 'css', animations: 'disabled' });
    }
    if (order === 'WarDeptReorg') await expect(page.locator('.policy-details [data-stat="eo.panel.breakdown"] strong')).toHaveText('+10%');
    await page.locator('.viewport-help').getByRole('button', { name: 'Close', exact: true }).click();
    await expect(trigger).toBeFocused();
    await expect(page.locator('.viewport-help')).toHaveCount(0);
    expect(await state(page)).toEqual(before);
  }
  gs.current_order = 'BookPanic';
  await importState(page, gs);
  for (const width of [320, 393, 600]) {
    await page.setViewportSize({ width, height: 740 });
    const bar = await page.locator('.conditions-hud').boundingBox();
    const effects = await page.locator('.hud-dynamic-row').boundingBox();
    expect(effects!.width).toBeGreaterThan(bar!.width * 0.9);
    expect(Math.abs(effects!.x - bar!.x)).toBeLessThan(18);
    expect(bar!.height).toBeLessThanOrEqual(110);
    const clock = (await page.locator('.hud-context').boundingBox())!;
    for (const part of ['.leg-cash', '.leg-vehicle']) {
      const bounds = (await page.locator(part).boundingBox())!;
      expect(Math.abs(bounds.y - clock.y)).toBeLessThan(2);
    }
    for (const part of ['.leg-destination', '.weather-indicator:not(.policy-indicator)', '.policy-indicator']) {
      const bounds = (await page.locator(part).boundingBox())!;
      expect(bounds.y).toBeGreaterThan(clock.y + 20);
      expect(bounds.x).toBeGreaterThanOrEqual(0);
      expect(bounds.x + bounds.width).toBeLessThanOrEqual(width);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    if (width === 393 && test.info().project.name === 'mobile') await page.screenshot({ path: 'test-results/mobile-status-review.png', scale: 'css', animations: 'disabled' });
  }
  await snap(page, 'mobile-hud-effects');
  await openMenu(page);
  const beforeHelp = await page.locator('.conditions-hud').boundingBox();
  await page.getByRole('switch', { name: 'Help & tips', exact: true }).click();
  expect(await page.locator('.conditions-hud').boundingBox()).toEqual(beforeHelp);
  await page.locator('#game-menu-button').click();
  await page.locator('.policy-indicator .help-trigger').click();
  await containedHelp(page);
  await page.locator('.viewport-help').getByRole('button', { name: 'Close', exact: true }).click();
  const trigger = page.locator('.policy-indicator .help-trigger');
  await trigger.focus();
  await page.keyboard.press('Enter');
  await page.keyboard.press('Tab');
  await expect(page.locator('.viewport-help .help-dismiss')).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(trigger).toBeFocused();
});

test('weather and resource help fit short phone viewports and close without spending time', async ({ page }) => {
  const gs = await baseline(page);
  await page.setViewportSize({ width: 320, height: 480 });
  for (const weather of ['Clear', 'HeatWave', 'ColdSnap', 'Smoke', 'Storm']) {
    gs.weather_state.today = weather;
    gs.weather_impact = { day: gs.day, weather, supplies: weather === 'Clear' ? 0 : -1, hp: 0, sanity: weather === 'Clear' ? 0 : -1 };
    await importState(page, gs);
    const before = await state(page);
    await page.locator('.weather-indicator:not(.policy-indicator) .help-trigger').click();
    await containedHelp(page);
    await expect(page.locator('.weather-details')).toBeVisible();
    await page.locator('.viewport-help').evaluate(e => { e.scrollTop = e.scrollHeight; });
    const close = await page.locator('.viewport-help .help-dismiss').boundingBox();
    expect(close!.y).toBeGreaterThanOrEqual(11);
    expect(close!.y + close!.height).toBeLessThanOrEqual(469);
    await page.locator('.viewport-help').getByRole('button', { name: 'Close', exact: true }).click();
    expect(await state(page)).toEqual(before);
  }
  await page.setViewportSize({ width: 393, height: 740 });
  await openMenu(page);
  const tips = page.getByRole('switch', { name: 'Help & tips', exact: true });
  if (await tips.getAttribute('aria-checked') !== 'true') await tips.click();
  await page.locator('#game-menu-button').click();
  const before = await state(page);
  for (const trigger of await page.locator('.critical-stats .help-trigger').all()) {
    await trigger.click(); await containedHelp(page);
    await page.keyboard.press('Escape');
    await expect(page.locator('.viewport-help')).toHaveCount(0);
  }
  expect(await state(page)).toEqual(before);
});


test('clear-weather phone strip uses two rows with icon-and-value resource readouts', async ({ page }) => {
  const gs = await baseline(page);
  gs.persona_id = 'whistleblower';
  atTown(gs, 'Des Moines');
  gs.day = 8; gs.clock_minutes = 555;
  gs.budget_cents = 8300; gs.vehicle.health = 96;
  gs.current_order = null; gs.exec_order_days_remaining = 0;
  gs.weather_state.today = 'Clear'; gs.weather_impact = null;
  await importState(page, gs);
  for (const width of [393, 430, 600]) {
    await page.setViewportSize({ width, height: 740 });
    expect((await page.locator('.conditions-hud').boundingBox())!.height).toBeLessThanOrEqual(80);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  await expect(page.locator('.hud-trip-context .leg-cash strong')).toHaveText('$83');
  await expect(page.locator('.hud-trip-context .leg-vehicle strong')).toHaveText('96%');
  for (const label of await page.locator('.hud-trip-context .leg-indicator .sr-only').all()) {
    expect((await label.boundingBox())!.width).toBeLessThanOrEqual(1);
  }
  await page.locator('.weather-indicator .help-trigger').click();
  await containedHelp(page);
  await expect(page.locator('.weather-details')).toContainText('No weather cost recorded yet.');
});
