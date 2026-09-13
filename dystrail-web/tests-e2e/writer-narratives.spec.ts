import { readFileSync } from 'node:fs';
import { test, expect, type Page } from '@playwright/test';
import { baseline, importState } from './helpers';
import { atTown, routes } from './geography';

// Final authored workshop snapshot, captured 2026-09-13. These pinned strings
// do not come from the game bundle: an omitted import must fail here.
// Document: 1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k
// Revision: ANLCKQlwLW9DDjI14b_JMfeMuESUHjLGbK1zsEZVWGoch3_GsjRXeM6p7zsKj1ACb1Cynu-zWe_jZFR2DFmKPsED1cBkZJQDYbhXIVKGa1o
// Handoff SHA-256: b3c9cc5db50c6f6dfbe402527c5bf14cab129315550f6fba0a4ad195db3b425b
const reviewed: {
  town_comments: Record<string, string>;
  locale_copy: Record<string, string>;
} = {
  "town_comments": {
    "Minneapolis": "I miss a spot cleaning the lab, I get written up. They delete its funding and get a podium.",
    "La Crosse": "Close the river lab? Great. We’ll ask the fish to rate the water from one to five stars.",
    "Madison": "The lab got temporary ‘bridge funding.’ Good. I was worried we’d have to start researching under one.",
    "Chicago": "They’ve paused the train extension. My boss says I should’ve left home before the funding cut.",
    "South Bend": "Hiring’s frozen. I’m covering three jobs. Funny how the workload survived the cold snap.",
    "Toledo": "Tariffs made our paper cost more. We’re printing the ‘economic victory’ flyers double-sided to afford it.",
    "Cleveland": "They cut college counseling. The American Dream now requires a parent who knows a guy.",
    "Pittsburgh": "They cut our research grants. Maybe if we discovered a new tax loophole, Congress would fund the lab.",
    "Cumberland": "Five park staff cut. The trash is now a self-guided exhibit.",
    "Hagerstown": "Privatize the mail? Can’t wait for Mom’s birthday card to arrive after this unskippable ad.",
    "Frederick": "Ninety HHS layoffs on the notice. My landlord says government efficiency is not an accepted payment method.",
    "D.C.": "Trump takes over the Kennedy Center. Hamilton cancels; I lose concession shifts. Finally, a president brave enough to confront Big Pretzel.",
    "Kansas City": "USDA canceled our food shipment. Tonight’s special: lightly sautéed notification email.",
    "Columbia": "The lake scientist needs HOA funding. First, the algae must be an approved shade of green.",
    "St. Louis": "Washington cut the program buying farmers’ food for hungry neighbors. Apparently the middleman wasn’t rich enough.",
    "Springfield, IL": "The shutdown closed Lincoln’s house. Of the people, by the people, back whenever the fuck we feel like it.",
    "Denver": "Washington stiffed us for the shelter bill. When I try that with rent, it’s suddenly a character flaw.",
    "North Platte": "They cut weather-balloon launches. Now we’re getting the forecast from my bad knee. It wants benefits.",
    "Omaha": "They want to cut the lab’s operating money. Great. We’ll keep the samples cold with fiscal discipline.",
    "Des Moines": "They cut money for local school food. We’re calling the carrots tactical equipment and trying again.",
    "Iowa City": "Cutting the writers’ program cut my housekeeping shifts. The culture war could at least leave a tip.",
    "Austin": "Veterans’ book group paused. ‘Thank you for your service’ is now the entire syllabus.",
    "Waco": "SNAP stopped. The register asks if I’d like to donate to fight hunger. I’d like to receive, actually.",
    "Dallas": "They want to cut the money that keeps our lab open. Apparently the cancer cells can work from home.",
    "Oklahoma City": "Full-time job, food-bank queue. If hard work made you rich, this line would need valet parking.",
    "Tulsa": "The health department’s funding is uncertain. Germs, meanwhile, have renewed their contract.",
    "Joplin": "USDA left the food bank three million short. I donated beans. My name had better be on the fucking building.",
    "Springfield, MO": "USDA canceled three million dollars in food. Washington’s contribution to the potluck is an empty tray labeled SAVINGS."
  },
  "locale_copy": {
    "trail.care_reason_0": "{name} feels sick after a MAHA raw-milk sample. The vendor blames the deep state inside their stomach.",
    "trail.care_reason_1": "{name} is dehydrated. The unloading boss crossed out the heat warning with a Sharpie. Apparently the sun doesn’t take corrections.",
    "trail.care_reason_2": "{name} stayed up listing five accomplishments for a DOGE-inspired boss. Exhausted, they submit a sixth: unpaid overtime.",
    "trail.care_reason_3": "{name} vomits after a MAHA wellness sample. The vendor cites ‘unpublished research.’ They publish a rebuttal into his sample bucket.",
    "trail.care_reason_4": "{name} lost sleep navigating a road with three patriotic names. None of them was the exit you needed.",
    "trail.care_reason_5": "{name} spent the night disputing a repair tariff. They’re exhausted. The trade war has secured a decisive victory over bedtime.",
    "trail.care_reason_6": "{name} is exhausted from a temp-job Signal chat. The boss added strangers, leaked the door code, and took inspiration from the cabinet.",
    "trail.care_reason_7": "{name} strains their back hauling donated food after USDA delivery cuts. Washington has replaced a delivery budget with their spine.",
    "ally_loss.reason_0": "{name} quits for MAGA. Their goodbye text includes a discount code for red hats. Even the betrayal has a checkout page.",
    "ally_loss.reason_1": "{name} is laid off in a DOGE-style cut. They leave to job-hunt. Efficiency has created forty hours of unpaid work.",
    "ally_loss.reason_2": "A repair tariff forces {name} to quit helping and take evening shifts. “They said China would pay. China can cover my fucking shift.”",
    "ally_loss.reason_3": "{name} quits to sell Trump merch. They promise to keep fighting fascism after they’ve cleared the inventory.",
    "ally_loss.reason_4": "{name} sends a Truth Social rant, says “do your own research,” then blocks you. Peer review completed.",
    "ally_loss.reason_5": "{name} sprains their ankle protesting and heads home. “Tell Washington I’ve filed a motion to sit the fuck down.”",
    "journey.care_body": "The medic offers {name} treatment before asking about payment. You briefly suspect an elaborate sting.",
    "journey.care_critical": "{name} is critically ill. Continuing without care will kill them. Provide care or take a full recovery day.",
    "journey.care_action": "Treat them · Supplies −2 · Morale +{morale}",
    "journey.shelter_action": "Leave a companion with the medics · Crew member leaves · Morale −{morale}",
    "journey.defer_action": "Keep driving without care · Sanity −{sanity} · Illness may worsen",
    "journey.fatal_action": "Continue without care · Fatal · Sanity −{sanity} · Morale −{morale}",
    "journey.care_helped": "{name} is fit to travel. Two supplies: the least insulting medical bill in America.",
    "journey.care_sheltered": "{name} stays for care. The health system works; the carpool doesn’t.",
    "journey.care_deferred": "{name} is still sick. You’ve recreated your old job’s sick-leave policy: keep going.",
    "journey.care_lost": "{name} dies. The crew continues without them.",
    "journey.player_lost": "{name} dies. The journey ends.",
    "journey.crew_stop": "Someone needs care",
    "journey.care_hint": "Each choice takes one hour. Care costs two supplies. Only companions can leave with the medics. A full recovery day clears illness."
  }
};

type TownFact = {
  town: string;
  text: Record<string, string>;
  comment: Record<string, string>;
  source: string;
  checked: string;
};
const townFacts: TownFact[] = JSON.parse(readFileSync('static/assets/data/town-facts.json', 'utf8'));
const english = JSON.parse(readFileSync('i18n/en.json', 'utf8'));
const stories = JSON.parse(readFileSync('static/assets/data/game.json', 'utf8'));

function copy(key: string, values: Record<string, string | number> = {}) {
  const template = reviewed.locale_copy[key];
  expect(template, `Reviewed writer key ${key}`).toBeDefined();
  return template.replace(/\{([^}]+)\}/g, (placeholder, name) => {
    expect(values[name], `Value for ${key}: ${placeholder}`).toBeDefined();
    return String(values[name]);
  });
}

async function checkpoint(page: Page) {
  return page.evaluate(() => JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
}

function readyState(original: any) {
  const state = structuredClone(original);
  state.day = 6;
  state.clock_minutes = 480;
  state.day_state.day_initialized = true;
  state.current_encounter = null;
  state.ending = null;
  state.abandoned = false;
  state.ally_notice = null;
  state.breakdown = null;
  state.boss.ready = false;
  state.crew_care = { reason: 0, strain: {}, pending: null, last_check_day: 6 };
  state.activities.local_word = null;
  state.route_services.stop = null;
  state.route_services.map_reviewed = null;
  state.stats = { ...state.stats, supplies: 10, hp: 10, sanity: 8, morale: 8, allies: 3 };
  state.journal = [];
  state.turn_journal_start = 0;
  return state;
}

function crewState(original: any, reason: number, strain: number, player = false) {
  const state = readyState(original);
  const member = state.party.members.find((candidate: any) =>
    player ? candidate.persona === state.persona_id : candidate.persona === 'organizer');
  expect(member, 'The affected person exists in the saved crew').toBeDefined();
  member.name = player ? 'Reese Navarro' : 'Mika Chen';
  member.status = 'Active';
  state.crew_care = {
    reason, strain: { [member.persona]: strain }, pending: member.persona, last_check_day: state.day,
  };
  state.scene_subject = member.persona;
  return { state, persona: member.persona as string, name: member.name as string };
}

async function expectNarrative(page: Page, reason: number, name: string, critical = false) {
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'crew-care');
  await expect(page.locator('.journey-scene h1')).toHaveText(`${name} · ${copy('journey.crew_stop')}`);
  const expected = `${copy(`trail.care_reason_${reason}`, { name })} ${copy(
    critical ? 'journey.care_critical' : 'journey.care_body', { name })}`;
  // Compare only the visible prose text nodes, excluding the adjacent source-help button.
  await expect.poll(() => page.locator('.crew-incident > .scene-narrative').evaluate(element =>
    Array.from(element.childNodes)
      .filter(node => node.nodeType === Node.TEXT_NODE)
      .map(node => node.textContent).join('').replace(/\s+/g, ' ').trim()
  )).toBe(expected);
  await expect(page.locator('.crew-incident')).not.toContainText('These choices take one hour.');
  await expect(page.locator('.crew-incident')).not.toContainText('Each choice takes one hour.');
}

async function expectChoice(page: Page, key: string, values: Record<string, number> = {}) {
  const [title, ...details] = copy(key, values).split(' · ');
  const button = page.locator('.crew-incident .camp-actions').getByRole('button', { name: title, exact: true });
  await expect(button.locator('.action-title')).toHaveText(title);
  await expect(button.locator('.action-detail')).toHaveText([...details, '1 hour'].join(' · '));
  return button;
}

function expectStationaryCost(before: any, after: any, minutes: number) {
  expect(after.day).toBe(before.day);
  expect(after.clock_minutes).toBe(before.clock_minutes + minutes);
  expect(after.driving_minutes_total).toBe(before.driving_minutes_total);
  expect(after.miles_traveled_actual).toBe(before.miles_traveled_actual);
}

async function expectCareOutcome(page: Page, before: any, persona: string, name: string, key: string) {
  const message = copy(key, { name });
  await expect(page.locator('.aftermath-panel .outcome-copy')).toHaveText(message);
  const after = await checkpoint(page);
  expectStationaryCost(before, after, 60);
  expect(after.crew_care.pending).toBeNull();
  expect(after.journal).toHaveLength(1);
  expect(after.journal[0].message).toBe(message);
  expect(after.journal[0].after).toEqual(after.stats);
  const member = after.party.members.find((candidate: any) => candidate.persona === persona);
  expect(member.name).toBe(name);
  return { after, member };
}

test('final workshop source contains all 28 town remarks and 27 narrative fields', () => {
  expect(Object.keys(reviewed.town_comments)).toHaveLength(28);
  expect(Object.keys(reviewed.locale_copy)).toHaveLength(27);
  // The workshop revises 28 towns; the 23 additional western towns stay in the game.
  expect(townFacts).toHaveLength(51);
  expect(new Set(townFacts.map(fact => fact.town)).size).toBe(51);
  for (const [town, remark] of Object.entries(reviewed.town_comments)) {
    const matches = townFacts.filter(fact => fact.town === town);
    expect(matches, town).toHaveLength(1);
    expect(matches[0].comment.en, town).toBe(remark);
  }
  for (const [key, expected] of Object.entries(reviewed.locale_copy)) {
    const actual = key.split('.').reduce((value, segment) => value?.[segment], english);
    expect(actual, key).toBe(expected);
  }
});

test('all 28 town conversations show their own revised remark and sourced fact offline', async ({ page, context }) => {
  test.setTimeout(180_000);
  const original = await baseline(page);
  await expect.poll(() => page.evaluate(() => (window as any).dystrailOffline?.state)).toBe('ready');
  await context.setOffline(true);
  let first = true;
  for (const [town, remark] of Object.entries(reviewed.town_comments)) {
    await test.step(town, async () => {
      const state = readyState(original);
      const route = routes.find((candidate: any) => candidate.stops.some((stop: any) => stop.name === town));
      expect(route, `${town} belongs to a playable route`).toBeDefined();
      state.persona_id = route.id;
      atTown(state, town, 0);
      await importState(page, state);
      if (first) {
        await page.reload();
        first = false;
      }
      const talk = page.getByRole('button', { name: 'Talk to locals', exact: true });
      await expect(talk).toBeEnabled();
      // Compare actual saved endpoints, after the import has normalized route precision.
      const before = await checkpoint(page);
      await talk.click();
      await expect(page.locator('.local-conversation')).toBeVisible();
      await expect(page.locator('.resident-byline')).toContainText(town);
      await expect(page.locator('.resident-remark p')).toHaveText(remark);
      const fact = townFacts.find(candidate => candidate.town === town)!;
      await expect(page.locator('.local-fact')).toHaveText(fact.text.en);
      await expect(page.locator('.fact-source a')).toHaveAttribute('href', fact.source);
      const heard = await checkpoint(page);
      expectStationaryCost(before, heard, 30);
      expect(heard.route_services.talked_at).toBe(state.route_services.stop);
      expect(heard.journal).toHaveLength(1);
      expect(heard.journal[0].message).toBe(fact.text.en);

      await page.locator('.conversation-actions button').click();
      await expect(page.locator('.town-arrival')).toBeVisible();
      await talk.click();
      await expect(page.locator('.resident-remark p')).toHaveText(remark);
      const reread = await checkpoint(page);
      expect(reread.stats).toEqual(heard.stats);
      expect(reread.receipts).toEqual(heard.receipts);
      expect(reread.clock_minutes).toBe(heard.clock_minutes);
      expect(reread.journal).toEqual(heard.journal);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    });
  }
  await context.setOffline(false);
});

const incidents = ['raw milk', 'heat warning', 'five accomplishments', 'wellness sample',
  'renamed detours', 'tariff invoice', 'Signal work chat', 'food parcels'];
for (const [reason, incident] of incidents.entries()) {
  test(`named crew incident: ${incident}, with care costs shown before choosing`, async ({ page, context }) => {
    const { state, persona, name } = crewState(await baseline(page), reason, 1);
    await importState(page, state);
    await context.setOffline(true);
    await page.reload();
    await expectNarrative(page, reason, name);
    const care = await expectChoice(page, 'journey.care_action', { morale: 1 });
    await expect(care).toBeEnabled();
    await expect(await expectChoice(page, 'journey.shelter_action', { morale: 1 })).toBeEnabled();
    await expect(await expectChoice(page, 'journey.defer_action', { sanity: 1 })).toBeEnabled();
    await expect(page.locator('.crew-incident .camp-actions > button')).toHaveCount(3);
    await care.click();
    const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_helped');
    expect(after.stats).toEqual({ ...state.stats, supplies: 8, morale: 9 });
    expect(member.status).toBe('Active');
    expect(after.crew_care.strain[persona]).toBeUndefined();
    expect(after.ending).toBeNull();
    await page.reload();
    expect((await checkpoint(page)).journal).toEqual(after.journal);
    expect((await checkpoint(page)).crew_care.strain[persona]).toBeUndefined();
    await context.setOffline(false);
  });
}

test('critical care saves the named companion for the advertised supplies and hour', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 3, 3);
  await importState(page, state);
  await expectNarrative(page, 3, name, true);
  await expect(await expectChoice(page, 'journey.fatal_action', { sanity: 1, morale: 3 })).toBeEnabled();
  await (await expectChoice(page, 'journey.care_action', { morale: 1 })).click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_helped');
  expect(after.stats).toEqual({ ...state.stats, supplies: 8, morale: 9 });
  expect(member.status).toBe('Active');
  expect(after.crew_care.strain[persona]).toBeUndefined();
  expect(after.ending).toBeNull();
});

test('leaving a critically ill companion with the aid team is safe and persists', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 7, 3);
  await importState(page, state);
  await expectNarrative(page, 7, name, true);
  const shelter = await expectChoice(page, 'journey.shelter_action', { morale: 1 });
  await expect(shelter).toBeEnabled();
  await shelter.click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_sheltered');
  expect(after.stats).toEqual({ ...state.stats, morale: 7 });
  expect(member.status).toBe('Departed');
  expect(after.crew_care.strain[persona]).toBeUndefined();
  expect(after.ending).toBeNull();
  await page.locator('#outcome-continue').click();
  await page.reload();
  const restored = await checkpoint(page);
  expect(restored.party.members.find((candidate: any) => candidate.persona === persona).status).toBe('Departed');
  expect(restored.journal).toEqual(after.journal);
});

test('continuing before illness becomes critical leaves the companion unwell', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 2, 2);
  await importState(page, state);
  await expectNarrative(page, 2, name);
  await expect(page.getByRole('button', { name: 'Continue without care', exact: true })).toHaveCount(0);
  await (await expectChoice(page, 'journey.defer_action', { sanity: 1 })).click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_deferred');
  expect(after.stats).toEqual({ ...state.stats, sanity: 7 });
  expect(member.status).toBe('Active');
  expect(after.crew_care.strain[persona]).toBe(2);
  expect(after.ending).toBeNull();
});

test('the explicitly fatal companion choice records death without ending the player journey', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 1, 3);
  await importState(page, state);
  await expectNarrative(page, 1, name, true);
  await expect(page.getByRole('button', { name: 'Keep driving without care', exact: true })).toHaveCount(0);
  await (await expectChoice(page, 'journey.fatal_action', { sanity: 1, morale: 3 })).click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_lost');
  expect(after.stats).toEqual({ ...state.stats, sanity: 7, morale: 5 });
  expect(member.status).toBe('Dead');
  expect(after.ending).toBeNull();
  await page.locator('#outcome-continue').click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'travel');
  expect((await checkpoint(page)).party.members.find((candidate: any) => candidate.persona === persona).status).toBe('Dead');
});

test('the player cannot use companion departure and their fatal choice ends this named journey', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 0, 3, true);
  await importState(page, state);
  await expectNarrative(page, 0, name, true);
  await expect(await expectChoice(page, 'journey.shelter_action', { morale: 1 })).toBeDisabled();
  await (await expectChoice(page, 'journey.fatal_action', { sanity: 1, morale: 3 })).click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.player_lost');
  expect(after.stats).toEqual({ ...state.stats, sanity: 7, morale: 5 });
  expect(member.status).toBe('Dead');
  expect(after.ending).toEqual({ type: 'collapse', cause: 'disease' });
  await expect(page.locator('.aftermath-panel')).not.toContainText('The crew continues without them.');
  await page.locator('#outcome-continue').click();
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'result');
  await expect(page.locator('#result-title')).toHaveText('ILLNESS ENDS THE JOURNEY');
  await expect(page.locator('.result-art .result-profile')).toContainText(name);
  await expect(page.locator('.ending-moments')).toContainText(copy('journey.player_lost', { name }));
  await expect(page.locator('.ending-moments')).not.toContainText('The crew continues without them.');
});

test('unaffordable care stays disabled while a safe companion departure remains available', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 5, 3);
  state.stats.supplies = 1;
  await importState(page, state);
  await expectNarrative(page, 5, name, true);
  await expect(await expectChoice(page, 'journey.care_action', { morale: 1 })).toBeDisabled();
  const shelter = await expectChoice(page, 'journey.shelter_action', { morale: 1 });
  await expect(shelter).toBeEnabled();
  expect((await checkpoint(page)).stats).toEqual(state.stats);
  await shelter.click();
  const { after, member } = await expectCareOutcome(page, state, persona, name, 'journey.care_sheltered');
  expect(after.stats).toEqual({ ...state.stats, morale: 7 });
  expect(member.status).toBe('Departed');
});

for (const [reason, day, contact] of [
  [0, 6, 'Jules'], [1, 7, 'Kit'], [2, 8, 'Ari'],
  [3, 9, 'Blair'], [4, 10, 'Casey'], [5, 11, 'Devon'],
] as const) {
  test(`outside ally departure ${reason}: ${contact} is named and the crew is unchanged`, async ({ page, context }) => {
    const state = readyState(await baseline(page));
    state.day = day;
    // No generated crew name can collide with these six known outside contacts.
    state.party.members.forEach((member: any, index: number) => { member.name = `Crew ${index + 1}`; });
    // An imported encounter delivers the same actual loss and log marker as ally
    // attrition. This isolates presentation from a random daily attrition roll.
    state.current_encounter = {
      ...stories[0], id: 'writer_narrative_ally_notice', name: 'A contact calls',
      desc: 'A contact has an update for the crew.',
      choices: [{ label: 'Take the call', effects: { allies: -1, morale: -1, log: 'log.ally.lost' } }],
    };
    await importState(page, state);
    await context.setOffline(true);
    await page.reload();
    await page.getByRole('button', { name: '1) Take the call', exact: true }).click();
    await expect(page.locator('#main')).toHaveAttribute('data-screen', 'ally-loss');
    const message = copy(`ally_loss.reason_${reason}`, { name: contact });
    await expect(page.locator('.ally-message')).toHaveText(message);
    await expect(page.getByRole('button', { name: 'Travel', exact: true })).toBeDisabled();
    const after = await checkpoint(page);
    expectStationaryCost(state, after, 30);
    expect(after.stats).toEqual({ ...state.stats, allies: 2, morale: 7 });
    expect(after.party).toEqual(state.party);
    expect(after.ally_notice.message).toBe(message);
    expect(after.journal).toHaveLength(1);
    expect(after.journal[0].message).toBe(message);
    await page.reload();
    await expect(page.locator('.ally-message')).toHaveText(message);
    await page.locator('#outcome-continue').click();
    await expect(page.locator('.ally-departure')).toHaveCount(0);
    const acknowledged = await checkpoint(page);
    expect(acknowledged.ally_notice).toBeNull();
    expect(acknowledged.party).toEqual(state.party);
    expect(acknowledged.stats).toEqual(after.stats);
    expect(acknowledged.journal).toEqual(after.journal);
    expect(acknowledged.clock_minutes).toBe(after.clock_minutes);
    await context.setOffline(false);
  });
}
