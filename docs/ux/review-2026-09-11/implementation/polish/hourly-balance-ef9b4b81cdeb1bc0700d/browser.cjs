'use strict';
// Disposable balance checks. Execution requires the root-supplied built URL and revision.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const out = __dirname;
const source = '/tmp/dystrail-final-balance-build/source';
const { chromium } = require('/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright');
const { expect } = require('/Users/vanna/Source/Dystrail/dystrail-web/node_modules/@playwright/test');
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const digest = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const pacingPath = source + '/dystrail-web/static/assets/data/pacing.json';
const campPath = source + '/dystrail-web/static/assets/data/camp.json';
const pacing = read(pacingPath), camp = read(campPath);
const baselines = Object.fromEntries(['classic', 'deep'].map(mode => [mode, read(out + `/fixtures/${mode}.json`)]));
const signed = value => value >= 0 ? `+${value}` : String(value);
const checkpoint = page => page.evaluate(() => JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
const canonical = state => { const copy = structuredClone(state); copy.inventory.tags.sort(); return copy; };
const same = (a, b) => assert.deepEqual(canonical(a), canonical(b));
let report, activePage, target;
const persist = () => fs.writeFileSync(out + '/browser-results.json', JSON.stringify(report, null, 2) + '\n');

async function menu(page) {
  const button = page.locator('#game-menu-button');
  if (await button.getAttribute('aria-expanded') !== 'true') await button.click();
}
async function closeMenu(page) {
  const button = page.locator('#game-menu-button');
  if (await button.getAttribute('aria-expanded') === 'true') await button.click();
}
async function importState(page, state) {
  await menu(page);
  await page.locator('#save-open-btn').click();
  const backup = page.locator('.save-text-backup');
  if (await backup.getAttribute('open') === null) await backup.locator('summary').click();
  await page.locator('#import-json').fill(JSON.stringify(state));
  await page.getByRole('button', { name: 'Import', exact: true }).click();
  await expect(page.locator('.drawer')).toHaveCount(0);
  await closeMenu(page);
}
function fixture(mode, sanity = 8, hp = 8, initialized = true) {
  const state = structuredClone(baselines[mode]);
  state.day = 6;
  state.clock_minutes = 870;
  state.current_encounter = null;
  state.ending = null;
  state.abandoned = false;
  state.ally_notice = null;
  state.breakdown = null;
  state.boss.ready = false;
  state.endgame.active = false;
  state.route_services.stop = null;
  state.route_services.map_reviewed = null;
  state.route_services.trading = false;
  state.activities.local_word = null;
  state.day_state = { ...state.day_state, day_initialized: initialized, did_end_of_day: false,
    traveled_today: false, partial_traveled_today: false, travel_blocked: false,
    rest_requested: false, suppress_stop_ratio: false };
  state.stats = { ...state.stats, supplies: 12, hp, sanity, morale: 8, allies: 3 };
  state.camp = { rest_cooldown: 0, repair_cooldown: 0 };
  state.diet = 'mixed';
  state.pace = 'steady';
  state.weather_state.today = 'Clear';
  state.weather_state.yesterday = 'Clear';
  state.weather_state.neutral_buffer = 100;
  state.weather_travel_multiplier = 1;
  state.current_order = null;
  state.exec_order_days_remaining = 0;
  state.exec_order_cooldown = 100;
  state.exec_travel_multiplier = 1;
  state.exec_breakdown_bonus = 0;
  state.illness_days_remaining = 0;
  state.illness_travel_penalty = 1;
  state.disease_cooldown = 100;
  state.vehicle.breakdown_cooldown = 100;
  state.crew_care = { reason: 0, strain: {}, pending: null, last_check_day: 100 };
  state.party.members.forEach(member => { member.status = 'Active'; });
  state.daily_remainders = { supplies: 0, sanity: 0, health: 0 };
  state.driving_minutes_total = 0;
  state.pace_fatigue_remainder = 0;
  state.prev_miles_traveled = state.miles_traveled_actual;
  state.distance_remainder = 0;
  state.day_start_remainder = 0;
  state.current_day_kind = null;
  state.current_day_miles = 0;
  state.current_day_reason_tags = [];
  state.current_day_record = { day_index: state.day - 1, kind: 'non_travel', miles: 0, tags: [] };
  state.journal = [];
  state.turn_journal_start = 0;
  return state;
}
async function screenshot(page, name) {
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'Horizontal overflow');
  const file = out + `/${name}.png`;
  await page.screenshot({ path: file, fullPage: true, animations: 'disabled' });
  report.screenshots.push(file);
  return file;
}
async function freshPage(browser, width) {
  const context = await browser.newContext({ viewport: { width, height: width === 393 ? 852 : 1000 }, reducedMotion: 'reduce' });
  const page = await context.newPage();
  activePage = page;
  page.setDefaultTimeout(15000);
  page.on('pageerror', error => report.errors.push({ case: report.currentCase, type: 'pageerror', message: error.message }));
  page.on('console', message => { if (message.type() === 'error') report.errors.push({ case: report.currentCase, type: 'console', message: message.text(), location: message.location() }); });
  page.on('requestfailed', request => report.failedRequests.push({ case: report.currentCase, url: request.url(), failure: request.failure() }));
  await page.goto(target.url);
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'setup', { timeout: 90000 });
  await page.waitForFunction(() => window.dystrailOffline?.state === 'ready');
  assert.equal(await page.evaluate(() => window.dystrailOffline.revision), target.revision, 'Active offline revision');
  return { context, page };
}
async function verifyAssets(page) {
  for (const [name, expected] of [['pacing', pacing], ['camp', camp]]) {
    const response = await page.request.get(new URL(`static/assets/data/${name}.json`, target.url).href);
    assert.equal(response.ok(), true, `${name} asset status`);
    assert.match(response.headers()['content-type'], /application\/json/);
    assert.deepEqual(await response.json(), expected, `${name} asset differs from frozen candidate`);
  }
}
async function conditions(page, mode, width) {
  await importState(page, fixture(mode, 8, 8, false));
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'travel');
  const before = await checkpoint(page);
  await page.getByRole('tab', { name: 'Conditions', exact: true }).click();
  const fields = page.locator('.daily-settings fieldset');
  await expect(fields).toHaveCount(2);
  const paceButtons = fields.nth(0).locator('button.setting-option');
  const dietButtons = fields.nth(1).locator('button.setting-option');
  await expect(paceButtons.locator('.setting-title')).toHaveText(['Steady', 'Heated', 'Blitz']);
  await expect(dietButtons.locator('.setting-title')).toHaveText(['Quiet', 'Mixed', 'Doomscroll']);
  for (const [index, speed] of [60, 70, 80].entries()) {
    await expect(paceButtons.nth(index).locator(':scope > span')).toHaveText(`Up to ${speed} MPH`);
    await expect(paceButtons.nth(index)).toBeEnabled();
  }
  for (const [index, delta] of [3, 2, -2].entries()) {
    await expect(dietButtons.nth(index).locator(':scope > span')).toHaveText(`Sanity ${signed(delta)} per day`);
    await expect(dietButtons.nth(index)).toBeEnabled();
  }
  const selections = [];
  for (const [index, [paceId, dietId]] of [['steady', 'quiet'], ['heated', 'mixed'], ['blitz', 'doom']].entries()) {
    await paceButtons.nth(index).click();
    await dietButtons.nth(index).click();
    await expect(paceButtons.nth(index)).toHaveAttribute('aria-pressed', 'true');
    await expect(dietButtons.nth(index)).toHaveAttribute('aria-pressed', 'true');
    const saved = await checkpoint(page);
    assert.equal(saved.pace, paceId);
    assert.equal(saved.diet, dietId);
    assert.deepEqual(saved.stats, before.stats);
    assert.equal(saved.day, before.day);
    assert.equal(saved.clock_minutes, before.clock_minutes);
    selections.push({ pace: saved.pace, diet: saved.diet });
  }
  const image = await screenshot(page, `conditions-${mode}-${width}`);
  return { dietSanity: [3, 2, -2], mph: [60, 70, 80], selections, settingsChangesCostNothing: true, screenshot: image };
}
async function rest(page, mode, width, scenario) {
  await importState(page, fixture(mode, scenario.sanity, scenario.hp, true));
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'travel');
  const before = await checkpoint(page);
  assert.equal(before.day_state.day_initialized, true, 'Rest fixture begins after daily effects were settled');
  await page.getByRole('button', { name: 'Camp', exact: true }).click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'camp');
  same(await checkpoint(page), before);
  const button = page.locator('.camp-modal').getByRole('button', { name: 'Rest', exact: true });
  const expectedDetail = `Sanity +${scenario.sanityGain}, Health +${scenario.hpGain}, Supplies -1 · 1 day`;
  await expect(button).toBeEnabled();
  await expect(button.locator('.action-detail')).toHaveText(expectedDetail);
  const image = await screenshot(page, `rest-${mode}-${width}-sanity-${scenario.sanity}`);
  await button.click();
  await expect(page.locator('.aftermath-panel')).toBeVisible();
  const after = await checkpoint(page);
  assert.equal(after.stats.sanity, before.stats.sanity + scenario.sanityGain, 'Rest grants the displayed capped sanity recovery');
  assert.equal(after.stats.hp, before.stats.hp + scenario.hpGain, 'Rest grants the displayed capped health recovery');
  assert.equal(after.stats.supplies, before.stats.supplies - 1, 'Rest consumes exactly the displayed supply');
  assert.equal(after.day, before.day + 1, 'Rest advances one day');
  assert.equal(after.clock_minutes, 480, 'Rest resumes at 08:00');
  assert.equal(after.miles_traveled_actual, before.miles_traveled_actual, 'Rest never advances miles');
  assert.equal(after.driving_minutes_total, before.driving_minutes_total, 'Rest never accrues driving time');
  assert.equal(after.camp.rest_cooldown, 2);
  assert.equal(after.journal.length, 1);
  assert.deepEqual(after.journal[0].after, after.stats);
  await page.reload();
  const reloaded = await checkpoint(page);
  assert.deepEqual(reloaded.stats, after.stats);
  assert.deepEqual(reloaded.journal, after.journal);
  return { beforeSanity: scenario.sanity, preview: expectedDetail, afterSanity: after.stats.sanity, supplyDelta: -1, dayDelta: 1, stationary: true, persisted: true, screenshot: image };
}
async function blockedDuringBreakdown(page, mode, width) {
  const state = fixture(mode, 2, 8, true);
  state.breakdown = { part: 'Battery', day_started: state.day };
  state.vehicle.health = 0;
  state.inventory.spares.battery = 1;
  state.budget_cents = 10000;
  await importState(page, state);
  const before = await checkpoint(page);
  const button = page.getByRole('button', { name: 'Camp', exact: true });
  await expect(button).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Rest', exact: true })).toHaveCount(0);
  await expect(page.locator('.camp-modal')).toHaveCount(0);
  same(await checkpoint(page), before);
  // Test the existing saved-phase guard using disposable storage only.
  await page.evaluate(() => {
    const saved = JSON.parse(localStorage.getItem('dystrail.autosave.v1'));
    saved.phase = 'Camp';
    localStorage.setItem('dystrail.autosave.v1', JSON.stringify(saved));
  });
  await page.reload();
  await expect(button).toBeDisabled();
  await expect(page.locator('.camp-modal')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Rest', exact: true })).toHaveCount(0);
  same(await checkpoint(page), before);
  const image = await screenshot(page, `breakdown-blocks-rest-${mode}-${width}`);
  return { disabled: true, forgedSavedCampPhaseBlocked: true, savedStateUnchanged: true, screenshot: image };
}

async function run() {
  const targetPath = process.argv[2];
  assert(targetPath, 'Await the root-supplied target JSON containing url and revision; no target supplied.');
  target = read(path.resolve(targetPath));
  assert.equal(typeof target.url, 'string');
  assert.equal(typeof target.revision, 'string');
  assert(target.revision.length > 0);
  const url = new URL(target.url);
  assert(['http:', 'https:'].includes(url.protocol));
  if (!url.pathname.endsWith('/')) url.pathname += '/';
  target.url = url.href;
  assert.deepEqual(pacing.diet.map(diet => [diet.id, diet.sanity]), [['quiet', 3], ['mixed', 2], ['doom', -2]]);
  assert.deepEqual(pacing.pace.map(pace => pace.speed_mph), [60, 70, 80]);
  assert.deepEqual(camp.rest, { sanity: 10, hp: 1, supplies: -1, day: 1, cooldown_days: 2, recovery_day: true });
  report = { url: target.url, revision: target.revision, sourceHashes: { pacing: digest(pacingPath), camp: digest(campPath) }, cases: [], screenshots: [], errors: [], failedRequests: [] };
  persist();
  const browser = await chromium.launch({ headless: true, executablePath: '/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py' });
  try {
    for (const mode of (target.modes ?? ['classic', 'deep'])) for (const width of (target.widths ?? [1440, 393])) {
      report.currentCase = { mode, width }; persist();
      const { context, page } = await freshPage(browser, width);
      try {
        await verifyAssets(page);
        const settings = await conditions(page, mode, width);
        const restResults = [];
        for (const scenario of [
          { sanity: 2, hp: 8, sanityGain: 8, hpGain: 1 },
          { sanity: 9, hp: 9, sanityGain: 1, hpGain: 1 },
          { sanity: 10, hp: 10, sanityGain: 0, hpGain: 0 },
        ]) restResults.push(await rest(page, mode, width, scenario));
        const breakdown = await blockedDuringBreakdown(page, mode, width);
        report.cases.push({ mode, width, conditions: settings, rest: restResults, breakdown });
        persist();
      } catch (error) {
        await page.screenshot({ path: out + '/failure.png', fullPage: true, animations: 'disabled' }).catch(() => {});
        fs.writeFileSync(out + '/failure-body.txt', await page.locator('body').innerText().catch(() => ''));
        fs.writeFileSync(out + '/failure-state.json', JSON.stringify(await checkpoint(page).catch(() => null), null, 2));
        throw error;
      } finally { await context.close(); }
    }
    assert.deepEqual(report.errors, []);
    delete report.currentCase;
    report.passed = true;
    persist();
    console.log(JSON.stringify({ revision: target.revision, cases: report.cases.length, restActions: report.cases.reduce((n, c) => n + c.rest.length, 0), errors: report.errors, screenshots: report.screenshots }));
  } catch (error) {
    if (activePage && !activePage.isClosed()) {
      await activePage.screenshot({ path: out + '/failure.png', fullPage: true, animations: 'disabled' }).catch(() => {});
      fs.writeFileSync(out + '/failure-body.txt', await activePage.locator('body').innerText().catch(() => ''));
      fs.writeFileSync(out + '/failure-state.json', JSON.stringify(await checkpoint(activePage).catch(() => null), null, 2));
    }
    report.failure = error.stack;
    persist();
    throw error;
  } finally { await browser.close(); }
}
run().catch(error => { console.error(error); process.exitCode = 1; });
