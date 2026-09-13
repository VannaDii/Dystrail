# Wasm browser test harness repair

Frozen source: `/tmp/dystrail-visual-refine/unit4-source`.
Final result: process exit 0; both registered Wasm browser tests passed unchanged, 0 failed, 0 ignored, 0 filtered out.

- `wasm_tests::receipt_hud_updates_without_replacing_the_stat_row`
- `wasm_tests::evidence_choice_click_awards_what_the_button_offers`

## Confirmed cause

Installed Google Chrome is 152.0.7977.84. The automatically chosen cached ChromeDriver is 153.0.8010.36. An isolated direct session request reproduced HTTP 200 with legacy protocol status 33 and the explicit error that the driver supports Chrome 153 while the browser is Chrome 152. The response also contains a placeholder sessionId.

The unchanged wasm-bindgen-test-runner 0.2.106 reads that sessionId without validating the legacy status, so it proceeds to navigation and reports a misleading HTTP 404. Its cached source shows this at `wasm-bindgen-cli-0.2.106/src/wasm_bindgen_test_runner/headless.rs`, lines 434–461. The newer locally cached runner source independently documents this exact diagnostic issue; no runner upgrade was needed or performed.

Chrome's official [version-selection instructions](https://developer.chrome.com/docs/chromedriver/downloads/version-selection) require matching the browser's major/minor/build version when choosing the driver. An existing temporary driver at `/tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver` is version 152.0.7977.82, matching the installed browser's 152.0.7977 build.

## Repair and exact successful command

Use wasm-pack's explicit `--chromedriver` option. `WASM_BINDGEN_TEST_WEBDRIVER_JSON` chooses browser capabilities but does not select the driver binary. The same capabilities JSON passed unchanged.

From `/tmp/dystrail-visual-refine/unit4-source`:

```sh
CARGO_TARGET_DIR=/Users/vanna/Source/Dystrail/target \
WASM_BINDGEN_TEST_WEBDRIVER_JSON=/tmp/dystrail-visual-refine/wasm-webdriver.json \
WASM_BINDGEN_BENCH_RESULT=/tmp/dystrail-visual-refine/wasm-harness-repair/unit4-benchmark.json \
wasm-pack test --headless --chrome \
  --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver \
  dystrail-web
```

The first matching-driver run already passed both tests but emitted a benchmark-output directory error because the frozen source uses a shared target directory. The final run above supplies an existing output directory and removes that error. Only the pre-existing wasm-pack platform/install and newer-version notices remain; no browser/server error occurs.

No dependencies were installed or upgraded, and no Rust, game, test, or shared source was edited. All sessions used disposable headless browser profiles. Root's shared browser and saved game were untouched.

## Evidence

- `unit3-wasm-tests.log` and `unit4-wasm-tests.log`: preserved original 404 failures.
- `chromedriver-153-session-error.json`: actual status-33 version mismatch response.
- `chromedriver-153.log`: isolated failing driver process output.
- `unit4-wasm-tests-matched-152.log`: first two-test pass, including benchmark path diagnostic.
- `unit4-wasm-tests-final.log`: final two-test pass and clean exit 0.
- `unit4-benchmark.json`: successfully written benchmark result.
- `wasm-webdriver.json`: unchanged explicit browser capabilities.

Matched driver SHA-256: `647a0d49aee655cfc0d0c0ead33a8758924ab79e83bfd5a47cc278a1e430095d`.
Executed unit-test Wasm SHA-256: `1e86daa9149ff15df032c828ea561d9236cd849482294c1d97d825464eaca3dc`.

Before reuse after a Chrome update, recheck the installed browser and selected driver's major/minor/build versions; the temporary path alone is not a compatibility guarantee.
