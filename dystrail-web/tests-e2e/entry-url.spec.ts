import { test, expect } from '@playwright/test';
import { baseline, waitForLaunch } from './helpers';

test('bare game URL adds its slash before offline preparation and preserves URL details', async ({ page, baseURL }) => {
  const bare = new URL(baseURL!);
  bare.pathname = bare.pathname.replace(/\/$/, '');
  bare.search = '?review=entry%20route';
  bare.hash = '#arrival';
  const canonical = new URL(bare);
  canonical.pathname += '/';
  await page.goto(bare.href);
  await expect(page).toHaveURL(canonical.href);
  await waitForLaunch(page);
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'setup');
  expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
  await expect(page.locator('#launch-gate')).toHaveCount(0);
});

test('bare game URL resumes a saved journey and can then reopen offline', async ({ page, context, baseURL }) => {
  await baseline(page);
  const readSave = () => page.evaluate(() => {
    const save = JSON.parse(localStorage.getItem('dystrail.autosave.v1')!);
    for (const key of ['state', 'pending']) save[key]?.inventory.tags.sort();
    return save;
  });
  const before = await readSave();
  await page.goto(baseURL!.replace(/\/$/, ''));
  await expect(page).toHaveURL(/\/play\//);
  await waitForLaunch(page);
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'travel');
  expect(await readSave()).toEqual(before);
  await context.setOffline(true);
  await page.reload();
  await waitForLaunch(page);
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'travel');
  expect(await readSave()).toEqual(before);
});
