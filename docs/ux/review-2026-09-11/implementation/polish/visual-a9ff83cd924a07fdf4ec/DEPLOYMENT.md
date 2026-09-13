# Verified visual release published

The approved immutable visual build is live at https://dystrail.com/play/. Publication and the complete post-deployment checks passed on 2026-09-13 UTC.

- Revision: `a9ff83cd924a07fdf4ec`
- Artifact: `/tmp/dystrail-visual-refine/unit5-web`
- Commit: `acaa88b1c1f104bf267048d393df2be403f056c9`
- Branch: `release/visual-refinement-2026-09-12`
- Successful deployment: https://github.com/VannaDii/Dystrail/actions/runs/34744528068
- Offline manifest SHA-256: `18d2e99114f6764b0a734cf80e9157ff31d1b526666c338f51d95be19e521c0b`
- Full release manifest SHA-256: `5bb61c022a37e257c2805f87e58744268c770b961f6d412defc1d4b5635b3198`
- Final publication evidence: `publication-summary.json`, with file hashes in `publication-evidence-sha256.json`.

## Scope and deployment

The release was prepared in `/tmp/dystrail-visual-release`, an isolated clone of prior production commit `84dd03513a908f4e2c08b2664acd6258bd9e10d0` and revision `24834f5498f7778940ee`. The shared source and shared Git metadata were not changed by staging or publication.

Only the approved artifact was overlaid. Eight existing game/fallback files changed and 23 files were added; no files were removed. All 59 non-game homepage, download-facing, documentation and domain files and all 13 older hashed game assets remain byte-exact. The inherited artifact-only Pages workflow verified and deployed the committed payload. Both package and deploy jobs succeeded.

The exact-branch deployment permission, ID `59845357`, was removed after completion. The only remaining policy is the pre-existing `main` policy, ID `39031816`. `deployment-run.json` and `branch-policies-after.json` preserve the successful run and cleanup.

Parent source-scope verification confirms that the engine, tester, all 20 locales and 22 content-data files are identical to the preceding released source. The shared camp-recovery experiment is excluded. This release does not claim to complete the remaining campaign-balance work.

## Live verification

`verify-live-release.py` exited 0. Every one of the 194 public payload files matches the committed SHA-256 and expected content type. The worker, HTML entry and offline manifest agree on the final revision. Homepage, play, documentation, HTTP-to-HTTPS and www canonical routes returned the expected live bytes and destinations. The homepage's approved versioned stylesheet remains exact.

`production-http-verification.json` contains every response, path, hash and content-type result. `staging-verification.json` records protected files, retained hashed assets and the complete change inventory.

## Existing installation, save and offline verification

The test used a disposable persistent Chrome profile created against real prior production before publication. It contains a six-person crew, Day 1 and zero miles traveled. The user's browser profile was never used.

The Node 24 update and closed-browser restart scripts each exited 0:

1. The old installation updated from `24834f5498f7778940ee` to the final revision. Its complete state, pending state and UI save envelope were preserved.
2. With networking disabled, the updated installation reloaded successfully, retained the same save and displayed all six crew members. The Van was inspected at desktop and 393px phone widths without horizontal overflow.
3. Chrome was closed, then reopened with networking disabled before the initial navigation. The saved envelope was restored exactly, the Trail and Van remained usable, and all six saved crew names appeared.

At each of those three loading-gate removals, a DOM observer recorded the final revision in the ready state, all 68 images decoded, and all seven prepared font faces loaded and registered in `document.fonts`. All seven document font faces were loaded too. The full cache check verified the sizes and SHA-256 of all 119 assets, 120 keys including the correct readiness marker, and no integrity failures. Browser runtime and console error lists are empty.

The unchanged normalized state hash is `8e782e3950290a64fbb4a7081e0fa68375c9dc1b5f331b22545287f01642a39c`. Normalization covers object-key serialization order and only the state/pending inventory tags stored as Rust HashSets. No gameplay fields or other array orders were excluded. No schema changes, rule changes or new state fields were required.

This verifies one real prior-production save fixture, not every possible historical save state. Parent pre-release browser coverage remains separate evidence for the broader gameplay states.

Reports and direct save snapshots:

- `production-browser-verification.json`
- `production-offline-restart-verification.json`
- `old-production-autosave.json`
- `production-updated-autosave.json`
- `production-offline-autosave.json`

The initial old-production preflight's raw JSON hash assertion failed before publication. Its failure, original snapshots and field-level comparison are preserved in `old-production-browser-first.json` and `old-production-save-diff.json`; the corrected complete-envelope check then passed. The raw-hash method was sensitive to serialization order. The fresh field-level comparison found only unordered pending inventory-tag positions changed, and the final verification ignores no other values.

## Screenshots inspected

- `output/playwright/production-updated-desktop.png`
- `output/playwright/production-offline-van-phone.png`
- `output/playwright/production-restarted-offline-phone-viewport.png`

Additional desktop/phone full-page captures remain in `output/playwright/`. The inspected views show the new typography, compact status display, stopped crew scene and illustrated Van contents on the actual production build, including offline use.

## Earlier verification supplied by the parent

The approved artifact passed the final 80-case focused browser run, 81 frontend native tests, unchanged Wasm tests, formatting, strict native and Wasm Clippy, game-client smoke, WebKit HUD/store/focus checks, and 132 settled checkout hit checks with exact purchase quantity, total and cash assertions. These were completed by the parent and other agents before the production signal; this release subtask did not rerun them. Copies of relevant records are in `verification/`.

No product changes were made after the final verified-artifact signal. The temporary WebKit dispatch hold was cleared by settled hit tests; the artifact remained unchanged.
