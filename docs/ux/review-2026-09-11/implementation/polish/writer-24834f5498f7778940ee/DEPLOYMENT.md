# Writer content and verified preview production release

Published https://dystrail.com/play/ at 2026-09-13 05:27:39 UTC.

- Revision: `24834f5498f7778940ee`
- Pages run: https://github.com/VannaDii/Dystrail/actions/runs/34740330227
- Commit: `84dd03513a908f4e2c08b2664acd6258bd9e10d0`
- Branch: `release/writer-preview-2026-09-12`
- Frozen game artifact: `/tmp/dystrail-writer-final-web`
- Isolated release workspace: `/tmp/dystrail-writer-preview-release`
- Offline manifest SHA-256: `03a4aa2482f042800cd4d6eb937fbe3b845430f4a01779700755a202d5e8fc18`
- Full release manifest SHA-256: `163f8d7a636906e762ebe040a468f614203c2204af63ee02080f0f9bf74ef9ba`

## Exact scope

All 171 payload files passed local and CI verification, then live HTTP SHA-256 and MIME checks. The eleven changed game/fallback paths and three new hashed assets are listed in staging-verification.json. Zero files were removed. All 59 non-play/homepage/docs/domain files and all ten older hashed game assets are preserved byte-for-byte. The homepage retains its approved CTA and versioned stylesheet. The root fallback matches the game entrypoint. HTTP and www game URLs redirect to HTTPS dystrail.com/play/.

The release includes the completed writer document (59 encounters/169 choices, 28 town comments, 14 crew/ally narratives, 11 common-care strings) and Italian, Spanish and Arabic translations, together with the approved preview changes. Root rechecked the completed document unchanged at 2026-09-13T05:20:36Z. Camp rest remains +4; the separate +6 balance experiment is excluded.

The existing artifact-only Pages workflow published the immutable bytes. Temporary environment policy 59840474 was limited to this exact release branch, then removed after success. The environment again has only existing main policy 39031816. No shared source, shared git history, browser profile, or player save was changed by the deployment agent.

## Existing installation and offline verification

A disposable Chrome profile had the actual older production revision 08a07362f3e5c5445818 cached, with a saved Day 1, zero-mile crew. The live update reached revision 24834f5498f7778940ee ready state with all 101 cache asset lengths and SHA-256 hashes correct, 102 keys including the readiness marker, and all 67 images decoded. Saved crew, stats, resources, clock, route, decisions and active cooldown data remained exact.

The strict first comparison identified retired fields. The full original and updated save snapshots and save-migration-diff.json preserve every raw difference. Frozen source proves CampState retains only rest/repair cooldowns and forage uses the preserved activities.foraged_on; encounter timing uses driving exposure; TravelConfig retains only weather_factor. The final verifier permits only the exact observed removed zero legacy cooldowns, four retired fixed-mile configuration fields with asserted old values, and two new defaults (driving_minutes_total=0, last_encounter_driving_minutes=null). It then compares the entire state and pending envelope. Inventory.tags is normalized only because its Rust type is HashSet. No unknown field or active player value is ignored.

Chrome was fully closed and reopened with networking disabled before navigation. The complete save envelope, all 101 cached assets, ready marker, 67 decoded images and six crew survived. The previous verification had deliberately selected The Van after its initial baseline; the cold-restart expectation explicitly asserts that persisted tab 0 rather than ignoring UI state. Initial failure evidence is retained. All browser checks ended without page or console errors and without horizontal overflow at 393 pixels.

Desktop, phone Van, and current phone viewport screenshots were visually inspected. The phone capture is a 393 by 852 Chrome emulation, not a physical phone capture. This existing-save proof is limited to the tested untraveled crew; it does not claim all possible legacy save migrations have been exhaustively tested.

## Verification limits

Root's frozen-source verification: full browser run 29 passed 406/406 with exit 0; native tests passed 322 with three previously known configuration/replay failures still open; strict native/Wasm Clippy, formatting, release builds, two registered Wasm tests and source security checks passed. Those counts are root-provided prepublication evidence, not newly rerun by this deployment agent. Global balance acceptance remains separate. GitHub reported 47 default-branch Dependabot alerts during push; this repository notice is distinct from the frozen source checks and is not described as resolved.

## Evidence

- `deployment-run.json`
- `branch-policies-after.json`
- `production-http-verification.json`
- `production-browser-verification.json`
- `production-offline-restart-verification.json`
- `save-migration-diff.json`
- `old-production-autosave.json`
- `production-updated-autosave-first.json`
- `production-browser-first-schema-check.json`
- `production-offline-restart-before-navigation-expectation.json`
- `output/playwright/production-updated-desktop.png`
- `output/playwright/production-offline-van-phone.png`
- `output/playwright/production-restarted-offline-phone-viewport.png`
