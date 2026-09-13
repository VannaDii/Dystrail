# Validation commands and runner notes

Run from the repository root unless another directory is stated. Logs are retained in `logs/`.

```sh
cargo fmt --all -- --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo clippy -p dystrail-web --target wasm32-unknown-unknown --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo build --workspace --release
cargo tarpaulin --packages dystrail-game --fail-under 77
python3 scripts/check_advisory_exceptions.py
cargo audit --deny warnings
cargo deny check licenses bans advisories sources
cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report csv --output /tmp/dystrail-campaign-receipts.csv
python3 scripts/encounter_evidence.py
python3 scripts/build_content_review.py
```

The campaign exits 1 with the failures preserved in `logs/campaign-receipts.log`. Native tests and strict Clippy were rerun after the final browser-only internationalization cleanup. The engine was unchanged after the retained coverage run.

Browser Rust tests:

```sh
WASM_BINDGEN_BENCH_RESULT=/tmp/dystrail-evidence-wbg-benchmark.json \
  wasm-pack test --headless --chrome \
  --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver \
  dystrail-web
```

Chrome is 152.0.7977.84; the matching driver is 152.0.7977.82. The driver came from the matching build entry in Google's [Chrome for Testing manifest](https://googlechromelabs.github.io/chrome-for-testing/latest-patch-versions-per-build-with-downloads.json). CI uses the supported matching-driver option in [setup-chrome](https://github.com/browser-actions/setup-chrome).

The first plain `wasm-pack test --headless --chrome dystrail-web` used the automatically downloaded ChromeDriver 153 and failed with HTTP 404 after session creation. The matched driver ran two registered tests successfully. An initial run then reported a missing benchmark-output directory; setting the explicit benchmark result path removed that diagnostic. Supplying only `--chromedriver` without `--chrome` was rejected by installed wasm-pack 0.13.1 despite its help text saying the flag implies Chrome. Both flags are used above. The tool's newer-version notice remains visible.

Trunk, from `dystrail-web`:

```sh
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-evidence-final
```

The final output was copied to the isolated review server and requested 8180 preview. A six-worker Python check fetched all 88 paths from `/play/offline-manifest.json`, checked byte lengths and SHA-256 values, and wrote the manifest revision and totals to `validation.json`.

Chrome CLI used the existing skill wrapper with `npm_config_offline=true` and session `dystrail-receipts`. A synthetic run was created through onboarding, then actual encounter fixtures were imported through Menu → Load. Checks covered clicking the offered reward, comparing state with the HUD/outcome, reload, journal entry, help, browser offline mode, offline reload, a second offline award, and another reload. Locale checks used 1280, 390 and 320 px widths in English, Spanish, Italian and Arabic. Online mode and English were restored afterward.

The unmodified bundled game client was run with:

```sh
node --import /tmp/dystrail-game-client-loader.mjs \
  /Users/vanna/.codex/skills/develop-web-game/scripts/web_game_playwright_client.js \
  --url http://127.0.0.1:8083/play/ \
  --click-selector '.retro-btn-primary' \
  --actions-json '{"steps":[{"buttons":[],"frames":90}]}' \
  --iterations 2 --pause-ms 1000 --screenshot-dir output/web-game-receipts
```

The temporary loader resolves the project's installed Playwright and selects installed Chrome. Direct invocation initially failed because the skill directory had no Playwright package, and then because the project's downloaded headless-shell binary was absent. No skill client was rewritten. The two final screenshots were generated with exit 0; the final character-selection view was inspected. This client check does not substitute for the receipt gameplay sequence above.

Native Safari, from `dystrail-web`:

```sh
SAFARI_OUTPUT=/tmp/dystrail-evidence-final-safari \
PLAYTEST_DIST=/tmp/dystrail-evidence-final \
SAFARI_VISIBLE_WAIT=3600000 SAFARI_PORT=56854 \
node tests-e2e/safari-feedback.mjs
```

This check remains pending while its owned window reports hidden. Earlier 60-second and 300-second attempts timed out before receiving clicks. No current Safari pass is claimed. Previous wider Safari verification is linked in the parent review.
