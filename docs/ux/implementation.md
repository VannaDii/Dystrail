# Dystopian Trail — playtest revision

The local release preview is at `http://127.0.0.1:8081/play/`. This revision implements the accumulated playtest feedback. It has not been deployed.

## Player experience

- The full **Dystopian Trail** name remains throughout the UX; Dystrail.com remains the domain.
- Outfitting is one workspace with category filters, visible prices and grants, quantity controls, a persistent loadout, remaining cash, capacity warnings, and a review step. Help appears beside the relevant controls. Mobile has a persistent cash/review bar.
- The scene carries the title, region, conditions, and seven-stat HUD: supplies, health, sanity, pants danger, credibility, morale, and allies. Each stat has contextual help.
- Pace and information diet each expose all three choices and their numerical effects. An interrupted day locks them until that day finishes, preventing repeated modifiers.
- Weather has condition icons, scene treatments, readable effect details, protection status, and a notice when an extreme condition begins. A weather contribution row explains how its effects can be offset by the other changes in the turn receipt. Camping remains available from the notice.
- Travel resolves one engine action, then presents a 2.6-second transition before revealing the next decision. A visible progress strip and game clock advance during that interval. Skip resolves the same pending action. Waiting never consumes resources; reduced-motion mode removes scene and weather animations.
- The clock displays the game day and an illustrative time of day: morning before a fresh turn, midday during an interrupted day, and advancing hours during transit. It is not a real-time survival timer.
- Normal travel returns directly to the next playable turn with an inline receipt. Encounters and camp actions retain a dedicated outcome with actual changes. Repair receipts include consumed spares, cash, vehicle condition, and elapsed days, and remain visible even when an encounter immediately follows the repair.
- Inventory separates replacement parts from equipped protection/documents and shows cash, vehicle condition, and mileage. The journal resolves translation keys and legacy log aliases before rendering.
- Entering low sanity opens an animated, dismissible alert. Terminal results use regional imagery, a correctly cropped persona portrait, a prominent score, readable statistic blocks, and native menu buttons.

## Recovery and controls

- `dystrail.autosave.v1` stores the active game, stage, pending onboarding state, logs, aftermath, last receipt, weather notice, and open route store. It is separate from `dystrail.save.default`, the player's manual save slot.
- A resolved action is saved before its travel animation completes. Refreshing during transit resumes at the resolved decision without taking another turn.
- Persona selection, shopping cart, category filter, and outfitting review are restored. Cart drafts are removed after checkout.
- Game saves retain random-stream positions, in-progress day accounting, and encounter rotation state. A regression test compares an uninterrupted journey with its saved/reloaded continuation. Older saves without these additive fields still load with defaults.
- The top Menu contains **Save**, **Load**, a real high-contrast switch, and a styled language listbox. Listbox arrows, Home/End, Escape, and native selection work. Popovers and dialogs dismiss outside their content.
- Number shortcuts work from the page as well as from a focused choice. They ignore text entry, modifier chords, repeated keys, and modal dialogs.
- Screen changes no longer focus the page container or large headings. Loading setup, persona, outfitting, or a recovered game does not add a page-sized focus ring. Deliberate keyboard navigation and dialog focus remain visible.
- The GitHub Pages build now copies the client entry to `404.html`, allowing nested game routes to boot and restore local state. This artifact change is prepared, not deployed.

## Supplies and cooldowns

- Crossing a real town on the selected persona route opens a settlement stop. The stop names match the route map and the geographic caption. Shops use the existing prices, discounts, cart rules, and purchase effects, and preserve the current journey.
- A community exchange costs **3 supplies for 1 spare tire**, once per stop. The used exchange marker survives manual and automatic saves. Travelling away closes that stop; camping does not.
- The browser previously never decremented camp cooldowns; only the simulation harness did. Completed game days now own this operation, guarded against running twice. Rest/forage show remaining completed days and a progress bar scaled to their configured cooldowns.
- The deterministic tester CSV baseline changed intentionally with that cooldown correction. No encounter weights, strategy thresholds, or dependency versions were changed to obtain a passing result.

## Art, crew and route revision

- Seven open-road environments follow actual route mileage, with a normal van animated independently of the background.
- Eleven new encounter-setting illustrations replace the generic road fallback; the retained dairy scene completes explicit coverage of all 35 encounter IDs. Subjects vary among the six personas. The convoy was redrawn in a chunkier cartoon style, and anatomical/seating issues were reviewed.
- Onboarding collects the player's name and a crew name, and pre-fills five editable gender-neutral first names. Full names are allowed. Every edit enters recovery immediately.
- The van composes the empty body with six independent occupants in three rows of two. Dead/departed status removes the relevant occupant and persists through reload. The roster shows those statuses. This adds presence handling for explicit story outcomes; it does not invent random death/departure events or tie them to the unrelated Allies stat.
- The full geographic map scene uses Census boundaries and OSM driving routes, with six distinct persona origins. It appears after travel at five-day checkpoints, towns and regional borders. Continue returns to the pending decision, and recovery preserves the map or its dismissal. No map strip is added to the normal screen. Regional corn-lobby, factory-reset and accountability jokes sit over accurate geography.
- A latent engine bug left `region` unchanged while miles advanced. Travel, rollback and save migration now synchronize the region. This intentionally changes later regional encounter/weather outcomes and the deterministic CSV reference digest; no balance thresholds or encounter weights were weakened.
- Map labels, named stops, regional road assets and encounter geographic captions all use the shared route catalog. See [the stage map](stage-map.md).
- The top Menu contains Language, a Contrast switch, Save and Load. Its stacking context keeps the HUD from intercepting clicks. The language control resets correctly when the menu closes.
- Help is rendered in a viewport-clamped portal and repositions on scroll/resize. The visible keyboard focus style remains; page loads do not auto-focus or jump.

The previous satirical adaptations remain: department renaming, bridge-crew efficiency, family-plan security, dubious milk studies, and tariff replacement costs. User-supplied satire is adapted as fiction rather than asserted as factual reporting.

All locale catalogs retain matching keys. Crew copy retains translations across all 20 shipped locales. New geographic map copy is translated in English, Italian, Spanish and Arabic, with English fallback in the remaining catalogs. Earlier playtest additions have English/Italian/Spanish/Arabic translations with English fallback in the other catalogs. RTL and locale persistence are browser-tested.

## Validation

Final commands run:

- `just fmt`
- `just lint` — formatting, workspace check, strict Clippy, locked workspace tests, headless Chrome WASM tests, and engine coverage pass. Coverage: **78.82%**, 3,248 / 4,121 lines.
- `just tests` — workspace and localization tests pass, including cooldown, trade accounting, repair receipt, and deterministic recovery regressions.
- `NO_COLOR=true just build-release` — release WASM and native workspace builds pass.
- `PLAYTEST_PORT=8081 PLAYTEST_CHANNEL=chrome npm --prefix dystrail-web run test:e2e -- --trace off` — **30 tests pass**, fifteen scenarios on desktop and mobile Chrome. They cover focus, keyboard selection, menus, locale/RTL, reduced motion, onboarding/cart/review recovery, pending transit recovery, encounter/aftermath/result recovery, repair costs, regional weather scenes, supply purchases, barter limits, cooldown completion, player/crew naming, absent occupants, periodic full map scenes, six persona origins, map recovery, Arabic map layout, town/encounter geography, viewport help placement, heat shimmer, and clock/scene synchronization.
- `git diff --check` — passes.
- `just security` and `cargo audit --file Cargo.lock --deny warnings` — fail on the existing dependency advisories. The real lockfile audit reports seven vulnerabilities and seven denied warnings. The recipe's audit step still incorrectly treats `audit.toml` as a lockfile; the subsequent `cargo deny` step detects advisories. No exceptions or dependency updates were introduced.
- `just qa` — the 1,000-iteration real-game sweep stops at Deep/Aggressive seed 1337, iteration 273: its risk expectation requires accumulating pants danger, but observes 2. The earlier playtest also had an unrelated encounter-diversity gate failure. These remain release blockers rather than reasons to weaken the checks.

The local preview server supports nested-route fallback. Browser tests use the installed Chrome channel and two workers. Screenshots are captured from the release build; imported states deliberately exercise rare situations and do not establish how often those situations occur naturally.

## Visual review

New art, names and geographic alignment:

- [Full U.S. map scene](screenshots/r4-map-us-chromium.png) · [Route detail on mobile](screenshots/r4-map-route-mobile.png)
- [Appalachian approach](screenshots/r4-open-appalachian-ridge-chromium.png) · [Arabic map](screenshots/r4-map-rtl-mobile.png)
- [Media Coach Onboarding](screenshots/r3-classic_media_training-chromium.png) · [Mutual aid](screenshots/r3-classic_mutual_aid-chromium.png)
- [Crew naming](screenshots/r3-crew-names-mobile.png) · [Absent crew and empty seats](screenshots/r3-crew-absences-chromium.png)
- [Unclipped weather help](<screenshots/r3-help-Weather effects-mobile.png>) · [Heat Wave](screenshots/r3-heat-wave-chromium.png)
- [Asset inventory and checksums](art-assets.md)

Earlier revision references:

- [Outfitting, desktop](screenshots/outfitting-chromium.png) · [mobile](screenshots/outfitting-mobile.png)
- [Travel and HUD, desktop](screenshots/travel-chromium.png) · [mobile](screenshots/travel-mobile.png)
- [Cold Snap](screenshots/cold-chromium.png)
- [Repair receipt before the next choice](screenshots/repair-chromium.png)
- [Route store and community exchange](screenshots/route-stop-chromium.png)
- [Low-sanity alert](screenshots/danger-chromium.png)
- [Camp](screenshots/camp-mobile.png)
- [Results](screenshots/result-mobile.png)
- [Arabic setup](screenshots/rtl-mobile.png)

Geographic build verification: `python3 scripts/build_geography.py` reproduces the local Census SVG and six route catalogs from retained inputs. `cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions` also passes. An initial tests/QA attempt overlapped coverage cleanup and lost temporary Cargo build artifacts; both commands were rerun after coverage completed. The later `just tests` passed; `just qa` reached its existing risk-expectation failure described above.
