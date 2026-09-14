import { chromium } from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import assert from 'node:assert/strict';

const root = '/tmp/dystrail-language-popover-20260913/release-prep';
const base = process.argv[2] || 'https://dystrail.com';
assert(['https://dystrail.com', 'http://127.0.0.1:8180'].includes(base));
const { revision } = JSON.parse(await readFile(root + '/staged-release-record.json', 'utf8'));
const output = root + (base.startsWith('https:') ? '/live-browser' : '/combined-preview-browser');
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, executablePath: '/Users/vanna/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell' });
const report = { revision, base, cases: [], passed: false };
try {
  for (const [name, viewport, deep] of [['desktop', { width: 1440, height: 900 }, false], ['phone', { width: 393, height: 852 }, true]]) {
    const context = await browser.newContext({ viewport, serviceWorkers: 'allow' });
    try {
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.goto(base + '/play?review=entry%20route#arrival');
      await page.waitForFunction(expected => window.dystrailOffline?.revision === expected && window.dystrailOffline.state === 'ready' && !document.querySelector('#launch-gate') && document.querySelector('#main'), revision, { timeout: 120000 });
      assert.equal(page.url(), base + '/play/?review=entry%20route#arrival');
      assert.equal(await page.evaluate(() => navigator.serviceWorker.controller !== null), true);
      if (deep) await page.getByRole('radio', { name: /The Deep End/ }).check();
      await page.locator('#game-menu-button').click();
      const chevron = await page.locator('.language-picker>button').evaluate(element => {
        const button = element.getBoundingClientRect();
        const arrow = element.querySelector('span').getBoundingClientRect();
        const style = getComputedStyle(element);
        return { gap: button.right - arrow.right, expected: parseFloat(style.paddingRight) + parseFloat(style.borderRightWidth) };
      });
      assert(Math.abs(chevron.gap - chevron.expected) < 1);
      const geometry = () => page.locator('#game-menu-panel').evaluate(element => ({ height: element.getBoundingClientRect().height, content: element.scrollHeight, scroll: element.scrollTop }));
      const before = await geometry();
      await page.locator('.language-picker>button').click();
      await page.getByRole('listbox').waitFor();
      assert.deepEqual(await geometry(), before);
      const list = await page.getByRole('listbox').evaluate(element => {
        const bounds = element.getBoundingClientRect();
        const menu = document.querySelector('#game-menu-panel').getBoundingClientRect();
        return { topLayer: element.matches(':popover-open'), outside: bounds.bottom > menu.bottom || bounds.top < menu.top, fits: bounds.left >= 11 && bounds.top >= 11 && bounds.right <= innerWidth - 11 && bounds.bottom <= innerHeight - 11 };
      });
      assert.deepEqual(list, { topLayer: true, outside: true, fits: true });
      assert.equal(await page.getByRole('option').count(), 20);
      await page.keyboard.press('End');
      assert.equal(await page.getByRole('option', { name: 'Türkçe', exact: true }).evaluate(element => element === document.activeElement), true);
      assert.deepEqual(await geometry(), before);
      await page.screenshot({ path: output + '/' + name + '.png' });
      assert.deepEqual(errors, []);
      report.cases.push({ name, viewport, mode: deep ? 'deep' : 'classic', url: page.url(), controlled: true, chevron, list, menuUnchanged: true, languageCount: 20, lastLanguageReachable: true, errors, passed: true });
    } finally { await context.close(); }
  }
  report.passed = true;
} catch (error) {
  report.failure = String(error.stack || error);
  process.exitCode = 1;
} finally {
  await browser.close();
  await writeFile(output + '/verification.json', JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
}
