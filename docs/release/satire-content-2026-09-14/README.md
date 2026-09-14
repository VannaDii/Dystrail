# Satire content publication — September 14, 2026 UTC

This release imports the A versions from the current Dystrail Satire Workshop into existing gameplay. The user requested publication, then limited the scope to content.

Source: https://docs.google.com/document/d/1YIwegfefxIMTskhFsbTSwHBmWNyGo9zCmANuKfvLf9k/edit

The captured native document was compared with all 16 local workshop tabs (4,706 paragraphs). `document-verification.json` records the revision and exact comparisons; `selected-units.json` preserves the 177 selected source packages.

## Scope

- 65 existing road encounters, retaining all 187 ordered choice effects.
- 44 intermediate town conversations; seven route endpoints retain their current records.
- Eight care scenarios, six outside-contact departures, four crossing packages, six orders, four repairs, eight activities, nine conditions, six openings, two hearing packages, and fifteen endings.
- Spanish, Italian and Arabic copy accompanies English. Other locale interfaces retain their existing translations and use English fallback for the revised narrative.

B/C variants, twelve proposed encounter families, additional mechanics and new artwork are excluded. Engine code, saved-state schemas, costs, probabilities, choice order, scene assets and the rest of the website remain unchanged. Presentation bindings select prose using existing state and completed engine outcomes.

## Runtime fidelity

The current permit implementation does not consume a Receipt, so successful permit passages use the workshop's passage text without its unsupported consumption claim. First checkpoint refusal uses the detour outcome, and paid roadside repairs use the roadside outcome. Care text names the affected person and preserves the distinction between a living departure and a death. An exhausted hearing uses exhaustion text, not a failed-vote claim. Existing abandonment text remains appropriate when abandoning at the destination.

## Validation commands

Executed from the isolated source checkout unless noted:

```sh
python3 docs/release/satire-content-2026-09-14/verify-content.py
cargo fmt --all
cargo fmt --all -- --check
cargo check -p dystrail-web --target wasm32-unknown-unknown --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -W clippy::pedantic
cargo clippy -p dystrail-web --target wasm32-unknown-unknown --all-features --locked -- -D warnings -W clippy::pedantic
cargo test -p dystrail-web workshop_tests -- --nocapture
cargo test -p dystrail-web exhausted_hearing
cargo test --workspace --all-features --locked
python3 scripts/check_advisory_exceptions.py
cargo audit --deny warnings
cargo deny check licenses bans advisories sources
cd dystrail-web
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/
PLAYTEST_EXTERNAL=1 PLAYTEST_PORT=51914 PLAYTEST_CHANNEL=chrome npx playwright test writer-content.spec.ts writer-narratives.spec.ts onboarding-results.spec.ts second-pass.spec.ts --reporter=line
PLAYTEST_EXTERNAL=1 PLAYTEST_PORT=51914 PLAYTEST_CHANNEL=chrome npx playwright test writer-narratives.spec.ts --grep 'critical care|critically ill|explicitly fatal|player cannot|unaffordable care' --reporter=line
```

The first browser pass exposed a missing space before critical-care warnings (66 passed, 10 affected). The renderer was corrected and all ten affected cases passed on rerun. Fourteen feedback/receipt browser checks also passed. Initial unit fixtures were updated for current serialization and revised titles. Strict lint required extracting weather presentation into its own helper; no diagnostics were suppressed.

Publication uses the exact artifact from passing source CI. Before deployment, compare every artifact integrity hash, retain all old hashed game assets, and verify a saved journey survives update, offline reload and a closed-browser offline restart. After deployment, verify live bytes, response types and browser rendering. Deployment records follow in the release artifact branch.

The first source CI run passed the native/Wasm tests, lint, security, release build, simulation sweep and six browser shards. Three assertions in each of the other two shards still expected the prior panic headline, direct-child trade buttons, and prior trade-journal prose. Their text/markup selectors were updated; all state, cost, reward and persistence assertions remain. All fourteen affected-file browser cases then passed locally in both viewport projects. The content verifier compares against the source baseline across committed as well as uncommitted changes.
