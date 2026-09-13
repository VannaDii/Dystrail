# Weather and compact travel controls — September 11, 2026

Preview: http://127.0.0.1:8180/play/ · Tested offline build: `2cd4ea64e1665b6196ad`.

Weather changes no longer create a confirm/camp panel, stop automatic travel by themselves, or bypass the normal map/encounter ordering. A compact scene HUD indicator shows the current condition and latest weather-only resource cost. A five-second highlight and polite live announcement notify changes, including the return to clear weather; reduced motion disables the pulse. Detailed help includes ongoing travel/risk modifiers and protection/exposure mechanics. Protected cold still shows its travel penalty when resource loss is zero.

The engine records actual weather costs after gear, exposure and stat limits. Reloading restores that receipt without applying it again. Old checkpoints containing the removed weather pause flag remain readable. The weather receipt also feeds the journal and optional conditions assessment.

Fast is now a saved toggle beside Camp, Review route and Resume travel. Normal is the off state; Step mode is removed, and older Step checkpoints migrate to Normal without advancing the game. Conditions, The Van and Journal occupy the former mode group on the left of the same toolbar (immediately above the action row on phones). Their separate lower row and the detached journey help button are gone. The detail controls open and collapse their panels without spending a turn; the van and journal remain accessible during repairs. The road scene is 60 px taller at the desktop test size and 40 px taller on phones. Genuine encounters, danger, repairs, map reviews and towns retain their normal stops.

## Verification

- **Chrome 152.0.7977.84:** 44 cases passed, exit 0 (22 desktop and 22 Pixel 7 viewport), including additional 320 px layout checks. Fast toggle keyboard operation, saved preference, legacy Step migration, detail-panel switching, stationary rest, pause/visibility recovery, continuous road/progress elements, named crew, repairs, cash rewards and town persistence passed. Real seeded storm costs, continuous travel through the change, no acknowledgment panel, reload without repeated costs, exposure/clear displays, legacy checkpoints, help containment, Arabic, reduced motion, shared action-row geometry and larger scene.
- **Safari 26.6.2:** 21 native WebDriver checks passed, exit 0. Includes the saved Fast toggle, old Step migration, toolbar detail switching and weather behavior, named crew, map, repair/trade actions, numbered choices, full offline asset recovery with the origin stopped, complete updates before launch, and interrupted-update fallback. See `safari-validation.json`.
- **Rust:** 236 tests passed; formatting, workspace/all-target/all-feature check, strict Clippy, native release and Trunk release passed. Engine coverage in the preceding weather pass: 78.25%, above the 77% gate. The toolbar refinement does not change the engine.
- **Localization:** all six new weather HUD keys and four toolbar labels exist in all 20 locales; English, Spanish, Italian and Arabic translations supplied. Other locales retain English fallback copy for these additions.
- **8180:** HTML, direct `/play/persona` route, service worker and offline manifest returned HTTP 200 and matched the tested build bytes. Local player storage was not changed by testing.

## Exact validation commands

```sh
cargo fmt --all
cargo fmt -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo test --workspace --all --all-features --locked -- --nocapture
cargo tarpaulin --packages dystrail-game --fail-under 77
cargo build --workspace --release
# From dystrail-web:
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-continuity-preview/play
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test weather.spec.ts continuity.spec.ts second-pass.spec.ts crew.spec.ts --workers=1 --trace=off --global-timeout=180000 --output=/tmp/dystrail-toolbar-chrome
SAFARI_OUTPUT=/tmp/dystrail-toolbar-safari node tests-e2e/safari-smoke.mjs
# From repository root:
wasm-pack test --headless --chrome dystrail-web
cargo audit --file Cargo.lock --deny warnings
cargo deny check licenses bans advisories sources
cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console
```

The real Safari checks used `/usr/bin/safaridriver --port 9515` and a disposable local server. Earlier reruns could not deliver clicks because the Mac was locked (Safari reported hidden and captured no click events). The user unlocked the Mac; the final full run passed. The test now fails clearly if Safari is hidden. An attempted animation-frame wait was removed because hidden Safari does not advance frames.

## Existing limitations retained

The unchanged dependency audit reports seven vulnerabilities and seven denied warnings; deny advisories fail while licenses, bans and sources pass. The broader campaign check still fails Classic/Aggressive seed 1337, iteration 273, at Pants 2. These gates were not weakened. `wasm-pack` completed but found zero wasm tests, so actual browser checks provide the interaction evidence. The separate bundled game-skill browser client could not resolve its Playwright package; repository Chrome tests and Apple's real Safari driver were used. No physical phone installation was tested.

The actual travel screenshots show engine outcomes. Heat/exposure and Arabic screenshots use explicit display fixtures; their costs are separately verified against the engine by native tests.
