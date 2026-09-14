import { readFileSync } from 'node:fs';
import { test, expect } from '@playwright/test';
import { baseline, importState, snap, waitForLaunch } from './helpers';

type Story = {
  id: string;
  name: string;
  desc: string;
  choices: { label: string; effects: { log?: string } }[];
};

const stories: Story[] = JSON.parse(readFileSync('static/assets/data/game.json', 'utf8'));
const pictured = new Set(['deep_circuit_breaker', 'deep_secure_line', 'sat_weather_desk']);

for (const story of stories) {
  test(`complete writer catalog: ${story.id}`, async ({ page, context }) => {
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    const gs = await baseline(page);
    gs.current_encounter = story;
    gs.day_state.day_initialized = true;
    gs.budget_cents = 5000;
    gs.stats.morale = 8;
    gs.stats.allies = 5;
    gs.journal = [];
    gs.turn_journal_start = 0;
    await importState(page, gs);
    await context.setOffline(true);
    await page.reload();
    await waitForLaunch(page);

    for (let choice = 0; choice < story.choices.length; choice++) {
      if (choice > 0) await importState(page, gs);
      await expect(page.locator('.journey-scene h1')).toContainText(story.name);
      await expect(page.locator('.encounter-desc p')).toHaveText(story.desc);
      const buttons = page.locator('.encounter-choice button');
      await expect(buttons).toHaveCount(story.choices.length);
      for (let index = 0; index < story.choices.length; index++) {
        await expect(buttons.nth(index).locator('.action-title')).toHaveText(story.choices[index].label);
      }
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      if (choice === 0 && pictured.has(story.id)) await snap(page, `writer-catalog-${story.id}`);
      await buttons.nth(choice).click();
      await expect(page.locator('.aftermath-panel')).toBeVisible();
      await expect(page.locator('.outcome-summary')).toContainText(story.choices[choice].effects.log ?? story.choices[choice].label);
    }

    expect(errors).toEqual([]);
    await context.setOffline(false);
  });
}
