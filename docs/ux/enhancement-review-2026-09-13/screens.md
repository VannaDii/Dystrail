**Dystrail: screen review and evidence**

Prepared September 13, 2026; updated for time-of-day lighting and the user's decision to retain randomness with conditional hearing rounds. Companion to the [enhancement plan](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/plan.md). The original screenshots are unchanged; the added lighting and finale findings come from source inspection, not a new visual playthrough.

This review combines the user's eleven screenshot attachments, six new captures from a short local walkthrough, and the inspected source. The walkthrough covered entry, character selection, naming, outfitting, travel, and a genuine encounter. It did not complete a campaign or reproduce every reported interruption. The later care, hunger, hearing, and result screens are user references, supported by source inspection. The [evidence manifest](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence-manifest.json) records the origin of every image.

“Functional” below means that the reviewed interaction or underlying path exists; it does not mean a comprehensive test passed. Health judgments concern the requested enhancements. All proposed behavior remains subject to the implementation plan.

| Journey step | Health in the reviewed experience | Related ideas | Evidence |
|---|---|---|---|
| 1. Enter and name a run | Setup works; repeat-play continuity and shared-link intake are incomplete. Character revision needs a coordinated visual brief. | 3, 7, 8 | [Entry](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/01-current-entry.jpg), [selection](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/02-current-personas.jpg), [names](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/03-current-names.jpg) |
| 2. Outfit the crew | Purchasing works, but protection is stored as possession tags rather than the quantities the new rules require. | 2, 9 | [Current outfitting](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/04-current-outfitting.jpg), [user reference](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-outfitting.png) |
| 3. Travel and recognize a new day | The moving/paused distinction is clear. Repeated critical-stat stops and the brief settings window weaken continuity; seating and clearer time-of-day lighting need coordinated design. | 1, 4, 11, 13, 14 | [Paused](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-paused-travel.png), [moving](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-moving-travel.png), [seating detail](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-van-seating.png) |
| 4. Meet an incident and choose | A real encounter correctly interrupted the local walkthrough. Choices reveal exact stat forecasts; major policy changes lack the requested announcement scene. | 2, 10 | [Current encounter](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/06-current-encounter.jpg), [care reference](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-crew-care.png) |
| 5. Recover from empty supplies | The condition has consequences in the engine; the supplied screen gives little explanation beyond the red zero. | 4, 5, 9 | [Empty supplies](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-empty-supplies.png) |
| 6. Resolve the hearing | Stat-influenced randomness is retained. The current three fixed stamina rounds and immediate result need a staged, variable-round presentation and corresponding rule changes. | 12 | [Hearing reference](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-hearing.png) |
| 7. Understand the result and replay | Actual values and replay actions exist. Comparisons must distinguish starting odds, hearing influence, final odds, and outcome; the current score target and vote outcome differ. | 6, 7 | [Scorecard reference](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-scorecard.png) |

**1. Enter and name a run**

Observed: the local sequence allowed character selection, crew/player naming, and progression to outfitting. The existing role cards make selection explicit. The user's references show the desired fields and the optional run-code field.

Source finding: current-run recovery can restore names, while fresh setup has no separate last-used crew/player preference. The run-code parser already exists, but setup initializes a default code and no URL intake was found. Characters have different origins/routes, so code equality alone does not establish identical trails.

Design consequence: add preferences with clear precedence below an existing save or an active edit. A code-only URL should prefill the code; a complete shared-trail link should also identify the persona and supported rules. Preserve an active run when opening a different linked run. The cast brief should define gender-neutral or feminine presentation and racial diversity explicitly, then govern every depiction; names or visual guesses should not determine pronouns.

![User reference: character selection](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-personas.png)

Additional references: [crew naming](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-crew-names.png), [run-code input](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-run-code.png).

**2. Outfit the crew**

Observed: the shop exposes prices, quantity controls, spending, supplies after checkout, and remaining cash. These are useful decision aids to retain. The current presentation groups equipment clearly.

Source finding: protective gear is reduced to inventory tags at purchase. One qualifying tag grants the corresponding full current benefit, regardless of how many travelers are present. Purchase caps also prevent buying six of each item. Town resupply already exists and must be extended rather than replaced.

Design consequence: this requires a simulation and economy change before new shop copy can be finalized. Display coverage, condition, and reserve stock; retain precise prices and quantities while replacing promised stat gains with qualitative descriptions. At current list prices, a coat, mask, and poncho for all six travelers totals $144 before food or spares. Starting budgets are $90–$130. The intended tradeoffs need viable starting packages and replacement opportunities.

![User reference: outfitting](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-outfitting.png)

**3. Travel and recognize a new day**

Observed: the supplied moving and paused screens clearly distinguish seated travelers from the roadside crew. The landscape provides room for environmental storytelling. The enlarged van reference shows substantial overlap between two occupants.

Source finding: the travel callback turns continuous travel off whenever supplies, health, or sanity are at or below 2. This is a repeated condition check, not a notification tied to entering a critical state. Document visibility and genuine phase changes also stop travel; normal-speed automatic map previews add another transition. The source supports a likely explanation for some unwanted pauses, but the user's particular pauses were not reproduced during this short walkthrough.

The simulation uses a five-hour driving window beginning at 08:00. Daily initialization locks pace and information diet, and automatic travel can quickly consume the next day's editing opportunity. Stationary activities can also cross day boundaries. The information diet is not a food-ration setting.

Design consequence: coordinate explicit pause reasons, a single morning planning checkpoint, announcements, and pending decisions. Show the end of the driving day without portraying 13:00 as sunset. Billboards need their own layer because background sections can be mirrored. Create near/far seated poses around defined seat boundaries, keeping presence and replacement-driver behavior intact.

Lighting follow-up: the world scene receives the current clock hour. Morning is currently 06:00–09:59, dusk 17:00–20:59, night 21:00–05:59, and the intervening hours share the daylight appearance. CSS applies tints/gradients and background darkening. The supplied 11:00 and 12:00 travel references therefore fall in the same band; they do not prove that all time-based lighting is absent. Expand this into approved morning/midday/afternoon/dusk/night profiles, using representative moving/stopped comparisons and coordinated sky, shadows, van, crew, and sign treatment. Keep real game time authoritative, including through overnight transitions and restored reports. Visual effectiveness across those times remains to be checked.

![User reference: moving travel](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-moving-travel.png)

![User reference: overlapping seated occupants](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-van-seating.png)

The [new local travel capture](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/05-current-travel.jpg) showed colored matte areas around standing crew that are absent from the supplied Safari image. This is a browser/rendering discrepancy to check during art integration, not a confirmed defect in every environment.

**4. Meet an incident and choose**

Observed: normal travel in the new run reached a “Pop-up Mutual Aid” encounter and stopped for a real choice. The care reference demonstrates the numerical forecasts the user wants replaced.

Source finding: the current encounter data contains 65 families and 187 choices. Numerical forecasts are assembled into visible text, tooltips, and accessible descriptions. Other activities use separate formatting paths. Active executive orders have HUD descriptions and brief highlighting, but no dedicated announcement phase.

Design consequence: define one consequence vocabulary and apply it across the relevant paths. Keep certainty, risk, irreversible departures, actual offers, affordability, and elapsed time intelligible. The new event scene should announce an already committed event once and then continue to the next required state. Reloading or reopening it must not apply the event again. Major-event prominence and fewer pointless stops depend on the same interruption rules.

![User reference: crew-care options](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-crew-care.png)

**5. Recover from empty supplies**

Observed: the user's screen shows zero supplies in a red block while Travel is still available. The supplied end-of-run screen also records starvation taking its toll.

Source finding: starvation already has a grace period, recurring health/sanity costs, illness consequences, and a limited backstop. This review did not simulate a full hunger sequence or prove its balance. The Oregon Trail comparison and the resulting recommendation are documented in the plan.

Design consequence: keep a recoverable survival condition, make its progression and useful actions clear, and prevent unchanged low supplies from demanding a new resume click every step. PPE costs must be tested alongside food access so the new protection model does not unintentionally turn hunger into the dominant failure mode. Rest is not a substitute for finding supplies.

![User reference: empty supplies](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-empty-supplies.png)

**6. Resolve the hearing**

Observed: the user reference presents an estimated vote chance, score contributions, round count, sanity cost, and a resolution button. It gives information but little staged closure.

Source finding: resolution deducts sanity through up to three rounds and then makes a single seeded vote roll. Exhaustion can end the hearing early. Other stats feed the raw journey score used to calculate odds. The ordinary chance cap is 88%; an internal Deep/Aggressive policy overrides the chance to 100% after stamina. Separate won/lost debate rounds do not currently exist. Boss-specific stat-weight and required-pass configuration fields are not used in the active resolution calculation.

Design consequence: retain the current stat-based starting odds and model the user's conditional rounds. Always resolve round 1, then roll whether rounds 2 and 3 occur; charge 2 sanity for each actual round and average only their performance values as an odds multiplier. After a surviving hearing closes, adjusted odds of at least 100% produce automatic victory; otherwise make the final vote draw. The [hearing model](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/hearing-model.md) records provisional influence ranges and continuation chances. Resolve/save once, then reveal stages; skip and reload must preserve outcomes. Player-controlled reveals provide pacing engagement, while tactical choices would be an additional design decision. The deterministic-victory recommendation is withdrawn.

![User reference: hearing](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-hearing.png)

**7. Understand the result and replay**

Observed: the scorecard already gives a seed, actual values, a score target, and multiple replay/share actions. Its cards have space for the requested comparison. The supplied result was an early ending, so it must not be interpreted as an observed vote loss.

Source finding: stat subtitles are empty. The displayed score applies multipliers that differ from the raw score used in vote probability. Health, morale, and sanity are missing from the current scorecard's stat collection despite their relevance. Meeting the displayed target and winning the final vote are distinct.

Design consequence: use the same staged-hearing model for resolution, forecasts, and result explanations. Show the base odds, actual influence, adjusted odds, and outcome. A stat-change comparison can hold a completed hearing's actual rolls fixed, but must state that condition and handle cases where no legal stat-only change would have won. Exhaustion and early journey endings must not invent an unseen final vote. A display-score target is a separate benchmark. Generate a complete replay link and preserve historical explanations for older rules.

![User reference: journey scorecard](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/evidence/reference-scorecard.png)

**Accessibility and verification limits**

The reviewed screens retain useful strengths: consistent scene framing, recognizable controls, clear current stats, and explicit routes. The proposed changes should preserve those while making time, risk, and outcomes easier to understand.

No keyboard-only campaign, assistive-technology session, mobile walkthrough, contrast measurement, or complete localization pass was performed. These are implementation acceptance checks, not passed findings. In particular:

- Billboards need readable type, contrast, enough reading time, a paused view, and an accessible text equivalent. They must not cover actions or carry essential information that disappears in motion.
- Time-of-day and weather layers need combined contrast checks. Changes must follow simulated time, preserve readable crew/sign silhouettes, and avoid dimming controls or relying on color alone to communicate the day boundary.
- Daily/event/hearing transitions need logical focus placement and return, concise announcements, keyboard continuation, and reduced-motion/skip equivalents that preserve decisions.
- Qualitative descriptions must match visible text, tooltips, and accessible labels. Critical consequences cannot be conveyed only through color or vague language.
- Equipment needs text for covered, worn, expired, and reserve states; icons and color should supplement it.
- New cast art should preserve identifiable roles and useful alternative descriptions without inferring gender or ethnicity from appearance.
- Text enlargement, small screens, long translations, and Arabic layout need review on the final compositions.

The six new captures are evidence of a bounded local walkthrough. They do not verify that a production build matches this checkout. Some full-page captures contain extra blank canvas; the user references remain the clearest visual evidence for the cited layout details.
