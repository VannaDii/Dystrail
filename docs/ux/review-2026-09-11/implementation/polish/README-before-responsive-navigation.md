# Loading, HUD and journal verification

September 12, 2026. Preview: http://127.0.0.1:8180/play/. Build `84804cf5709353f5093a`.

The game stays behind its loading screen until the complete bundle is cached locally: **87 files / 90,472,173 bytes**. This includes every scene, portrait, map, stylesheet, game script and data file. Optional source articles are external reading and are not required for gameplay.

The build hook fingerprints all shipped files. The worker downloads with integrity checks and waits for every cache write; the game entry point awaits that preparation. Every subsequent launch checks every cache entry again. Missing assets block launch until repaired. An interrupted download shows Retry. A new release activates only after its complete bundle is ready; an interrupted update retains the previous complete build and save.

## Changes in this pass

- Fixed Help & Tips shifting the scene and menu. Hidden tips retain their layout space, are disabled and absent from the accessibility tree, and close any open tip. Information buttons stay available. Exact geometry remains unchanged when toggling with mouse or keyboard, including after an offline reload.

- Fixed the map's independent HUD showing zero receipts. Receipt counts are now required at every HUD call site and tested through awards, map/reload, town, trade, store and offline play.
- Kept Journal entries inside their scrolling panel instead of painting over the footer. The log is named and keyboard-focusable.
- Removed the inset green background behind Receipts, Morale and Allies.
- Gave encounter buttons a separate number column so subtitles align exactly with the primary text. They remain buttons with their existing accessible names and shortcuts. The layout also aligns in Arabic direction.
- Corrected stale browser fixtures to match the requested endings, stat cards, contextual weather effects, multi-option trading and one-click town departure. No product feature or test was removed to make these checks pass.
- Reverified the existing full-bundle launch gate in response to the repeated requirement; the gate was retained without a bypass.
- Excluded `.DS_Store` from release packaging. The Safari preview helper had treated that existing extensionless metadata file as an application route and returned HTML, which failed the worker's integrity check. The preview helper now checks for an existing file before using its route fallback. No game asset was removed.

## Current evidence

- 260 native Rust tests pass; strict native and wasm-target Clippy and native/Trunk release builds pass.
- Both registered Rust/Chrome browser tests pass.
- Twenty checks passed with exit 0 on the preceding `1868d6c37d8c25729dae` package: loading/offline/update behavior plus receipt awards, journal containment, outcome cards, town controls, ally acknowledgments and temporary effects, at desktop and phone sizes.
- Eight current Chrome desktop/phone checks pass with exit 0: Help & Tips geometry, keyboard and offline persistence; held/failed downloads, Retry and missing-cache repair; offline relaunch and update rollback.
- Native Safari 26.6.2 passes six checks with exit 0 on the current build: held first download, unchanged help layout at desktop/narrow widths, origin-stopped relaunch with all 87 asset hashes, complete update, failed-update fallback and a second offline relaunch. The entire saved state is compared; inventory tags are compared as the unordered Rust set they represent.
- Native Safari also completed all 18 feedback checks on the preceding build, covering receipts, journal, conversations, outcomes, store/map alignment and stationary camp. Its measured conversation width was 600 px.
- All 87 current served files match their lengths and hashes on ports 8180, 8083 and 56854. The preceding package also has a complete direct Chrome Cache Storage hash audit. No missing or invalid file is counted as ready.
- Button geometry checks at 390 px confirm identical primary/subtitle alignment, no horizontal overflow and no inset stat backgrounds. Desktop and phone screenshots were visually reviewed.
- The unmodified web-game client passes with exit 0; its resulting character screen was inspected.
- All 134 existing desktop/phone assertions passed on the preceding 88-file package: 67 desktop and 67 phone. The desktop runner stalled during cleanup and was stopped with exit 143; the separate phone run exited 0. The first 87-file package reran the 20 relevant checks above; the current Help & Tips build reran the eight relevant Chrome checks and six native Safari checks. Game source, artwork, styles and data were unchanged by the packaging fix; generated JS/Wasm export ordering and HTML integrity values changed during the rebuild. [Exact results](validation.json), [preceding evidence](validation-before-help.json) and [packaging comparison](bundle-comparison.json).

Latest screenshots: [Chrome help off](evidence/help-off-chromium.png), [native Safari help off](safari/help-off-600.png), [Safari update gate](safari/update-held.png). Earlier screenshots: [loading gate](evidence/loading-desktop.png), [desktop choices](evidence/choices-desktop.png), [phone choices](evidence/choices-phone.png), [Arabic layout](evidence/choices-ar-phone.png), [contained journal](evidence/journal-phone.png), [map receipts](evidence/map-receipts.png). [Commands and retained logs](commands.md).

## Remaining decision

Native Safari verification is complete. The earlier hidden/locked-window failures are historical and retained in the logs. No broad Safari reset or Dock workaround was used. Physical-device home-screen installation remains untested.

The engine and balance settings are unchanged since the [8,000-run campaign](../campaign/README.md). Its old acceptance targets still fail; the [preserve-gameplay versus retune decision](../campaign/balance-decision.md) remains open. Coverage from that unchanged engine is 78.72%. Existing dated dependency-maintenance exceptions remain recorded under `docs/security`.

The preceding complete Chrome run on `c15a520500ee421472e8` exited 1 with 122 passes and 12 failures caused by outdated assertions. Those failures and the focused correction runs are retained in the logs. An earlier 52-pass run required stopping its hung reporter and is not recorded as a clean pass.
