# Journey continuity, scene framing and onboarding — September 11, 2026

Working preview: http://127.0.0.1:8083/play/. Usual preview: http://127.0.0.1:8180/play/.

Current build: `f6b4922d52c336b3eec9`, copied to both previews. All 88 served assets match their size and SHA-256 manifest. [Latest West Coast routes and local-conversation redesign](../west-coast/README.md). [Full feedback ledger and work order](../feedback.md). [Browsable encounter/options inventory](../content/index.html).

## Latest feedback pass

- Removed Pants from the state, rules, choice effects, weather, endings, scores, translations, policies and simulation reports. Legacy save fields are ignored. The old PANTS run-code token still resolves to its original seed through a compatibility alias.
- Local conversations show the available reward before claiming it and strike out the actual claimed reward afterward. At maximum credibility the button says Credibility at maximum; a visit records zero. Conversations stay available and revisits do not award credit or spend time again.
- Ally losses have a persistent acknowledgment screen, actual before/after count, a reason and a journal entry. Automatic maps hold that decision until dismissal. Six fictional political-satire messages cover ordinary attrition; encounter-caused losses retain their own explanation.
- A single small Outcome heading replaces the previous two headings. Cash, condition, evidence and spare parts use the same change cards as the six remaining stats. Cash is labeled Cash and formatted in whole dollars. Discounted store/repair/bribe prices round up to a whole dollar before affordability checks or charging; old balances with leftover cents round up on recovery.
- Journal entries have separate date/location treatment, readable narrative and actual resource badges. The completion notice spans both town-work buttons and is centered beneath them.
- All 51 towns have state/district, sourced 2020 population and a local attraction. Local conversations have separate sourced administration-impact facts and fictional remarks. The store heading and Leave store button align; map attribution sits beside the location in the header. The inventory now includes the six ally messages as well as encounters, towns, care, shops and repairs.

## Current verification

247 Rust tests, strict Clippy, formatting, native/web releases and 79.05% engine coverage passed. Installed Chrome passed the 60-case desktop/phone batch and 24 final targeted checks. Native Safari 26.6.2 passed 11 current-build checks, including the conversation, source context and return to town. Safari ignored window-resize requests and stayed at 1728 CSS pixels; phone-layout claims refer to Chrome, not Safari. English, Spanish, Italian and Arabic conversations were inspected at 1280 and 390 pixels in Chrome.

The current dependency audit reports seven vulnerabilities and seven warnings; deny advisories fail while bans/licenses/sources pass. The current campaign simulation fails Classic - Balanced seed CL-ORANGE02 at a 79.3% travel ratio, below the 90% target. No acceptance thresholds or dependency diagnostics were suppressed. These remain release gates and are recorded in [the latest evidence](../west-coast/README.md); this is not a production or campaign-balance certification.

## Changes

- The launch screen waits for the complete game cache before starting play. A failed or missing asset keeps the loading screen open with Retry; previously complete versions remain playable if an update fails. The cache is rechecked before every launch. Saves are independent of the asset cache.
- Automatic route maps keep travel running and close after five seconds. Stay on map cancels the countdown and pauses travel; Resume closes the map and continues in one action. Manual route reviews remain open. Town arrivals and crew decisions interrupt automatic travel.
- Resource stats sit above the artwork, the clock/weather strip overlays its top, and the turn/next-stop bar sits below the artwork without a gap. Critical stats pulse three times and stay red; the redundant sanity modal is removed.
- The Trail, Conditions, The Van and Journal are exclusive, keyboard-accessible tabs. Generic encounter-ready filler is filtered from travel receipts. Repair resource details are centered.
- Guided foraging and farm gleaning appear as camp cards outside towns with explicit costs and shared cooldown progress. Town services retain paid work and trading.
- Fast, Camp, Review route and Resume have equal compact widths: at most 144 px per control on desktop, with responsive shrinking on phones. Store and trade use distinct SVG illustrations instead of typographic symbols.
- Help & tips defaults on and persists independently of game saves. Turning it off removes optional help buttons while keeping service, weather, source and installation information. Offline readiness is the last menu row, below installation and trail actions.
- The start action shares the run-code row and aligns with The Deep End column; it stacks below the code on mobile. Character selection begins with a random choice, persists across reload, and supports number and arrow keys without focusing anything on entry. The shared mission sits above the selection grid; character information uses a consistent layout and the selected-character summary and Continue button share a centered action row.
- The compact crew grid is preserved. Portrait backgrounds and frames are removed, transparent image padding is reduced through presentation, and the unwanted crew-name explanatory sentence is removed. Existing names and portrait artwork are preserved.
- Endings use the recorded cause, actual route position and primary player identity. A ready-for-hearing flag no longer overrides an earlier loss. Early illness endings use a rest-area scene and the current route location. The former North Platte/Omaha example predates the West Coast routes and is historical. Player death gets its own message; an older saved final journal message is corrected for display. The full scorecard stays open. Loading another ending on the same day refreshes the result.
- Page changes no longer force a scroll to the top. Save/export confirmations use a non-blocking, viewport-visible status message and do not change header height or steal focus. Export success says the file was copied, rather than repeating the action label.

## Verification scope

See `validation.json` and the accompanying browser/native logs for the final build and results. Browsers use disposable origins and contexts; the player's saves on ports 8180 and 8083 are not altered by the tests. The supplied save is imported only into an isolated test context. Screenshots in this folder use synthetic test players.

The browser suite covers the compact layout at desktop, phone and 320 px widths, Arabic/RTL and reduced-motion states, keyboard interactions, camp costs and cooldowns, map timing and interruption, actual repair changes, persistence, full asset caching, offline relaunch, complete update-before-play and failed-update fallback. Native Safari uses Apple's WebDriver and a temporary HTTP origin; the offline test stops that origin completely.

All English translation keys are present in each of the 20 locales. Several locales also retain older unused keys; the key sets are not literally identical. New explanatory copy is translated into English, Spanish, Italian and Arabic with English fallback in other locales; The Trail is translated in all 20.

Physical mobile installation was not performed. The installed game smoke client also ran against the preview. A temporary copy used the repository Playwright dependency and selected installed Chrome because its bundled browser was absent; no other client behavior changed. Current audit and campaign results are described above; earlier results remain in the preceding review as history.

## Commands

```sh
cargo fmt --all
cargo fmt -- --check
cargo test --workspace --all --all-features --locked -- --nocapture
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic -A clippy::multiple-crate-versions
cargo build --workspace --release
cargo audit
cargo deny check
cargo tarpaulin --packages dystrail-game --fail-under 77 --target-dir /tmp/dystrail-coverage-current
cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console
# From dystrail-web:
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-continuity-preview/play
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test scene-flow.spec.ts preload.spec.ts pwa.spec.ts second-pass.spec.ts weather.spec.ts offline-status.spec.ts a11y.spec.ts onboarding-results.spec.ts --workers=2 --trace=off --global-timeout=300000 --output=/tmp/dystrail-final-chrome
SAFARI_OUTPUT=/tmp/dystrail-final-safari node tests-e2e/safari-smoke.mjs
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test feedback.spec.ts ux-round.spec.ts scene-flow.spec.ts --project=chromium --workers=1 --trace=off
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test feedback.spec.ts ux-round.spec.ts scene-flow.spec.ts --project=mobile --workers=1 --trace=off
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test pwa.spec.ts preload.spec.ts onboarding-results.spec.ts --project=chromium --workers=1 --trace=off
SAFARI_OUTPUT=/tmp/dystrail-safari-feedback-final node tests-e2e/safari-feedback.mjs
PLAYER_RESULT_FIXTURE=/tmp/dystrail-player-result-fixture.json PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test onboarding-results.spec.ts --grep 'early illness ending' --workers=1 --trace=off --output=/tmp/dystrail-attached-save-chrome
```

Earlier failed runs are kept separately: test fixtures initially used a different trail distance and the wrong export-button label, and the character keyboard check caught a real focus regression introduced with random selection. The production fix transfers focus only after explicit keyboard selection, preserving stillness on entry.
