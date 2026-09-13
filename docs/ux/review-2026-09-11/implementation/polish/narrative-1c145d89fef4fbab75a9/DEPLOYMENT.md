# Narrative release published and verified

The reviewed narrative build is live at https://dystrail.com/play/.

- Revision: `1c145d89fef4fbab75a9`
- Commit: `40df1edf293a8961a79b9efeda33d0383caaff7e`
- Branch: `release/crew-neutral-narrative-2026-09-13`
- Successful Pages run: https://github.com/VannaDii/Dystrail/actions/runs/34746871285
- Exact artifact: `/tmp/dystrail-crew-count-narrative/web`
- Frozen source: `/tmp/dystrail-crew-count-narrative/release-source`
- Offline manifest SHA-256: `c70601fce90ea1a63f2197d0077327fae727f7ca9b4d30c913534d54bd57730d`
- Release manifest SHA-256: `1301f36c950bf7c0155665105cb6305dc33be58ccaf8eabc8946aebfc43aac2f`

## Scope and isolation

The payload was staged in `/tmp/dystrail-narrative-release`, an independent clone of public production commit `acaa88b1c1f104bf267048d393df2be403f056c9` and revision `a9ff83cd924a07fdf4ec`. The isolated commit changes five game/fallback files, adds two generated JS/Wasm files, and updates the payload manifest and verifier. Nothing was deleted. All 59 non-game homepage, documentation and domain files and all 20 previously published hashed game assets remain byte-identical.

The compiled source differs from the preceding approved visual source in exactly 421 narrative string leaves across 21 JSON files. These are 20 common fields across 15 encounters, canonical data and all 20 existing locales, plus one Spanish-only correction. All 613 other source files are identical. Effects, timing, probabilities, IDs, choice ordering/counts and gameplay configuration are unchanged, including camp rest sanity +4. The complete narrative proof and 64-case localized desktop/phone/offline verification are in the sibling `crew-count-narrative-2026-09-13` directory.

No shared source or shared Git metadata was changed for publication. The existing artifact-only workflow deployed the exact committed bytes; no production build used the shared working tree.

## Live verification

Every one of the 196 public payload files matches the frozen SHA-256 and an explicit MIME policy. The worker, game HTML and offline manifest agree on the published revision. Homepage, play, documentation, HTTP-to-HTTPS, www canonical routing and the approved versioned homepage stylesheet match the expected destinations and bytes. `production-http-verification.json` contains all responses and assertions.

The temporary exact-branch deployment policy was removed after successful package and deploy jobs. Only the pre-existing main policy, ID 39031816, remains. `deployment-run.json` and `branch-policies-after.json` preserve the run and cleanup.

## Saved installation and offline verification

Before dispatch, a fresh disposable Chrome profile loaded real a9 production, created a six-person crew, saved Day 1 at zero miles, and passed complete-envelope preservation after offline reload. The user's browser storage was never used.

After publication, the same installation updated to the exact narrative revision, retained its complete saved state/pending/UI envelope, and reloaded offline. Chrome then closed and reopened with networking disabled before the first navigation. The full stored envelope remained equal; the Trail and Van were usable, all six saved crew names were visible, and the phone layout had no horizontal overflow.

At each updated online, offline reload and closed-browser offline loading-gate removal, the DOM observer recorded the correct revision in `ready` state, all 68 images decoded, and all seven prepared font faces loaded and registered. All document font faces were loaded. Cache verification checks all 119 asset sizes and SHA-256 values, 120 keys including the revision readiness marker, and zero missing or invalid assets. Browser runtime and console error lists are empty.

The unchanged normalized state SHA-256 is `3487425f6fb69a8a03553bff82813d81cc779fc7066cddeae8b50179fe47f20f`. Normalization changes only object-key order and the state/pending inventory-tag arrays stored as Rust HashSets. No gameplay fields or other array orders are excluded. This proves one real prior-production save fixture, not every historical save.

Native Safari was not rerun because the Mac is locked. These release checks used isolated Chrome and do not claim native Safari coverage.

## Review and evidence

The first push was rejected by automatic approval review because destination ownership/public history had not been established. Read-only checks showed the repository is public, the signed-in account is its owner with admin/push rights, the complete base history is already published there, and exactly one new commit contains only the reviewed release payload and integrity metadata. The same bounded push was then accepted. No transport workaround was used; the original reason and evidence are retained in `approval-review-record.json` and `push-egress-scope.json`.

Inspected production screenshots:

- [Updated desktop](output/playwright/production-updated-desktop.png)
- [Van offline on phone](output/playwright/production-offline-van-phone.png)
- [Closed-browser offline restart](output/playwright/production-restarted-offline-phone-viewport.png)

`publication-summary.json` is the concise machine-readable result. Exact reports, direct save snapshots, release/source integrity proofs, scripts and screenshot hashes are retained beside this report. Earlier review screenshots and the full 421-leaf before/after inventory remain in the sibling narrative evidence directory.
