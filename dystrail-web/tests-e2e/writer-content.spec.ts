import { test, expect } from '@playwright/test';
import { baseline, importState, snap } from './helpers';

// Final authored workshop snapshot, captured 2026-09-13. These pinned strings
// do not come from the game bundle: an omitted import must fail here.
// Document: 1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k
// Revision: ANLCKQlwLW9DDjI14b_JMfeMuESUHjLGbK1zsEZVWGoch3_GsjRXeM6p7zsKj1ACb1Cynu-zWe_jZFR2DFmKPsED1cBkZJQDYbhXIVKGa1o
// Handoff SHA-256: b3c9cc5db50c6f6dfbe402527c5bf14cab129315550f6fba0a4ad195db3b425b
const stories = [
  {
    "id": "sat_alternator_tariff",
    "title": "The Patriotic Parts Counter",
    "desc": "The TV says China pays the tariffs. The cashier turns the $12 card reader toward you.",
    "choices": [
      {
        "label": "Buy the kit — $12",
        "log": "You buy the kit. China owes you a beer."
      },
      {
        "label": "Get a written quote",
        "log": "She prints it. “For your reimbursement from China.”"
      },
      {
        "label": "Leave with the $12",
        "log": "The kit stays on the shelf, awaiting payment from Beijing."
      }
    ],
    "cashChoice": 0,
    "cashDelta": -1200
  },
  {
    "id": "sat_billion_pothole",
    "title": "The Eight-Billion-Dollar Pothole",
    "desc": "DOGE listed an $8 million contract as $8 billion. A diner owner offers $10 to haul pothole patch. “I checked my math.”",
    "choices": [
      {
        "label": "Save the claim and correction",
        "log": "You save both. A cashier has to explain a missing quarter. DOGE gets three spare zeroes."
      },
      {
        "label": "Haul the bags — earn $10",
        "log": "You haul the bags and pocket $10. Your back would prefer DOGE’s math."
      },
      {
        "label": "Drive around the damn hole",
        "log": "Finally, a problem you can solve by moving further left."
      }
    ],
    "cashChoice": 1,
    "cashDelta": 1000
  },
  {
    "id": "sat_cabinet_guest",
    "title": "Manager Approval Required",
    "desc": "A server on break scrolls through the cabinet’s leaked airstrike chat. “They invited a journalist. I need a manager’s PIN to refund ranch.”",
    "choices": [
      {
        "label": "Read the actual report",
        "log": "You save it. Apparently you’re cleared for airstrikes, just not dipping sauce."
      },
      {
        "label": "Retell it from memory—with impressions",
        "log": "The crew laughs. You invent a detail, then defend it. Now you’re doing the whole administration."
      },
      {
        "label": "Give her back her break",
        "log": "Two minutes for fries, then back to guarding the strategic ranch reserve."
      }
    ],
    "cashChoice": -1,
    "cashDelta": 0
  }
];

for (const story of stories) {
  test(`writer scene ${story.id} retains its complete copy and exact choice effects offline`, async ({ page, context }) => {
    const gs = await baseline(page);
    const response = await page.request.get('/play/static/assets/data/game.json');
    expect(response.ok()).toBe(true);
    expect(response.headers()['content-type']).toContain('application/json');
    const events = await response.json();
    const event = events.find((e: { id: string }) => e.id === story.id);
    expect(event).toBeTruthy();
    expect(event.name).toBe(story.title);
    expect(event.desc).toBe(story.desc);
    expect(event.choices.map((choice: { label: string; effects: { log: string } }) => ({
      label: choice.label, log: choice.effects.log,
    }))).toEqual(story.choices);
    gs.current_encounter = event;
    gs.day_state.day_initialized = true;
    gs.budget_cents = 5000;
    gs.stats.morale = 5;
    gs.stats.credibility = 7;
    await importState(page, gs);
    await context.setOffline(true);
    await page.reload();
    for (let choice = 0; choice < 3; choice++) {
      if (choice > 0) await importState(page, gs);
      await expect(page.locator('.journey-scene h1')).toContainText(story.title);
      const description = page.locator('.encounter-desc p');
      await expect(description).toHaveText(story.desc);
      await expect(description).toHaveCSS('white-space', 'pre-line');
      const buttons = page.locator('.encounter-choice button');
      await expect(buttons).toHaveCount(3);
      for (let index = 0; index < 3; index++) {
        await expect(buttons.nth(index).locator('.action-title')).toHaveText(story.choices[index].label);
      }
      if (choice === 0) await snap(page, `writer-${story.id}`);
      await buttons.nth(choice).click();
      await expect(page.locator('.aftermath-panel')).toBeVisible();
      await expect(page.locator('.outcome-summary')).toContainText(story.choices[choice].log);
      if (choice === story.cashChoice) {
        const after = await page.evaluate(() => JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
        expect(after.budget_cents).toBe(gs.budget_cents + story.cashDelta);
      }
      if (story.id === 'sat_cabinet_guest' && choice === 1) {
        const after = await page.evaluate(() => JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
        expect(after.stats.morale).toBe(7);
        expect(after.stats.credibility).toBe(5);
      }
    }
    await context.setOffline(false);
  });
}
