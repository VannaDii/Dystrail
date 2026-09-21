**Feature: Visual world and satire integration**

Prepared September 14, 2026. This is a separately scoped implementation feature extracted from the enhancement plan. It covers art production and the presentation/content support needed to deliver the latest compatible satire. This task created the brief and coverage inventory; artwork and gameplay implementation have not begun here.

**Outcome**

The game presents a cohesive, more racially diverse cast whose characters are gender neutral or feminine; convincing van seating; scene art matched to the actual satire and game state; readable roadside advertisements; and lighting that communicates time and weather. The latest compatible A/B/C satire variants can be selected, illustrated, localized, saved, and replayed correctly.

This feature owns ideas **1, 8, 13, and 14**, the qualitative presentation work from **2**, and the graphical announcement work from **10**. It supplies shared art and content bindings to the hearing feature. Changes to hearing probability, PPE quantities/wear, hunger balance, daily planning rules, remembered names, URL intake, and general travel-pause policy retain their existing separate scope.

**Verified source baseline and content boundary**

The current workspace is at `f1ed4fe` and contains unrelated in-progress hearing, engine, localization, and scene changes. A separate satire source checkout is at `c56bc90`, following `30652f1`, which imported the reviewed A-version content into existing game flows. The feature must start from a reconciled source baseline that includes that content work and the relevant completed hearing work. Do not overwrite the current checkout with the other checkout or copy entire locale files over newer changes.

The captured workshop pack has 644 records. The source release selected 177 narrative packages and preserved 65 existing road families with 187 ordered choices. Its release records document English, Spanish, Italian, and Arabic coverage and English narrative fallback for other offered locales. The source release deliberately deferred B/C variants, new encounter mechanics, and artwork.

| Content group | Records | Feature disposition |
|---|---:|---|
| Existing-compatible narrative families, A/B/C | 531 | In scope: retain the 177 integrated A packages and enable their compatible B/C variants with matching presentation. |
| Factual and functional records | 77 | In scope for consistency, asset/reference mapping, and any necessary presentation corrections; these do not imply 77 new scenes. |
| Twelve proposed western encounter families, A/B/C | 36 | Tracked dependency: their new choices/effects require a separate encounter-mechanics feature before activation. Preserve the written records and art briefs. |
| **Total accounted for** | **644** | Every source record has an explicit disposition. |

The [coverage inventory](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/features/art-and-satire/satire-art-coverage.csv) contains all 644 IDs, family/variant relationships, runtime keys, scene directions, overlay counts, and initial art-mapping status. The [baseline record](/Users/vanna/Source/Dystrail/docs/ux/enhancement-review-2026-09-13/features/art-and-satire/baseline.json) pins the inspected source revision and file hashes. This was a local source review, not a fresh live-document or production verification. Refresh the native workshop revision before implementation and reconcile any changes against this baseline.

Use the latest structured scene directions, not just older scene names or generic encounter categories. The older brief's executive-order labels already have explicit mappings in the refreshed source and content bindings: Militarize → TravelBanLite, Gag → BookPanic, Tariffs → TariffTsunami, TaxCuts → DoEEliminated, Deregulate → WarDeptReorg. Preserve those mappings.

**ART-01 — Establish the cast and visual direction**

Create six canonical crew designs, retaining their role identities while making each gender neutral or feminine and improving racial diversity across the group. Review fictional townspeople, medics, workers, outside contacts, and hearing officials against the same direction. Write down intended presentation and pronoun usage; do not infer pronouns from player-entered names or assign mechanical traits by race.

Use a consistent pixel-art scale, palette, outline, lighting, and silhouette language. Preserve the recognizable blue van unless a later visual review explicitly changes it. Record a small set of approved reference views for each person before producing the full asset set.

Deliverables: six character sheets; six selection portraits; six standing poses; twelve proposed near/far seated poses; a roster of supporting-character designs; and a list of every dependent appearance. Reuse approved views where appropriate across naming, care, encounters, hearing, results, and shared images. Audit homepage art and screenshots for obsolete versions.

**ART-02 — Rebuild seating and character composition**

Design the van interior and occupants together. Establish six understandable seat positions, seatbacks, window masks, and distinct head/shoulder silhouettes. Avoid the current stacked-bust composition. Include a natural driving pose and preserve replacement-driver behavior when the original driver is absent.

Crew members remain independent transparent layers. Show only active travelers, preserve user names and role identity, and keep outside allies distinct from van passengers. Use real alpha transparency rather than relying on a browser-specific color-key effect. Define near/far placement and clipping metadata so a new portrait does not need ad hoc positioning in each screen.

Acceptance examples: full crew, absent near/far partners, replacement driver, a single remaining traveler, and a small mobile view. No scene may resurrect a departed character or invent an additional traveler.

**ART-03 — Produce scenes from the satire's staging requirements**

Build a reusable scene catalog for the six travel regions, towns, roadside services, camp/care, crossings, activities, policy announcements, the hearing, and ending types. Map each supported narrative variant to a setting, props, local characters, crew placement, editable text overlays, and valid display state. Count unique production assets after this mapping; 644 editorial scene directions are not 644 required full-screen paintings.

Reuse settings where the depicted situation remains accurate. Use distinct focal props, actions, NPCs, and compositions when different variants need them. A visual gag must support the actual premise. For example, an offer of gloves at a park requires the ranger, glove box, and neglected picnic area described in that variant; a generic civic-office picture does not meet the scene brief.

Separate pre-choice staging from completed outcomes. Before a decision, the crew observes an offer: it has not already paid, accepted food, repaired the van, worked, or departed. Outcome art and text follow the completed engine result, including failed affordability checks, critical care, living departures, deaths, and arrival status. Endings must not depict D.C. or a hearing when the run ended earlier.

Keep written surfaces blank in generated artwork. Render sign wording, captions, quantities, and other required words through editable localized overlays. Preserve factual source hooks and qualifications separately from fictional speech.

**ART-04 — Integrate time, weather, and roadside billboards**

Extend the existing clock-driven lighting layer with approved early-morning, midday, late-afternoon, dusk, and night treatments. Review matching moving/stopped scenes across sky, shadows, van, crew, and signs. Add selective masks or limited alternate artwork where tinting a painted daytime scene cannot produce a convincing result. Use separate local-light layers for headlights, lamps, or illuminated signs where appropriate.

Combine time and weather without dimming controls or obscuring characters and messages. Scene lighting follows represented simulation time; pausing or waiting at a menu does not advance it. Retain timestamps through saved event/hearing reports and overnight transitions. The current 08:00–13:00 driving window is a separate rule; do not fabricate sunset at 13:00.

Initial billboard production target: twelve short satirical advertisements, two associated with each region, and two reusable sign structures. Treat these as a distinct small writing/art package alongside the existing workshop. Keep copy sharp, readable, and independently editable. Billboards live on their own roadside layer, so mirrored landscapes never reverse their lettering. They must fit both moving and stopped compositions without covering the van, crew, HUD, or choices. Provide a readable paused presentation and accessible text equivalent.

Billboard and decorative selection must not consume simulation randomness. Save or deterministically recover the selected sign and maintain it across pause/reload. Changing a sign cannot alter encounter availability, rewards, or hearing odds.

**CONTENT-01 — Bind compatible variants to scenes and outcomes**

Extend the existing workshop presentation bindings rather than replacing the engine or reimporting older copy. Add stable editorial variant IDs and scene IDs, validate their relationship to runtime families, and retain ordered choices and effects. Source records for the 195 road variants mapped to existing families were audited without mechanical differences; recheck that relationship against the implementation baseline before activation.

Enable A/B/C selection for compatible road, town, care, outside-contact, and supporting families. Eligibility and actual state come before variety. Keep an ongoing care incident on its selected variant. Persist the selected variant and content version in a backward-compatible presentation record so reload/import does not rewrite an unresolved situation.

For this bounded feature, use deterministic presentation selection from the run identity and occurrence, separate from the simulation random stream. Existing bare-code replay must not silently depend on a player's previous-content history. Cross-run novelty history and an 80%-unseen guarantee remain later selection work unless a compatible replay contract is explicitly included. Measure variant coverage and repetition; do not claim the larger novelty target from inventory size alone.

Review unsupported or mismatched prose before binding it. Preserve known fidelity corrections from the content release: permits do not consume a Receipt in the current implementation; roadside purchase and town service use different outcomes; the cashless repair path remains available; leaving a living companion is distinct from death. The source legal-fund copy must not promise a protection tag the purchased item does not grant.

**CONTENT-02 — Make the new satire readable in the game**

Apply qualitative consequence subtitles consistently across decision surfaces, including their tooltips and accessible descriptions. Preserve exact prices, offers, actual quantities, affordability, duration, and completed results where those facts define the action. Do not hide a critical risk or promise a capped gain.

Use a reusable graphical announcement for newly activated major policy events, with the matching satire and current effect readout. Present each activation once, retain its effect in the existing status bar, and resume or dispatch the next genuine pending action. This is presentation of an existing event, not a new event probability or effect. Persist acknowledgement so reload cannot repeat a grant, penalty, or forced announcement. Ordinary ambient flavor remains on existing nonblocking surfaces.

Provide matching assets and copy bindings for the staged hearing, but consume the hearing feature's actual report. The older workshop copy assumes three stamina rounds and one vote; adapt it to the implemented one/two/three-round, exhaustion, automatic-victory, and final-vote states. Do not show a second or third round that never occurred, or announce a vote after an automatic result. Hearing probabilities, continuation tuning, and sanity costs stay owned by the hearing feature.

Retain the English/Spanish/Italian/Arabic narrative coverage already established by the content release. Translate newly activated B/C text, billboards, revised hearing lines, and overlay copy in those languages. Preserve other offered locale interfaces and the deliberate English narrative fallback. Validate placeholders, text expansion, Arabic layout, source links, and name/pronoun continuity. Update all offered locales for any new functional controls they expose.

**DELIVERY-01 — Package, integrate, and verify**

Use an isolated implementation checkout from a reconciled source baseline. Shared touchpoints include scene rendering, styles, workshop bindings, locale files, recovery records, hearing pages, and result presentation. Integrate finished changes deliberately; do not replace another feature's newer versions with the content release's older files.

Preserve stable role/runtime IDs and old-save recovery. Version the asset catalog and content bindings; update offline precaching, preload references, and cache invalidation together. Retain assets needed by existing saved installations during update. Refresh share-image composition and homepage screenshots after the game assets are integrated.

Deliver the feature in five reviewable slices:

1. **Source and catalog:** reconcile the content/hearing baseline; freeze source revision; map all compatible units to scenes and define the cast, overlay, and persistence formats.
2. **Cast and van:** produce and integrate the six crew designs, seating, standing poses, and supporting-character coverage.
3. **Scenes and lighting:** implement the reusable scene catalog and time/weather treatments, validating staging before producing the remaining variations.
4. **Satire presentation:** add billboards, compatible A/B/C bindings, qualitative subtitles, major announcements, hearing copy adapters, and localization.
5. **Release candidate:** complete visual, state, content, replay, performance, and offline verification; update derived/share/homepage assets and prepare the reviewable release.

**Acceptance criteria**

- All 531 compatible narrative records and 77 reference/functional records have an explicit runtime/asset disposition; the 36 proposed-mechanics records remain clearly tracked and inactive.
- Every active scene matches the chosen satire variant, represented time, location, weather, and current cast. Pre-choice and completed-action staging agree with actual state.
- All crew and fictional supporting-character designs satisfy the gender-neutral/feminine and diversity brief. Seating is legible without awkward portrait overlap.
- Required lettering is editable and localized. Billboards remain readable and correctly oriented in moving/stopped views.
- Visual changes and prose variants preserve mechanical outcomes for the same seeds and action sequences. No gameplay RNG is consumed for decoration or variant choice.
- Variant selection, scene state, and announcement acknowledgement survive reload, save/import, and offline updates. Existing saves receive a documented fallback.
- English, Spanish, Italian, and Arabic narrative/overlay coverage is complete for activated content; other offered locales retain their interface and intentional fallback behavior.
- The new hearing's actual report controls displayed rounds and outcomes. Shared changes do not overwrite its probability or sanity logic.
- Targeted browser checks cover mobile/desktop, Arabic, text enlargement, reduced motion, keyboard focus, crew absence, care/repair outcomes, billboards, time/weather combinations, and restored states. Engine comparisons verify unchanged mechanics; a bounded route/mode sweep measures content coverage and repetition.
- The asset catalog records accepted artwork, reusable layers, dependencies, and source-unit mappings. No release claim rests solely on file counts or a draft marked complete.

The feature is ready for implementation planning from this brief. Its asset count and production estimate should follow the scene-catalog pass, which distinguishes genuinely new illustrations from reusable settings, props, characters, and overlays.
