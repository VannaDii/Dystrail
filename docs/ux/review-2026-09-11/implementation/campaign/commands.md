# Commands and evidence

From the repository root unless noted:

```sh
cargo test -p dystrail-tester --locked
cargo test -p dystrail-game late_route_camping_never_moves --locked
cargo fmt --all
cargo fmt --all -- --check
git diff --check
cargo test --workspace --all --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo clippy -p dystrail-web --target wasm32-unknown-unknown --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo build --workspace --release --locked
cargo tarpaulin --packages dystrail-game --fail-under 77 --target-dir /tmp/dystrail-coverage-current
WASM_BINDGEN_BENCH_RESULT=/tmp/dystrail-campaign-final-benchmark.json wasm-pack test --headless --chrome --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver dystrail-web
cargo run --release -p dystrail-tester --locked -- --mode logic --scenarios real-game --iterations 1000 --report csv --output /tmp/dystrail-campaign-final-1000.csv
```

The first new stationary-camp regression failed before the fix. The intentional replay changes each failed the old digest while the independent repeated-run equality passed; the final digest records the corrected behavior. Final native/browser/lint/coverage commands passed. Campaign acceptance still failed; raw measurements and logs are retained.

A 100-iteration debug campaign and a preliminary 1,000-iteration release campaign ran while the tester was reconciled with player behavior. Neither was a pass. The final run above includes the camp correction and actual repair-day tagging. Release optimization changes runtime only; it does not change scenarios or thresholds.

From `dystrail-web`:

```sh
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-campaign-final-web
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/continuity.spec.ts --grep 'late-route camp|stationary rest records' --workers=1
```

The Chrome test ran twice because the new assertion initially addressed an unflattened Rust field instead of its serialized name. The clean final run passed four tests in 13.3 seconds. Node's deprecation and color-environment notices remain visible.

From the repository root, the unmodified skill client ran with its existing temporary loader that resolves local Playwright and selects installed Chrome:

```sh
node --import /tmp/dystrail-game-client-loader.mjs /Users/vanna/.codex/skills/develop-web-game/scripts/web_game_playwright_client.js --url http://127.0.0.1:8083/play/ --click-selector '.retro-btn-primary' --actions-json '{"steps":[{"buttons":[],"frames":90}]}' --iterations 2 --pause-ms 1000 --screenshot-dir output/web-game-camp
```

A six-worker local HTTP read verified each served asset's manifest length and SHA-256. The release was copied to the isolated 8083 preview and then to the requested 8180 preview.

Final native release was repeated after the camp fix and passed. The existing isolated Playwright CLI session reloaded the current build and confirmed two displayed/saved receipts, seven HUD stats and no application errors; its exact action is recorded in `logs/chrome-receipts-reload.log`.

The current native Safari attempt used:

```sh
SAFARI_OUTPUT=/tmp/dystrail-campaign-final-safari PLAYTEST_DIST=/tmp/dystrail-campaign-final-web SAFARI_VISIBLE_WAIT=120000 SAFARI_PORT=56854 node tests-e2e/safari-feedback.mjs
node --check dystrail-web/tests-e2e/safari-feedback.mjs
```

Syntax validation passed. The current runner did not reach gameplay checks: Safari kept reporting a hidden document, and a scoped driver request later stalled. The owned runner was terminated (exit 143), which is not a test pass. The earlier full visibility timeout and current log are both retained. Direct computer-use focus/navigation also did not produce a verified game state. A targeted AppleScript in the earlier attempt had returned `AppleEvent timed out (-1712)`.

The current isolated test preview is left available through `/tmp/dystrail-safari-visible-preview.py` at port 56854, without an active test runner. No user browser save on the primary preview was changed.

Final handoff verification fetched 22 local review/inventory/evidence links successfully and confirmed ports 8180, 8083 and 56854 all serve revision `a24e05514179da6c0b69`; see `review-validation.json`. The owned Safari session cleanup request also timed out after eight seconds. The HTTP service was left intact rather than terminating a system service. Its stale scoped diagnostic request ended with `UND_ERR_HEADERS_TIMEOUT`; neither is treated as an application failure or a Safari pass.
