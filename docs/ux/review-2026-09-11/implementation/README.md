# Dystopian Trail · Implementation review

**Current preview:** http://127.0.0.1:8180/play/. Latest changes and verification: [Loading, HUD and journal](polish/README.md) · [Campaign continuity](campaign/README.md). [Complete feedback and work order](feedback.md) · [Full content inventory](content/) · [Current browser gallery](index.html).

The notes below preserve the earlier second-playthrough evidence. Their build, counts and limitations have been superseded by the linked current report.

## Historical second-playthrough update

Playable locally at http://127.0.0.1:8081/play/. [Full encounter and town inventory](content/) · [Browser evidence](index.html) · [Verification data](validation.json).

## What changed

- **Outfitting:** visible paid starter cart ($71 before discounts, 16 supplies), all quantities editable, including an empty van. No hidden base supplies. Capacity enforced before payment. Preparation guidance lives in contextual help.
- **Names and language:** the selected character’s name drives early endings and journal entries. Removed the start-screen step footer and player-facing “persona” labels. Crew portraits retain their proportions in light frames.
- **Scene presentation:** shorter top HUD; bottom scene strip for the next stop, services, money and vehicle condition. Service icons open explanations. Portrait labels match their frames. Travel keeps the road and progress elements mounted and scrolling between quiet actions; the van sits on the road. The normal van still contains three rows of two active occupants.
- **Consequences:** compact journal receipts replace the separate Last Action block. Repeated When/where rows are omitted. Costs are checked before selection, cash rewards actually pay, and activity time rolls correctly across midnight.
- **Gathering and work:** foraging or farm work outside town; barter, food-for-work and paid unloading in town. Their costs, limits and cooldowns persist.
- **Breakdowns:** named Tire, Battery, Alternator and Fuel Pump replacements. Choose an onboard spare, a priced purchase, supply barter, or local-radio work exchange. All alternatives stay visible with their costs; unavailable ones are disabled. Cashless players retain a time/sanity/morale-costed route onward. The browser no longer silently consumes an available spare during travel.
- **Crew care:** eight specific fictional political-satire reasons, each with a factual hook. Choices identify costs, departure and critical consequences. Lost or departed crew stay absent.
- **Satire:** 24 new regional events bring the bank to **59 encounters and 169 choices**. Every event has a recognizable Trump-administration hook and a source available through contextual help. Eligible unseen encounters take priority before repetition. The larger choice bank includes cash rewards and larger Pants swings.
- **Towns:** 28 exact-location facts, with player and random NPC portraits, a saved conversation, and fictional political commentary. Chicago includes a documented local administration impact. Other towns pair sourced local facts with satire; the inventory explicitly identifies the opportunity for more town-specific policy reporting.

Political satire does not need neutral wording or artificial balance. Earlier statements that partisan satire could not be written were incorrect. The implemented distinction is between sourced factual claims and invented comic situations, dialogue and characters.

## Offline and installation

This is now an installable **PWA**. The existing single-page game remains the application; no separate native client is required.

The first visit downloads the full game: 80 files, **75.7 MB**, including all scenes, NPCs, maps, scripts and data. The Menu shows download progress and **Ready for offline play**. Let this complete before disconnecting. Game state continues to autosave locally. External source links are optional reading; they are not needed to play.

On an online launch, a changed build is downloaded and verified before gameplay starts. The new worker activates only after the entire bundle is complete. An interrupted update keeps the previous version playable and preserves the run. A bounded connection check lets a cached game launch when the server cannot be reached.

Chrome exposes its install prompt when available. Safari instructions point to Add to Dock on macOS or Share → Add to Home Screen on iPhone/iPad. The manifest, Apple touch icon and standalone metadata are shipped. HTTPS is required on a deployed origin; localhost is valid for development. Browser site-data clearing removes both the cached game and its saves.

## Verified browsers

| Browser | Evidence |
| --- | --- |
| **Chrome 152.0.7977.84** | Current desktop assertions pass across the 32-case suite in focused batches. Includes real installability checks in a persistent profile, all assets offline, closed-tab relaunch, update-before-start and failed-update fallback. |
| **Safari 26.6.2 on macOS** | **17 checks pass with exit 0** through Apple’s native WebDriver. Onboarding/names, paid editable cart, menu/help, Arabic/reload, route map, heat/cold, foraging, cashless repair, numbered choices/cash, exact-town conversation, offline assets, safe updates and abandonment. |
| **Phone-size Chrome** | 21 current-round assertions cover gameplay, continuity, narrow layout, Arabic, weather, tooltips, pointer/focus and recovery. This is Pixel 7 emulation, not a physical phone. |

The real Safari offline test shut down its disposable server completely, then reloaded the saved game and read every bundled asset successfully. It restarted the server with changed content, confirmed that the launch gate held, and finally interrupted another update to verify fallback. Chrome also closed the original page and opened a fresh page while disconnected.

Some Chrome Playwright batches still hang during final browser shutdown **after all test assertions pass**; these are recorded as runner errors, not clean exits. A focused nine-case PWA/gameplay batch completed cleanly in 18.9 seconds, and the final eight-case Chrome continuity batch completed cleanly in 31.3 seconds. No physical iPhone/Android installation or standalone OS launch was performed. Safari screenshots are from actual desktop Safari, not Playwright WebKit.

## Other checks and release limits

- **234 Rust tests passed**, strict Clippy passed, formatting and whitespace checks passed.
- **78.16% coverage**, above the 77% requirement; native and Trunk release builds passed.
- `wasm-pack` completed using installed tooling, but discovers **zero wasm-bindgen tests**. Browser behavior is covered by the Chrome and Safari runs above.
- The actual unchanged lockfile still reports **7 vulnerabilities and 7 denied warnings**; cargo-deny advisories fail while licenses, bans and sources pass. The existing `just security` recipe incorrectly uses `audit.toml` as a lockfile, so verification used `Cargo.lock` directly.
- The broader 1,000-run campaign check still fails Classic/Aggressive seed 1337, iteration 273, at Pants 2. Its acceptance threshold was not weakened. This local UX build is not a production or balance certification.
- New UI, event text, care reasons, town facts and source explanations support English, Italian, Spanish and Arabic. The other locale files retain English fallback for new keys. Existing journal prose stays in the language in which the action was recorded.
- 59 encounters share **12 setting compositions**, and towns share three regional backdrops. These are explicit mappings, not claims of 59 bespoke illustrations. See [Scenes & gaps](content/) for art and editorial follow-up opportunities.

## Reproduce

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo test --workspace --all --all-features --locked -- --nocapture
cargo tarpaulin --packages dystrail-game --fail-under 77 --target-dir /tmp/dystrail-coverage-current
cargo audit --file Cargo.lock --deny warnings
cargo deny check licenses bans advisories sources
wasm-pack test --headless --chrome --mode no-install dystrail-web
cargo build --workspace --release
cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console
python3 scripts/build_content_review.py
cd dystrail-web
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-continuity-preview/play
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test --project=chromium --workers=1
/usr/bin/safaridriver --port 9515
# In another terminal, with Safari Remote Automation available:
node tests-e2e/safari-smoke.mjs
```

The Safari harness owns a disposable server and closes only its own automation session. Test fixtures are imported through the game’s Load interface. The player’s 8081 localStorage was never read or changed. The tested build was copied to the local game server and its HTML, worker, manifest and NPC atlas verified byte-for-byte. No production deployment, branch, commit or PR was made.

Earlier evidence is preserved in [the first-round notes](README-round1.md). Current logs are under [logs/](logs/); the structured data records exact scope and limits.

Latest update: [automatic weather impacts and compact travel controls](weather/README.md), with [screenshots](weather/index.html). Current game preview is http://127.0.0.1:8180/play/.

Latest menu repair: [offline indicators and installation controls](offline-menu/README.md), with [screenshots](offline-menu/index.html). Verified in Chrome and Safari; preview remains on port 8180.
