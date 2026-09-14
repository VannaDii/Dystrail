import { test, expect } from '@playwright/test';
import { baseline, importState, snap, waitForLaunch } from './helpers';

// Final authored workshop snapshot, captured 2026-09-13. These pinned strings
// do not come from the game bundle: an omitted import must fail here.
// Document: 1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k
// Revision: ANLCKQnwMWWXuS4ELr1I0ohWwa1b-7h9hCUDSdGapEtxM96pfap2t9ljAaL7YlDbeOUqHkCw-mxgQk7SVY7QXTBMrGZhoF7TGhpoQofaZjM
// Copy-only release: A versions; original ordered mechanics retained.
const stories = [
  {
    "id": "sat_alternator_tariff",
    "title": "China’s card declined",
    "desc": "Trump says China pays the tariffs. At a roadside shop, your supplies cost $12. The cashier taps a Chinese flag against the card reader. Declined.",
    "choices": [
      {
        "label": "Pay the $12",
        "log": "Your payment works. “Congratulations,” she says. “You’re China now.”"
      },
      {
        "label": "Get the itemized quote",
        "log": "She circles the tariff. “China’s very generous with your money.”"
      },
      {
        "label": "Keep the $12",
        "log": "You leave empty-handed. The tariff has successfully protected the supplies from Americans."
      }
    ],
    "cashChoice": 0,
    "cashDelta": -1200
  },
  {
    "id": "sat_billion_pothole",
    "title": "A donation you can trip over",
    "desc": "Super PACs accept unlimited contributions. You stop at an event depot offering $10 to carry a donor’s giant ceremonial cheque. Its accompanying poster says every citizen has a voice. The cheque is too wide to fit through the citizens’ entrance.",
    "choices": [
      {
        "label": "Save the terms and donor display",
        "log": "You document the arrangement, head aching. The donor’s speech is officially independent; it still requires three people and a loading door."
      },
      {
        "label": "Carry the cheque for $10",
        "log": "Ten dollars and sore muscles. The donor signs in one effortless stroke; you need to sit down after delivering his free speech."
      },
      {
        "label": "Pass on the job",
        "log": "You leave disappointed. A worker removes the public entrance sign so the donor’s equal voice can scrape through sideways."
      }
    ],
    "cashChoice": 1,
    "cashDelta": 1000
  },
  {
    "id": "sat_cabinet_guest",
    "title": "The sea forgot its new name",
    "desc": "Trump ordered federal maps to say Gulf of America. At a marine-supply counter, a customer demands an American refund because his old chart still says Mexico. The clerk has the order. The customer checks his shoes for foreign seawater.",
    "choices": [
      {
        "label": "Read and save the order",
        "log": "You save the actual naming change, head aching. The clerk explains that the water did not relocate; the customer looks personally betrayed by geography."
      },
      {
        "label": "Perform the ocean’s citizenship interview",
        "log": "The crew laughs until you invent a maritime border change. The clerk takes the chart away before your geography reaches the fish."
      },
      {
        "label": "Leave the clerk to work",
        "log": "You leave irritated. The customer wrings out a sock and asks whether the shop can exchange the water for the patriotic version."
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
    await page.reload();await waitForLaunch(page);
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
