# Offline menu repair — September 11, 2026

Preview: http://127.0.0.1:8180/play/ · Offline build: `79387147b691d54008d8`.

The offline section previously occupied one column of the two-column menu. General header button padding also overflowed its small circular help controls. It now spans the menu, with a centered status/help row and a labeled installation action. Download progress and recovery actions have their own full-width rows.

Installation availability and installed state are now part of the component snapshot, so their events update the controls even when the offline readiness value does not change. An installed app hides the install control. Browsers without a native prompt show **How to install**, which opens contextual platform instructions. Changing languages refreshes all of the offline text immediately.

## Verification

- Chrome 152: eight cases passed, exit 0, desktop and phone projects. Menu width, contained help glyphs, contextual help and dismissal, download progress, retry, Arabic text/layout, install availability updates and installed-state handling. Install prompts are event fixtures; no app was installed on the host.
- Real offline recovery, all 80 asset files, saved town play, complete update before launch, and interrupted-update fallback also pass in Chrome.
- 236 native Rust tests, strict Clippy, formatting, native release and Trunk release pass. All 20 shipped locale key sets match; the new install-help label is translated into English, Spanish, Italian and Arabic, with English fallback elsewhere.
- Safari 26.6.2: 22 native WebDriver checks passed, exit 0, after the user left the browser idle. The final build passes full-width layout and centered help, labeled installation guidance, immediate Arabic translation, contextual help dismissal, saved play, all-asset offline recovery, complete updates and interrupted-update fallback. Two earlier attempts lost the first click during active browser use; those failure logs remain separate.

## Commands

```sh
cargo fmt --all
cargo fmt -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -Dclippy::all -Dclippy::pedantic -Dclippy::cargo -Dclippy::nursery -Aclippy::multiple-crate-versions
cargo test --workspace --all --all-features --locked -- --nocapture
cargo build --workspace --release
# From dystrail-web:
NO_COLOR=true PUBLIC_URL=/play trunk build --release --public-url /play/ --dist /tmp/dystrail-continuity-preview/play
PLAYTEST_PORT=8083 PLAYTEST_EXTERNAL=1 PLAYTEST_CHANNEL=chrome node node_modules/playwright/cli.js test offline-status.spec.ts pwa.spec.ts --workers=1 --trace=off --global-timeout=120000 --output=/tmp/dystrail-offline-ui-chrome
SAFARI_OUTPUT=/tmp/dystrail-offline-ui-safari node tests-e2e/safari-smoke.mjs
```

The bundled game-skill client still cannot resolve its Playwright package; the repository suite supplies the working Chrome loop. Existing dependency audit and campaign failures are documented in the preceding review and were not changed by this menu repair. No physical phone installation or production deployment was performed.
