# Validation workflows

Working directory is the repository root unless noted otherwise. These are the current pass's workflows; failed audit and campaign outputs remain in this folder and were not suppressed.

```sh
cargo fmt --all
cargo fmt -- --check
cargo test --workspace --all --all-features --locked -- --nocapture
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic -A clippy::multiple-crate-versions
cargo build --workspace --release
cargo tarpaulin --packages dystrail-game --fail-under 77 --target-dir /tmp/dystrail-coverage-current
cargo audit
cargo deny check
cargo run -p dystrail-tester -- --mode logic --scenarios real-game --iterations 1000 --report console
python3 scripts/build_content_review.py
git diff --check
```

From `dystrail-web`:

```sh
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-conversation-build
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test feedback.spec.ts scene-flow.spec.ts ux-round.spec.ts onboarding-results.spec.ts preload.spec.ts pwa.spec.ts map.spec.ts --workers=2 --trace=off
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test map.spec.ts feedback.spec.ts ux-round.spec.ts --workers=2 --trace=off
SAFARI_OUTPUT=/tmp/dystrail-final-safari-verified node tests-e2e/safari-feedback.mjs
```

The installed Playwright CLI's isolated `dystrail-west-headless` session also exercised eight regional scenes and English, Spanish, Italian and Arabic conversations at 1280 and 390 CSS pixels. Its `snapshot` and `run-code` commands captured actual layout and screenshots. The game skill's smoke client used installed Chrome through a temporary copy; the copy was removed afterward.

Final review checks loaded all eight gallery images, exercised encounter/town region filters, searched Des Moines, opened care and scene inventories, and fetched JSON/CSV downloads successfully. Southwest yields 13 encounters / 36 choices and six towns; the Des Moines filter yields one town. The native release was rebuilt once more after the final text changes and finished successfully.

One review-check script initially used `URL` in the CLI sandbox, where that global is unavailable. It was rerun successfully with the known local URL. This was a check-script error, not an application error.

For outcomes, exact builds and browser limitations, see [validation.json](validation.json) and [README.md](README.md).
