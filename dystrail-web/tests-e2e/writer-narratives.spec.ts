import { readFileSync } from 'node:fs';
import { test, expect, type Page } from '@playwright/test';
import { baseline, importState, waitForLaunch } from './helpers';
import { atTown, routes } from './geography';

// Final authored workshop snapshot, captured 2026-09-13. These pinned strings
// do not come from the game bundle: an omitted import must fail here.
// Document: 1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k
// Revision: ANLCKQnwMWWXuS4ELr1I0ohWwa1b-7h9hCUDSdGapEtxM96pfap2t9ljAaL7YlDbeOUqHkCw-mxgQk7SVY7QXTBMrGZhoF7TGhpoQofaZjM
// Handoff SHA-256: b3c9cc5db50c6f6dfbe402527c5bf14cab129315550f6fba0a4ad195db3b425b
const reviewed: {
  town_comments: Record<string, string>;
  locale_copy: Record<string, string>;
} = {
  "town_comments": {
    "Albuquerque": "“Trump wants projects for cars.” She points at your shoes. “Put a license plate on those. We might get you a sidewalk.”",
    "Amarillo": "“Trump’s cheaper-beef plan upset our ranchers.” He inspects his sandwich. “I’ll notify Argentina when the affordable bit arrives.”",
    "Austin": "“The federal grant vanished. Veterans can still appear in campaign ads; apparently letting them discuss a book is where Washington draws the line.”",
    "Billings": "“USDA approved Montana’s food ban, with exceptions for protein bars and fresh baked goods. Hunger is simple. Your snack now needs a classification hearing.”",
    "Boise": "“Trump’s funding freeze was withdrawn, but the other orders still needed checking. Washington canceled the padlock and left the chain.”",
    "Cheyenne": "“Gray cheered Trump for ending wind and solar subsidies, then cheered coal and oil. A free market, with the secretary of state holding the scorecards.”",
    "Chicago": "“Trump’s agencies paused the train money. CTA sued. Apparently the Red Line needs a transfer through federal court before it can reach another neighborhood.”",
    "Cleveland": "“Trump’s administration cut GEAR UP, the program helping students toward college. The ladder is still available; Washington just repossessed the bottom rungs.”",
    "Columbia": "“Governor Kehoe ended Missouri’s paid-sick-leave requirement. My fever can still stay home. It just needs to find somebody else to pay the rent.”",
    "Cumberland": "“Trump cut park staff nationally. Here the only carpenter took early retirement. If that bench collapses, we’ll call it a successful transition to meadow.”",
    "Dallas": "They call freezers overhead. Let a senator keep his lunch at room temperature and see how fast overhead becomes critical infrastructure.",
    "Denver": "Washington said help the arrivals, then refused the reimbursement. I tried that with my landlord: patriotic housing partnership. He still wants the rent.",
    "Des Moines": "Trump’s USDA canceled the local-food programs. The apples still passed inspection. Unfortunately, they failed the new test: being lunch for someone who needs it.",
    "El Paso": "Trump made border land part of an Army base. Cross the same dirt, collect an extra charge. Even the ground got promoted before I did.",
    "Flagstaff": "Trump proposed closing the canyon’s river-monitoring center. If I stopped checking leaks to save money, I would be called a former plumber.",
    "Frederick": "Ninety health jobs marked for cuts. If I streamlined the hospital by removing the toilets, I would still have a mop problem.",
    "Hagerstown": "Trump and Musk floated privatizing the mail. Great: Grandma can become a premium delivery zone. She has always wanted to live inside a surcharge.",
    "Iowa City": "Trump’s State Department cut the writing program. More than ninety percent of its grant money stayed in America. Apparently America First has a spelling exception.",
    "Joplin": "EPA removed lead from local yards. Trump proposed cutting its budget in half. I suppose the children could submit smaller sandboxes.",
    "Kansas City": "Trump’s USDA canceled food deliveries. The milk never arrived, but the savings did. Unfortunately, the children keep asking for the kind you can pour.",
    "La Crosse": "Trump’s budget proposed closing our river science center. The fish cannot lobby. We are teaching one to hold a checkbook, but the ink runs.",
    "Las Vegas": "Trump wanted fewer foreigners coming in. Some tourists took the hint. My landlord has declined to accept a stronger border instead of rent.",
    "Madison": "Trump interrupted research grants, so the university found bridge money. Science is driving on the spare while Washington takes credit for reducing tire expenses.",
    "Minneapolis": "Trump terminated research awards. Seventy-two of them. The university still studies uncertainty; Washington has simply moved the experiment into everyone’s paycheck.",
    "Missoula": "Trump withdrew the blanket funding freeze. Other reviews stayed. The fridge is open, the food is still locked up, and everyone applauds the thaw.",
    "North Platte": "Trump’s cuts left fewer people to launch weather balloons. One forecast balloon a day instead of two. The afternoon will have to stop doing weather.",
    "Oklahoma City": "Trump’s USDA canceled local-food funding. Now the pantry asks neighbors to fill the gap. Small government comes with a surprisingly large grocery list.",
    "Omaha": "Trump’s NIH cap treats the light bill as expendable overhead. Fine. Let the next medical breakthrough develop a natural immunity to darkness.",
    "Phoenix": "Mayes says lawsuits protected a billion and a half in federal funds. Washington has added a new grant requirement: bring your own attorney general.",
    "Pittsburgh": "Trump’s agencies canceled ninety-one Pitt awards. The researchers still have questions. Washington has funded the answer: we cannot afford to find out.",
    "Rapid City": "Trump made his birthday a free park day. I tried celebrating mine at the gate. Apparently the birthday discount requires executive experience.",
    "Reno": "Trump proposed capping the money that keeps labs running. Courts paused it. My lunch now has two protections: this cooler and the separation of powers.",
    "Salt Lake City": "Trump’s NIH proposal threatened forty-five million in research revenue. Apparently the cure should cover its own rent. Has anyone tried asking the disease for a deposit?",
    "San Antonio": "Trump froze funding; the district kept programs running while waiting for instructions. The children also kept being children. Terrible failure to coordinate with Washington.",
    "Sioux Falls": "Federal cuts left Feeding South Dakota asking legislators for three million. Hunger now needs a committee hearing. It has prepared some very brief stomach remarks.",
    "South Bend": "Trump’s funding and tax uncertainty put campus hiring on ice. The dishes still arrive hot. Washington has created a hiring freeze with excellent steam.",
    "Spokane": "Trump’s administration rescinded the home-cooling grant. The heat kept its funding. I am applying to become a federally recognized breeze.",
    "Springfield, IL": "Congress shut down government and Lincoln tours closed. You could admire democracy through the window. Please do not touch; it was a display model.",
    "Springfield, MO": "Trump’s USDA canceled three million dollars in food. A third was protein. The pantry has plenty of explanations, but nobody has found a recipe yet.",
    "St. Louis": "Trump canceled the next local-farm food round. The last one can finish. Apparently hunger is supposed to respect the end of a fiscal paragraph.",
    "Toledo": "Trump says tariffs protect American business. Our American paper supplier added a surcharge. The envelope industry finally has an answer to what is inside: another bill.",
    "Tucson": "Trump’s grant portals kept shutting. Nearly two hundred jobs faced funding risk. New job description: convince a website that Tucson still exists.",
    "Tulsa": "Trump’s cuts have health officials guessing what Washington will fund. I asked whether uncertainty is contagious. They need a grant to find out.",
    "Waco": "Washington announced half the benefits. I cut one sandwich into two. Same accounting method, but mine comes with crusts."
  },
  "locale_copy": {
    "trail.care_reason_0": "{name} tries raw milk from a “Make America Healthy Again” stall. The vendor calls it “alive with goodness.” Judging by the screaming from the toilet, the goodness has a knife.",
    "trail.care_reason_1": "During a paid unloading shift, {name} becomes dehydrated. The boss crossed out the heat warning with a Sharpie, but apparently, the sun doesn’t take corrections. #sharpiegate",
    "trail.care_reason_2": "After a temp shift, {name} stays up proving those hours count under Congress’s benefits work rules. By dawn, the only unpaid work is proving they work.",
    "trail.care_reason_3": "{name} tries a MAHA tonic promising to “drain the swamp” from their body. They vomit into the sample bucket. The swamp has acquired a gift shop.",
    "trail.care_reason_4": "{name} loses sleep reconciling maps after Trump’s Gulf and mountain renaming. The van still needs the same turn. Geography has failed its loyalty test.",
    "trail.care_reason_5": "{name} spends all night disputing Trump’s tariff on a van part. By dawn, they’re waving a white napkin at the card reader. It asks for a tip.",
    "trail.care_reason_6": "{name} loses sleep helping a DOGE-cut friend list their work for an efficiency review. They count five tasks. The person requesting the list has created a sixth.",
    "trail.care_reason_7": "At a paid unloading job for a billionaire’s political fundraiser, {name} injures their back moving his memoir, I Did It All Myself. The donor steps over them to reach the podium.",
    "ally_loss.reason_0": "Your outside contact {name} quits helping and joins a MAGA merchandise drive. Their goodbye includes a discount code. Political conversion now has affiliate tracking.",
    "ally_loss.reason_1": "DOGE cuts cost your outside contact {name} their job. They withdraw to find work. Their last assignment was to document the value of documenting their value.",
    "ally_loss.reason_2": "Your contact {name} takes extra shifts to cover tariff-inflated repair costs and can’t keep helping. They’ve found out which foreign country pays. It’s the one printed on their address.",
    "ally_loss.reason_3": "Your contact {name} leaves to sell merchandise to rival political rallies. They’ve solved the two-party problem with two card readers.",
    "ally_loss.reason_4": "Your contact {name} sends a video claiming journalists invented the Signal scandal, then blocks the crew. The explanation is now more secure than the bombing chat.",
    "ally_loss.reason_5": "Your contact {name} twists an ankle outside a political fundraiser and withdraws. The accessible entrance was reserved for donors. Equal access was available at the next sponsorship level.",
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
    "journey.care_hint": "Each choice takes one hour. Care costs two supplies. Only companions can leave with the medics. A full recovery day clears illness.",
    "workshop.CARE-01.title": "Alive With Goodness",
    "workshop.CARE-01.setup": "{name} tries raw milk from a “Make America Healthy Again” stall. The vendor calls it “alive with goodness.” Judging by the screaming from the toilet, the goodness has a knife.",
    "workshop.CARE-01.continuing": "{name} is still ill from the MAHA raw milk. The vendor has promoted the stomach cramps to a cleansing process.",
    "workshop.CARE-01.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-01.outcomes.helped": "{name} recovers with care. The vendor asks for a testimonial. You offer the bathroom key.",
    "workshop.CARE-01.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-01.outcomes.deferred": "{name} is still ill from the MAHA raw milk. The vendor has promoted the stomach cramps to a cleansing process.",
    "workshop.CARE-01.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-01.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-01.choices.c0.label": "Provide care",
    "workshop.CARE-01.choices.c0.outcome": "{name} recovers with care. The vendor asks for a testimonial. You offer the bathroom key.",
    "workshop.CARE-01.choices.c1.label": "Leave in shelter",
    "workshop.CARE-01.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-01.choices.c2.label": "Press on",
    "workshop.CARE-01.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-02.title": "Executive Forecast",
    "workshop.CARE-02.setup": "During a paid unloading shift, {name} becomes dehydrated. The boss crossed out the heat warning with a Sharpie, but apparently, the sun doesn’t take corrections. #sharpiegate",
    "workshop.CARE-02.continuing": "{name} is still dehydrated. The Sharpie has made a full recovery and returned to management.",
    "workshop.CARE-02.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-02.outcomes.helped": "Care restores {name}. The sun continues operating outside the boss’s chain of command.",
    "workshop.CARE-02.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-02.outcomes.deferred": "{name} is still dehydrated. The Sharpie has made a full recovery and returned to management.",
    "workshop.CARE-02.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-02.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-02.choices.c0.label": "Provide care",
    "workshop.CARE-02.choices.c0.outcome": "Care restores {name}. The sun continues operating outside the boss’s chain of command.",
    "workshop.CARE-02.choices.c1.label": "Leave in shelter",
    "workshop.CARE-02.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-02.choices.c2.label": "Press on",
    "workshop.CARE-02.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-03.title": "Proof of Labor",
    "workshop.CARE-03.setup": "After a temp shift, {name} stays up proving those hours count under Congress’s benefits work rules. By dawn, the only unpaid work is proving they work.",
    "workshop.CARE-03.continuing": "{name} is exhausted from the benefits paperwork. Their shift is documented; the night spent proving it remains volunteer work.",
    "workshop.CARE-03.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-03.outcomes.helped": "Care helps {name} recover. You put the forms away before they develop a work requirement of their own.",
    "workshop.CARE-03.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-03.outcomes.deferred": "{name} is exhausted from the benefits paperwork. Their shift is documented; the night spent proving it remains volunteer work.",
    "workshop.CARE-03.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-03.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-03.choices.c0.label": "Provide care",
    "workshop.CARE-03.choices.c0.outcome": "Care helps {name} recover. You put the forms away before they develop a work requirement of their own.",
    "workshop.CARE-03.choices.c1.label": "Leave in shelter",
    "workshop.CARE-03.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-03.choices.c2.label": "Press on",
    "workshop.CARE-03.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-04.title": "Drain the Sample Bucket",
    "workshop.CARE-04.setup": "{name} tries a MAHA tonic promising to “drain the swamp” from their body. They vomit into the sample bucket. The swamp has acquired a gift shop.",
    "workshop.CARE-04.continuing": "{name} is still sick from the tonic. The vendor says discomfort proves it works. Refunds apparently prove nothing.",
    "workshop.CARE-04.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-04.outcomes.helped": "{name} recovers with care. The vendor offers a loyalty card. You suggest a mop.",
    "workshop.CARE-04.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-04.outcomes.deferred": "{name} is still sick from the tonic. The vendor says discomfort proves it works. Refunds apparently prove nothing.",
    "workshop.CARE-04.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-04.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-04.choices.c0.label": "Provide care",
    "workshop.CARE-04.choices.c0.outcome": "{name} recovers with care. The vendor offers a loyalty card. You suggest a mop.",
    "workshop.CARE-04.choices.c1.label": "Leave in shelter",
    "workshop.CARE-04.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-04.choices.c2.label": "Press on",
    "workshop.CARE-04.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-05.title": "Patriotically Lost",
    "workshop.CARE-05.setup": "{name} loses sleep reconciling maps after Trump’s Gulf and mountain renaming. The van still needs the same turn. Geography has failed its loyalty test.",
    "workshop.CARE-05.continuing": "{name} is exhausted. Two maps disagree about the name of something that has not moved.",
    "workshop.CARE-05.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-05.outcomes.helped": "Care restores {name}. You follow the road, which has managed to stay in office.",
    "workshop.CARE-05.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-05.outcomes.deferred": "{name} is exhausted. Two maps disagree about the name of something that has not moved.",
    "workshop.CARE-05.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-05.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-05.choices.c0.label": "Provide care",
    "workshop.CARE-05.choices.c0.outcome": "Care restores {name}. You follow the road, which has managed to stay in office.",
    "workshop.CARE-05.choices.c1.label": "Leave in shelter",
    "workshop.CARE-05.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-05.choices.c2.label": "Press on",
    "workshop.CARE-05.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-06.title": "A Very Small Trade War",
    "workshop.CARE-06.setup": "{name} spends all night disputing Trump’s tariff on a van part. By dawn, they’re waving a white napkin at the card reader. It asks for a tip.",
    "workshop.CARE-06.continuing": "{name} is exhausted from the tariff dispute. China has not sent a relief driver.",
    "workshop.CARE-06.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-06.outcomes.helped": "{name} recovers with care. You declare a ceasefire with the receipt printer.",
    "workshop.CARE-06.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-06.outcomes.deferred": "{name} is exhausted from the tariff dispute. China has not sent a relief driver.",
    "workshop.CARE-06.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-06.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-06.choices.c0.label": "Provide care",
    "workshop.CARE-06.choices.c0.outcome": "{name} recovers with care. You declare a ceasefire with the receipt printer.",
    "workshop.CARE-06.choices.c1.label": "Leave in shelter",
    "workshop.CARE-06.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-06.choices.c2.label": "Press on",
    "workshop.CARE-06.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-07.title": "Five Things Before Breakfast",
    "workshop.CARE-07.setup": "{name} loses sleep helping a DOGE-cut friend list their work for an efficiency review. They count five tasks. The person requesting the list has created a sixth.",
    "workshop.CARE-07.continuing": "{name} remains exhausted. The review has classified time spent answering the review as administrative waste.",
    "workshop.CARE-07.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-07.outcomes.helped": "Care restores {name}. You record one completed task: keeping an actual person functioning.",
    "workshop.CARE-07.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-07.outcomes.deferred": "{name} remains exhausted. The review has classified time spent answering the review as administrative waste.",
    "workshop.CARE-07.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-07.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-07.choices.c0.label": "Provide care",
    "workshop.CARE-07.choices.c0.outcome": "Care restores {name}. You record one completed task: keeping an actual person functioning.",
    "workshop.CARE-07.choices.c1.label": "Leave in shelter",
    "workshop.CARE-07.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-07.choices.c2.label": "Press on",
    "workshop.CARE-07.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen.",
    "workshop.CARE-08.title": "Self-Made, Staff Carried",
    "workshop.CARE-08.setup": "At a paid unloading job for a billionaire’s political fundraiser, {name} injures their back moving his memoir, I Did It All Myself. The donor steps over them to reach the podium.",
    "workshop.CARE-08.continuing": "{name} remains injured. The donor’s speech about self-reliance has required another pair of hands to turn the pages.",
    "workshop.CARE-08.critical": "{name} is critically unwell. Pressing on without care now will kill them. If this is your character, the journey will end.",
    "workshop.CARE-08.outcomes.helped": "Care helps {name} recover. You leave the remaining memoirs to pull themselves up by their dust jackets.",
    "workshop.CARE-08.outcomes.sheltered": "{name} remains alive in shelter and has left the expedition.",
    "workshop.CARE-08.outcomes.deferred": "{name} remains injured. The donor’s speech about self-reliance has required another pair of hands to turn the pages.",
    "workshop.CARE-08.outcomes.companion_lost": "{name} has died after the crew continued without care. Their place in the van is empty.",
    "workshop.CARE-08.outcomes.player_lost": "You have died after continuing without care. Your journey ends here.",
    "workshop.CARE-08.choices.c0.label": "Provide care",
    "workshop.CARE-08.choices.c0.outcome": "Care helps {name} recover. You leave the remaining memoirs to pull themselves up by their dust jackets.",
    "workshop.CARE-08.choices.c1.label": "Leave in shelter",
    "workshop.CARE-08.choices.c1.outcome": "{name} stays in shelter and leaves the traveling crew alive. The incident is cleared.",
    "workshop.CARE-08.choices.c2.label": "Press on",
    "workshop.CARE-08.choices.c2.outcome": "You continue without care. {name} remains unwell, and the incident can worsen."
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

async function expectNarrative(page: Page, reason: number, name: string, critical = false, continuing = false) {
  await expect(page.locator('#main')).toHaveAttribute('data-screen', 'crew-care');
  await expect(page.locator('.journey-scene h1')).toHaveText(`${name} · ${copy('journey.crew_stop')}`);
  const family = `workshop.CARE-${String(reason + 1).padStart(2, '0')}`;
  const expected = [copy(`${family}.${critical || continuing ? 'continuing' : 'setup'}`, { name }),
    ...(critical ? [copy(`${family}.critical`, { name })] : [])].join(' ');
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
  const fields: Record<string, string> = { 'journey.care_helped':'helped', 'journey.care_sheltered':'sheltered', 'journey.care_deferred':'deferred', 'journey.care_lost':'companion_lost', 'journey.player_lost':'player_lost' };
  const message = copy(`workshop.CARE-${String(before.crew_care.reason + 1).padStart(2, '0')}.outcomes.${fields[key]}`, { name });
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

test('final workshop source contains all 44 town remarks and scenario-specific care fields', () => {
  expect(Object.keys(reviewed.town_comments)).toHaveLength(44);
  expect(Object.keys(reviewed.locale_copy)).toHaveLength(147);
  // The workshop revises all 44 intermediate towns; seven endpoint records remain unchanged.
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

test('all 44 town conversations show their own revised remark and sourced fact offline', async ({ page, context }) => {
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
        await page.reload();await waitForLaunch(page);
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
    await page.reload();await waitForLaunch(page);
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
    await page.reload();await waitForLaunch(page);
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
  await page.reload();await waitForLaunch(page);
  const restored = await checkpoint(page);
  expect(restored.party.members.find((candidate: any) => candidate.persona === persona).status).toBe('Departed');
  expect(restored.journal).toEqual(after.journal);
});

test('continuing before illness becomes critical leaves the companion unwell', async ({ page }) => {
  const { state, persona, name } = crewState(await baseline(page), 2, 2);
  await importState(page, state);
  await expectNarrative(page, 2, name, false, true);
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
  await expect(page.locator('#result-title')).toHaveText('Illness Has No Upload Button');
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
    await page.reload();await waitForLaunch(page);
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
    await page.reload();await waitForLaunch(page);
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
