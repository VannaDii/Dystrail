# Current verification commands

Run from the repository root unless a command specifies the web directory. Build 56213723b1c62ed89b93 is the immutable `/tmp/dystrail-responsive-navigation-web` artifact. Logs and selected screenshots are retained in [units-2026-09-12](units-2026-09-12/).

```sh
cargo fmt --all -- --check
git diff --check
cargo test --workspace --all --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo clippy -p dystrail-web --target wasm32-unknown-unknown --all-targets --all-features --locked -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo build --workspace --release --locked
cargo tarpaulin --packages dystrail-game --fail-under 77
python3 scripts/check_advisory_exceptions.py
cargo audit --deny warnings
cargo deny check licenses bans advisories sources
```

The release used `NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-responsive-navigation-web` from `dystrail-web`.

Registered Wasm tests used `wasm-pack test --headless --chrome --chromedriver /tmp/dystrail-chromedriver-152/chromedriver-mac-arm64/chromedriver dystrail-web` with a temporary benchmark output path.

Chrome runs used the installed `chrome` channel, external port 8083, one worker, no tracing and bounded suite deadlines. New cases are `navigation-toggle.spec.ts` and `entry-layout.spec.ts`; regressions are `scene-flow.spec.ts`, `map.spec.ts`, `turn-history.spec.ts`, and `outcome-reachability.spec.ts`. Desktop and mobile were run separately after a combined teardown stalled. See validation.json for every exit status, including timeouts after passing assertions.

Native Safari used `PLAYTEST_DIST=/tmp/dystrail-responsive-navigation-web` with `tests-e2e/safari-inventory.mjs` and `tests-e2e/safari-pwa.mjs`, each on a disposable local origin. The unmodified develop-web-game client used the existing runtime loader, two 90-frame iterations, and screenshot inspection.

The preview publisher copied the verified immutable artifact, checked all served bytes and the worker, and left browser storage untouched. All three preview origins were then independently compared against the artifact. No commit, PR, production deployment, or social post was made.
