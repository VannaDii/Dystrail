import { readFileSync } from 'node:fs';
import { test, expect } from '@playwright/test';
import { baseline, importState, openMenu, snap } from './helpers';

const stories = JSON.parse(readFileSync('static/assets/data/game.json', 'utf8'));
const examples = ['deep_circuit_breaker', 'deep_secure_line', 'sat_weather_desk'];

for (const [language, option] of [['it', 'Italiano'], ['es', 'Español'], ['ar', 'العربية']]) {
  const copy = JSON.parse(readFileSync(`i18n/${language}.json`, 'utf8')).encounter_copy;
  for (const id of examples) {
    test(`writer localization ${language}: ${id} remains readable offline`, async ({ page, context }) => {
      const gs = await baseline(page);
      const story = stories.find((entry: { id: string }) => entry.id === id);
      gs.current_encounter = story;
      gs.day_state.day_initialized = true;
      gs.budget_cents = 5000;
      gs.journal = [];
      gs.turn_journal_start = 0;
      await importState(page, gs);
      await context.setOffline(true);
      await page.reload();
      await openMenu(page);
      await page.getByRole('button', { name: 'Language', exact: true }).click();
      await page.getByRole('option', { name: option, exact: true }).click();
      await page.locator('#game-menu-button').click();

      await expect(page.locator('html')).toHaveAttribute('lang', language);
      await expect(page.locator('html')).toHaveAttribute('dir', language === 'ar' ? 'rtl' : 'ltr');
      await expect(page.locator('.journey-scene h1')).toContainText(copy[id].name);
      await expect(page.locator('.encounter-desc p')).toHaveText(copy[id].desc);
      const buttons = page.locator('.encounter-choice button');
      await expect(buttons).toHaveCount(story.choices.length);
      for (let choice = 0; choice < story.choices.length; choice++) {
        await expect(buttons.nth(choice).locator('.action-title')).toHaveText(copy[id][`choice_${choice}`]);
      }
      if (id === 'deep_secure_line') await snap(page, `writer-${language}`);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      await buttons.first().click();
      await expect(page.locator('.outcome-summary')).toContainText(copy[id].log_0);
      await page.reload();
      await expect(page.locator('html')).toHaveAttribute('lang', language);
      await context.setOffline(false);
    });
  }
}
