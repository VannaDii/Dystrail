# Verification commands

Current build: `84804cf5709353f5093a`, from `/tmp/dystrail-help-stable-web`. Previous complete-cache build: `1868d6c37d8c25729dae`, from `/tmp/dystrail-offline-final-web`. The preceding HUD build was `595e6114615d79492626`. Immutable build directories and campaign evidence are preserved.

From `dystrail-web`, using installed Chrome and the isolated port 8083 preview:

```sh
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/preload.spec.ts tests-e2e/pwa.spec.ts --workers=1 --output=/tmp/dystrail-hud-final-offline-results
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/states.spec.ts -g 'route store' --workers=1 --output=/tmp/dystrail-hud-final-store-results
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test --workers=2 --output=/tmp/dystrail-hud-final-complete-results
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test --project=mobile --workers=1 --output=/tmp/dystrail-hud-final-mobile-results
```

The offline batch passed six cases. The focused store batch passed two cases. The combined suite passed all 67 desktop cases, then stalled for more than four minutes during worker cleanup with no Chrome child processes remaining. Only that owned runner and its two workers were stopped, exit 143. The separate phone project passed 67 tests and exited 0. This is not represented as a clean combined run.

After the packaging correction, from `dystrail-web`:

```sh
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-offline-final-web
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/preload.spec.ts tests-e2e/pwa.spec.ts tests-e2e/feedback.spec.ts --workers=1 --output=/tmp/dystrail-offline-final-browser-results
```

The final package passed 20 desktop/phone checks with exit 0. The unmodified game-client command below was also repeated against the final preview with `output/web-game-offline-final` as its screenshot directory, exit 0. A direct cache audit verified all 87 final files; a separate HTTP audit verified every file on all three preview ports. [Cache result and code](logs/dystrail-offline-final-cache.log), [served-file results](served-assets.json), [final browser log](logs/dystrail-offline-final-browser.log).

From the repository root:

```sh
WASM_BINDGEN_BENCH_RESULT=/tmp/dystrail-hud-final-benchmark.json wasm-pack test --headless --chrome --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver dystrail-web
node --import /tmp/dystrail-game-client-loader.mjs /Users/vanna/.codex/skills/develop-web-game/scripts/web_game_playwright_client.js --url http://127.0.0.1:8083/play/ --click-selector '.retro-btn-primary' --actions-json '{"steps":[{"buttons":[],"frames":90}]}' --iterations 2 --pause-ms 1000 --screenshot-dir output/web-game-hud-final
```

The two registered wasm tests and the unmodified game client exited 0. The existing temporary loader resolves local Playwright and selects installed Chrome. The client screenshot was inspected; separate interaction tests cover actual gameplay.

The existing `dystrail-receipts` Chrome CLI session was used for a 390 px Arabic layout check and the initial Cache Storage audit. Each of the then-88 responses was read from the versioned cache and checked against manifest length and SHA-256. Primary text and subtitles had identical right edges, with the number in a separate column. English was restored. [Initial result and executed code](logs/cache-rtl.log). The final 87-file audit is linked above.

Retained logs: [native tests](logs/native-tests.log), [native Clippy](logs/native-clippy.log), [wasm Clippy](logs/wasm-clippy.log), [native release](logs/native-release.log), [Trunk release](logs/build.log), [wasm tests](logs/wasm-tests.log), [offline/update tests](logs/offline.log), [town departure correction](logs/store.log), [game client](logs/client.log).

Earlier failures remain available: [122-pass/12-failure previous full run](logs/dystrail-current-complete-e2e.log), [focused stale-assertion run](logs/dystrail-current-stale-regressions.log), [incorrect extra Resume click](logs/states.log), and [14 passing receipt/journal cases on the preceding build](logs/dystrail-receipts-journal-chrome-final.log). The store test now checks the automatic departure directly; it no longer clicks a second Resume after the van has already resumed travel.

Earlier native Safari attempts were blocked; the successful resumed runs below supersede that status. The exact old session timed out; a new session was initially refused as already paired. A later bounded probe created and deleted an owned session successfully. The final-build test then timed out because its window stayed hidden; computer use on the next attempt reported that the Mac is locked. No Safari-wide reset, system-service kill or Dock retry was used. The owned preview server at port 56854 serves the current artifact.

The native harness now holds and fails a scene download before permitting launch, then checks Retry, the complete offline cache, current receipt/journal behavior and subtitle alignment. It uses the session's initial window when available and bounds WebDriver HTTP calls at 30 seconds:

```sh
SAFARI_PORT=56921 SAFARI_VISIBLE_WAIT=120000 SAFARI_OUTPUT=/tmp/dystrail-offline-final-safari-visible PLAYTEST_DIST=/tmp/dystrail-offline-final-web node tests-e2e/safari-feedback.mjs
```

The visibility wait expired while the Mac was locked and the harness closed its own session/server. A fresh test window can be opened after unlock; no native assertion pass is inferred from preparing or starting the harness. The first hidden-window attempt used port 56920 and a 30-second visibility timeout; its exit-1 log is retained.


## September 12: resumed Safari verification and stable Help & Tips

From `dystrail-web` (logs are under this directory's `logs/`):

```sh
SAFARI_PORT=56922 SAFARI_VISIBLE_WAIT=60000 SAFARI_OUTPUT=/tmp/dystrail-offline-resumed-safari PLAYTEST_DIST=/tmp/dystrail-offline-final-web node tests-e2e/safari-feedback.mjs
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-help-stable-web
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/scene-flow.spec.ts -g 'help preference' --workers=1 --output=/tmp/dystrail-help-stable-browser-results
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test tests-e2e/preload.spec.ts tests-e2e/pwa.spec.ts --workers=1 --output=/tmp/dystrail-help-stable-offline-results
SAFARI_PORT=56927 SAFARI_OUTPUT=/tmp/dystrail-help-stable-safari-verified PLAYTEST_DIST=/tmp/dystrail-help-stable-web node tests-e2e/safari-pwa.mjs
```

The first Safari command passed 18 checks and exited 0 on revision 1868d6c37d8c25729dae. The current Help & Tips build passed two geometry/preference Chrome cases, six Chrome loading/update cases and six native Safari cases, each runner exiting 0. The original geometry regression failed before the fix: the desktop scene moved 6.8125 px and the phone menu shrank by 8 px. After the fix, every measured rectangle and scroll position is identical.

The focused Safari PWA fixture initially omitted the required player name, then measured during its own smooth scroll, then compared Rust HashSet tag ordering as an ordered array. Those fixture failures are retained. The corrected fixture supplies a name through native input, starts from a settled viewport, compares tag membership without dropping any value, and waits for paint before screenshots. No game change was made to bypass an offline check.

From the repository root:

```sh
cargo test --workspace --all-targets
cargo build --workspace --release
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
cargo clippy -p dystrail-web --target wasm32-unknown-unknown -- -D warnings -W clippy::pedantic
cargo fmt --all -- --check
git diff --check
WASM_BINDGEN_BENCH_RESULT=/tmp/dystrail-help-stable-benchmark.json wasm-pack test --headless --chrome --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver dystrail-web
node --import /tmp/dystrail-game-client-loader.mjs /Users/vanna/.codex/skills/develop-web-game/scripts/web_game_playwright_client.js --url http://127.0.0.1:8083/play/ --click-selector '.retro-btn-primary' --actions-json '{"steps":[{"buttons":[],"frames":90}]}' --iterations 2 --pause-ms 1000 --screenshot-dir output/web-game-help-stable
python3 /tmp/dystrail-help-stable-audit.py
```

260 native tests, strict native/wasm lint, two registered wasm/Chrome tests and the unmodified game client pass with exit 0. Its screenshot and the current Chrome/Safari help screenshots were inspected. The HTTP audit verified all 87 files on the three owned preview ports. Browser launch and local network checks required the tool's sandbox escalation; successful reruns are distinguished from the retained permission failures. Engine and dependency source are unchanged by this UI fix, so prior campaign, coverage and security evidence remains applicable.

[Native Safari current results](safari/validation.json) · [18 feedback checks](safari/feedback/validation.json) · [Safari log](logs/dystrail-help-stable-safari-verified.log) · [Chrome layout log](logs/dystrail-help-stable-browser.log) · [Chrome loading/update log](logs/dystrail-help-stable-offline.log).
