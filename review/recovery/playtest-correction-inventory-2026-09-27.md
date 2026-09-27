# Playtest visual correction inventory — 2026-09-27

This is the correction ledger for the current recovery worktree. Counts are **scene variants or locations**, not image-generation requests. Categories can overlap. Reuse and composition come before new art; no additional generation is authorized by this ledger.

## Directly observed in the 27 September playtest

| Area | Correction | Production decision |
|---|---|---|
| All world scenes | Enforce the same short scene frame on a given viewport; keep actions in reach, retain bottom-left captions, avoid stretching figures | CSS camera/crop plus desktop and phone inspection of every scene family |
| Moving van | Seat the actual surviving crew in front of seatbacks with consistent scale and windshield/door occlusion | Correct existing 1536×1024 composite; reuse approved cast/van art |
| Moving billboards | Make signs pass the van, space them evenly and farther apart, freeze each sign’s copy, increase the run-seeded pool to at least 3× the measured mean signs per leg | Layout/selection/copy; reuse text-free props and translate copy |
| Clean-air banner and other blank signs | Determine whether intentionally blank source art lacks its HTML overlay; add localized satire in interface layer only | No embedded text in raster art |
| Community-offer outcomes | Bind retained outcome compositions where correct; hold mismatches | Map each offer/result to an approved asset |
| Hotel biscuit scene | Hotel lift bank plus stairs, with a coherent biscuit/jet premise | ENC-C18-B: retained asset search, then review hold if none fits |
| 51 towns | Show recognizable local geography or landmarks rather than generic town art | Reference publicly available current place imagery, then use approved pixel-art style with attributable source review |
| Result scene | Keep scorecard and navigation reachable; the pending crossing scene must show its Continue action even after the run enters the Result phase | Render decision children without the ordinary journey toolbar in Result; rename the misleading story title |

## Existing asset gaps confirmed by the scene ledger

- **Road decisions: 35 variants** still use a historical fallback: 24 need the stated location and 11 need an essential visual gag. These are variants, not automatically 35 new images. Four further road variants are explicit held fallbacks: ENC-C02-A, ENC-C04-A, ENC-C07-C, ENC-C08-A.
- **Ally/community offers: 12 held art mismatches**; each requires a correct retained scene or a review hold.
- **Care: 9 integrated setting backgrounds still lack a credibly scaled in-scene affected traveler; 15 other care scenes/outcomes remain. CARE-07-A is rejected and excluded.**
- **Crossings: 4 compositions** remain after retained integration: CROSS-01-A, CROSS-01-C, CROSS-02C-C, CROSS-03-A.
- **Generated panels on hold: 3 variants** (ENC-S17-B, ENC-S18-A, TOWN-01-A). These failed visual review and cannot be silently bound.
- **Town geography: 51 stops** need a per-stop recognizability audit; the current text-profile inventory does not prove their artwork is correct.

The localization audit found 20 locale files. English has 3,904 leaf keys; the other 19 currently omit between 1,596 and 2,355 of those keys, with additional long strings still identical to English. Existing English fallback is a functional safeguard, not a completed translation. This is a measured content-production gap and must remain open in acceptance reports.

Billboard sizing basis: 106 legs across six routes average 173.2 miles. At one billboard per 60-mile travel step, the average is 3.34 billboards per leg; three times that mean rounds up to 11 distinct messages. The retained pool has 12 messages already translated across the 20 locale files. The longest leg can show ten signs, so the full pool cycles without repetition during a normal leg.

Implemented in the recovery branch: one run-seeded sign per driving step, one viewport crossing per step, unchanged image and copy during that crossing, with the same image/headline/copy on phone. All 12 messages have three nonempty fields in every locale file. Browser fit still needs rechecking after the local browser executable is restored; field presence alone does not validate translation quality or layout.

### Road location variants

- `ENC-C14-B` — Outdoor repair fair and first-aid chair
- `ENC-D02-B` — Indoor school gym equipment sale
- `ENC-D02-C` — Nursing-home rummage sale
- `ENC-D08-A` — Pregnancy-center open house with advertising display
- `ENC-D08-B` — Union-hall table sale
- `ENC-D10-B` — School-supply sale
- `ENC-S01-C` — Veterans hall supper
- `ENC-S04-C` — Marina visitor area
- `ENC-S06-C` — Roadside letter stall
- `ENC-S08-C` — Souvenir printing stall beside food counter
- `ENC-S10-B` — Mountain-photograph souvenir stall
- `ENC-S12-A` — Grocery checkout and crates
- `ENC-S12-B` — Discount-store loading bay
- `ENC-S12-C` — Catering depot
- `ENC-S13-B` — Food-and-water shop checkout
- `ENC-S13-C` — Food-and-water shop and lost property
- `ENC-S14-B` — Clinic fundraiser with chairs
- `ENC-S18-B` — Souvenir printer and postcard racks
- `ENC-S19-C` — Event printer and disclosure inserts
- `ENC-S26-A` — Transport supporters leaflet stall
- `ENC-S28-A` — House open-house cleanup
- `ENC-S30-A` — Coastal transport meeting
- `ENC-S32-A` — Outdoor turbine repair stall
- `ENC-S34-C` — Baby-goods warehouse

### Road essential-gag variants

`ENC-C16-C`, `ENC-C18-B`, `ENC-D06-B`, `ENC-D09-A`, `ENC-S03-B`, `ENC-S05-A`, `ENC-S07-C`, `ENC-S09-A`, `ENC-S09-C`, `ENC-S10-A`, `ENC-S17-B`

### Ally/community mismatches

- `ALLY-01-B` — Straw-hat stall replaces plain hat cartons, customs bill and crew phone request.
- `ALLY-01-C` — Route map and lunch replace enormous donor microphone and disconnected small microphone.
- `ALLY-02-B` — Opposing arrow sheets replace disconnected headset and packed belongings.
- `ALLY-03-A` — Bank/mailbox lobby replaces required garage invoice, apron and overnight-shift alarm.
- `ALLY-03-B` — Cash-register scene lacks the medical bill under a second-job timecard.
- `ALLY-03-C` — Apron is present but groceries look abundant, contradicting sparse food budget. Preserve identity; revise bag/second-job context.
- `ALLY-04-A` — Gaming classroom replaces rival-color merchandise tables and paired card readers.
- `ALLY-04-C` — Miniature objects replace laptop, monthly calendar and theatrical countdown.
- `ALLY-05-C` — Football trophy and award replace discarded receipt and blocked phone message.
- `ALLY-06-A` — Industrial shoulder injury replaces injured ankle, closed ramp and privileged entry.
- `ALLY-06-B` — Machine-shop consultation replaces cleaning equipment and empty service desk.
- `ALLY-06-C` — Sore back and ice pack replace sore shoulder, speaker handcart and distant donor stagehands.

### Crossing compositions

- `CROSS-01-A` — Officer inspecting open hood and ordinary alternator of actual stopped van; no dismantling or damage
- `CROSS-01-C` — Officer at actual van manufacturer plate; actual waiting travelers and disproportionately tall blank forms stack
- `CROSS-02C-C` — Steel quote, solitary cone and unfinished repair area with actual van safely on approach
- `CROSS-03-A` — Official looking into actual van; cash tray and travelers case folder visible

## Acceptance rules

- No embedded text or religious symbols in generated or retained art. HTML/CSS supplies all readable signage and is localized.
- Keep 768×512 selected atlas cells at native 3:2 proportions and the approved camera/perspective rules in `scene-cell-review.json`; place actual crew from game state, not painted substitute occupants.
- Check each candidate at desktop and phone widths for crop, subjects, foreground occlusion, captions, and reachable actions. A successful build or non-empty image is insufficient.
- The approved 19 image-generation attempts have been used. User review is required for further image-generation scope; no retries are automatic.
- Complete all 20 selected locales across interface and active content, and verify native meaning and fit. Existing English fallback and matching key tests do not meet this finish line.
- Preserve 644 accounted content records and keep 36 mechanics-dependent records inactive.
