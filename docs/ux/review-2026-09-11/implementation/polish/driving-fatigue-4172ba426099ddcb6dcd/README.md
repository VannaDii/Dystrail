# Driving fatigue preview — 4172ba426099ddcb6dcd

Verified on September 13, 2026 and published to the local preview at http://127.0.0.1:8180/play/. Production remains the reviewed narrative revision `1c145d89fef4fbab75a9`.

The source is the exact published narrative source plus the 25 approved driving-fatigue/UI files: four engine files, the Conditions renderer and 20 locale bundles. All 634 frozen inputs were rechecked; 609 remain unchanged. All game-data/config files and narrative leaves remain unchanged, including camp Sanity +4. Source tree SHA-256: `6d6732a4d66cb12783e272c4105af9f07aae9208f19434448854c9f639ddb885`. The exact before/after hashes and integration-manifest hashes are in `source-verification.json`.

The build changes only the generated JS/Wasm pair, index, offline manifest and worker; 116 artifact files are unchanged. All 119 served cache assets (109,964,802 bytes) and the exact service worker match the frozen artifact. The preview update is atomic and does not access browser saves.

## Verified behavior

- Eight engine pace-fatigue regressions and all 81 frontend native tests passed (78 library + three integration tests). Formatting and strict native/Wasm Clippy passed; full command logs and exact counts are retained.
- 120 Conditions layout checks passed: all 20 locales, Classic/Deep, 320/393/1440px. Six icons, signed values and driving/day periods render without page or card overflow.
- Eighty keyboard help checks passed: both help controls in every locale/mode, bounded popovers, close/focus restoration, and unchanged day, clock, driving minutes, fatigue remainder, miles and stats.
- Actual clear-road travel in both modes covered 300/350/400 miles during five driving hours at Steady/Heated/Blitz, with direct pace Sanity costs 0/1/2. Every one-hour result was checked, including 13:00→next-day 08:00 rollover and the `1 h driving` receipt. These are isolated neutral-weather/healthy-vehicle fixtures, not replacement campaign acceptance tests.
- Offline reload after two hours retained exact clock, stats, miles and fractional fatigue for all six mode/pace combinations. Existing saves lacking the new remainder field load with zero.
- Two hours of guided forage produced +2 Supplies/+1 Sanity in both modes, with zero miles/driving time and unchanged 299-unit partial fatigue. No stationary pace cost was charged.
- Keyboard pace/diet selection, no-time behavior and offline persistence passed in both modes. The required unmodified web-game skill client completed two action bursts through persona selection, using the existing installed-Chrome adapter. The existing game has no render_game_to_text hook; DOM, actual saves and rendered screenshots supply gameplay evidence.

Reviewed screenshots include clean phone/desktop English and Arabic Conditions, Arabic help, the five-hour travel receipts and forage result. The temporary save toast was allowed to clear before clean captures. Browser errors: zero. Two temporary screenshot-harness mistakes (syntax and waiting for detached instead of hidden empty status) were corrected without product changes.

Native Safari remains unavailable because the Mac is locked. This unit does not close the remaining campaign retuning work, change any experience guard, update production, or create notifications.
