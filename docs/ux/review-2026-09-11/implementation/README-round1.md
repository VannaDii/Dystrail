# Dystopian Trail continuity update

Implemented the approved F01–F12 review, including the subsequent route-only map and abandon-trail requests. Local implementation; no production deployment or pull request.

## What changes in play

| Review | Implemented behavior |
| --- | --- |
| F01 · Travel rhythm | Resume carries the van through quiet actions. Normal, Fast and Step controls; meaningful decisions, towns, danger and weather interrupt. Visibility loss pauses further travel. Reload resolves an initiated action once and resumes paused. |
| F02 · Stage entry | Stage changes return to the scene without forced heading focus. Compact scenes and controls, with secondary information behind explicit tabs. |
| F03 · Consequences | One persisted journal stores actual before/after stats, resource changes, place and time. Last action comes from that history; rest, repairs, purchases, trades, encounters and the ending survive save/load. |
| F04 · Preparation | Forecasts account for stat caps. Checkout prevents supply overflow. Next-town distance, capacity, vehicle condition, money and recent supply use provide preparation context. |
| F05 · Time and weather | Free navigation spends no time. Actions advance a persisted clock. Recovery days remain stationary. Heat, smoke and storm consume supplies; weather alerts show modifiers and offer continuation or camp. |
| F06 · Towns | Explicit arrival scene and departure, shop, varying atomic exchanges, local advice and next-leg context. Transactions keep the town location and record costs. |
| F07 · Crew | Named fatigue/illness, care, recovery, safe departure and explicitly warned fatal decisions persist. Active-roster casting excludes absent people; six seats stay arranged as three rows of two. |
| F08 · Persona | Readable origin contrast and clearer practical benefit versus score multiplier. Persona-specific origins retained. |
| F09 · Information | Survival resources and social influence have distinct HUD treatment. Optional condition, inventory and journal views retain contextual help. |
| F10 · Route | Accurate road geometry; map appears as a full scene at safe intervals. It always frames the selected route. No overview/zoom toggle. Current position, trace, town services, region and encounter geography share route data. |
| F11 · Purpose and ending | D.C. objective appears during setup; final-vote outlook explains contributing resources. Endings lead with the named crew and their turning points, with score details secondary. |
| F12 · Visual continuity | Eighteen new crew-free scene settings, independently selected active portraits, named scene subjects and proportion-preserving atlas rendering. Normal-size dynamic van retained. |

## Abandon trail

The top menu now offers **Abandon trail** during a run. An in-game confirmation defaults focus to **Keep traveling**; Escape and clicking outside cancel. Confirming ends and autosaves the journey, preserves the crew and journal, and shows a distinct ending. The separate manual save remains available to load. Reload does not revive the abandoned run.

## Verification

- Rust workspace tests: **226 passed**.
- Clippy with the repository's strict lint groups: passed.
- Formatting and whitespace checks: passed.
- Native release build and browser release build: passed.
- Game coverage command: **78.24%**, above the required 77%.
- Browser verification covers desktop and Pixel 7, keyboard controls, Arabic layout, reduced motion, tooltips, all persona routes, scene associations, named absences, cap-aware outcomes, repair costs, town transactions, clock/rest, autosave, manual save and abandoning. There were **23 passing desktop cases and 23 passing phone cases**. An earlier combined runner stalled during browser shutdown after the desktop assertions passed. After the final HUD and stage-entry adjustments, **all 23 phone cases passed cleanly** in two batches: 6 cases in 9.2 seconds and 17 cases in 46.0 seconds. Exact results and limits are recorded in [validation.json](validation.json).
- Visibility-loss test dispatches a controlled browser visibility event in headless Chromium; it verifies the handler and prevents subsequent actions. This is not a separate operating-system background-throttling certification.
- `wasm-pack test --headless --chrome --mode no-install dystrail-web` completed, but this repository discovers **no wasm-bindgen tests**. The browser release is exercised by Playwright.

### Checks that are still flagged

- Auditing the actual `Cargo.lock` reports **7 vulnerabilities and 7 denied warnings**. Dependency policy checking fails advisories; licenses, bans and sources pass. The lockfile is unchanged by this UX work. The existing `just` audit recipe incorrectly passes `audit.toml` as a lockfile and scans zero dependencies; the verification here instead used `cargo audit --file Cargo.lock --deny warnings`.
- The 1,000-iteration real-game simulation fails the **Classic / Aggressive** expectation at **seed 1337, iteration 273**: final Pants danger is **2**, below that scenario's risk expectation. The balance criterion was not weakened to make this pass. This UX pass is not a balance certification.
- Main new interface copy is translated in English, Italian, Spanish and Arabic. Remaining locale files retain readable English fallbacks for new strings; key and interpolation coverage checks pass.

The deterministic tester CSV fixture changed intentionally with stationary recovery and the corrected weather supply sign. Its updated digest is covered by the native suite.

## Evidence and assets

Browser work used the isolated **8083** origin and disposable test profiles. The user's **8081** local storage was not read or replaced. Existing uncommitted workspace work was preserved.

The screenshot gallery contains 18 direct browser captures of this implementation; the final layout captures pause animation for a readable still image. Controlled imports establish rendering and decision recovery, not the natural frequency of incidents. See [art manifest](art-manifest.json) for source atlases, cell associations, hashes and rejected candidates. Generated full-body sheets with painted transparency were rejected; existing clean transparent persona art remains in use.
