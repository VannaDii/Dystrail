import { test, expect, type Page } from '@playwright/test';

async function waitForBridge(page: Page) {
  await page.waitForFunction(
    () =>
      typeof window !== 'undefined' &&
      (window as any).__dystrailTest &&
      typeof (window as any).__dystrailTest.screen === 'function'
  );
}

async function setScreen(page: Page, name: string) {
  const result = await page.evaluate((screen) => {
    const bridge = (window as any).__dystrailTest;
    if (bridge && typeof bridge.screen === 'function') {
      try {
        bridge.screen(screen);
        return { ok: true };
      } catch (error) {
        return { ok: false, error: String(error) };
      }
    }
    return { ok: false, error: 'test bridge unavailable' };
  }, name);
  if (!result.ok) {
    throw new Error(`bridge.screen("${name}") failed: ${result.error ?? 'unknown error'}`);
  }
}

test.describe('Dystrail screen routing coverage', () => {
  test('screens match UX inventory', async ({ page }) => {
    page.on('pageerror', (err) => {
      console.error('pageerror:', err.message);
    });
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        console.error('console error:', msg.text());
      }
    });
    await page.addInitScript(() => {
      localStorage.setItem('dystrail.locale', 'en');
      localStorage.setItem('dystrail.hc', '0');
    });
    await page.goto('./?test=1');
    await waitForBridge(page);
    await page.locator('[data-testid="boot-screen"] kbd').waitFor();
    await page.evaluate(() => (window as any).__dystrailTest.seed(4242));

    const screens = [
      { name: 'boot', selector: '[data-testid="boot-screen"]' },
      { name: 'menu', selector: '[data-testid="menu-screen"]', url: /\/menu/ },
      { name: 'about', selector: '[data-testid="about-screen"]', url: /\/about/ },
      { name: 'settings', selector: '[data-testid="settings-screen"]', url: /\/settings/ },
      { name: 'persona', selector: '[data-testid="persona-screen"]', url: /\/persona/ },
      { name: 'mode-select', selector: '[data-testid="mode-select"]', url: /\/mode/ },
      { name: 'outfitting', selector: '[data-testid="outfitting-screen"]', url: /\/outfitting/ },
      { name: 'travel', selector: '[data-testid="travel-screen"]', url: /\/travel/ },
      { name: 'inventory', selector: '[data-testid="inventory-screen"]', url: /\/inventory/ },
      { name: 'pace-diet', selector: '[data-testid="pace-diet-screen"]', url: /\/pace-diet/ },
      { name: 'map', selector: '[data-testid="map-screen"]', url: /\/map/ },
      {
        name: 'route-prompt',
        selector: '[data-testid="route-prompt-screen"]',
        url: /\/route/,
      },
      { name: 'camp', selector: '[data-testid="camp-screen"]', url: /\/camp/ },
      { name: 'encounter', selector: '[data-testid="encounter-screen"]', url: /\/encounter/ },
      { name: 'crossing', selector: '[data-testid="crossing-screen"]', url: /\/crossing/ },
      { name: 'store', selector: '[data-testid="store-screen"]', url: /\/store/ },
      { name: 'boss', selector: '[data-testid="boss-screen"]', url: /\/boss/ },
      {
        name: 'result-victory',
        selector: '[data-testid="result-screen"]',
        url: /\/result/,
        headline: 'YOU PASSED CLOTURE!',
      },
      {
        name: 'result-pants',
        selector: '[data-testid="result-screen"]',
        url: /\/result/,
        headline: 'NATIONAL PANTS EMERGENCY',
      },
      {
        name: 'result-sanity',
        selector: '[data-testid="result-screen"]',
        url: /\/result/,
        headline: 'SANITY FRACTURE',
      },
      {
        name: 'result-resource',
        selector: '[data-testid="result-screen"]',
        url: /\/result/,
        headline: 'STARVATION COLLAPSE',
      },
      {
        name: 'result-boss-loss',
        selector: '[data-testid="result-screen"]',
        url: /\/result/,
        headline: 'FILIBUSTER CRUSHED YOU',
      },
    ];

    for (const screen of screens) {
      await test.step(`screen: ${screen.name}`, async () => {
        await setScreen(page, screen.name);
        await expect(page.locator(screen.selector)).toBeVisible();
        if (screen.url) {
          await expect(page).toHaveURL(screen.url);
        }
        if (screen.headline) {
          await expect(
            page.locator('[data-testid="result-screen"] .result-headline')
          ).toHaveText(screen.headline);
        }
      });
    }
  });
});
