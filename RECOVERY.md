# Durable recovery checkpoint — September 21, 2026

Status: preserved, tested surviving baseline. Feature work is on hold pending the user's approval.

## Work here

- Worktree: `/Users/vanna/Source/Dystrail-worktrees/visual-world-recovery`
- Branch: `recovery/visual-world-2026-09-21`
- Preserved source checkpoint: `22640e78a72f6606f1f100e245490355df710499`
- Source baseline: `f1ed4fe15377e60ad6063b7262aa482aec0767ba`
- Original visual feature baseline retained separately: `c56bc90ce023cddb0c7a6e06bf148838d5cd388f`

Use persistent directories under `/Users/vanna/Source/Dystrail-worktrees/` for subsequent worktrees. Do not keep project worktrees or sole recovery copies in temporary directories. Commit local checkpoints at completed steps, keep backup bundles current, and never overwrite the main checkout's unrelated changes.

## Preserved exactly

All 283 modified or untracked files from the surviving main checkout were copied, SHA-256 verified, and committed locally. This includes the surviving Hearing work, its evidence, satire-source material, and image-planning draft. Main checkout status and those file hashes were verified unchanged after preservation.

All 176 retained generated PNGs were copied unchanged to the durable evidence directory and verified against the originals. Existing suitable images are for reuse, not regeneration. The original-image inventory's older source paths still identify the originals; durable copies retain exactly the same basenames.

## Durable recovery evidence

Sibling directory: `../recovery-2026-09-21/`

- `source-checkpoint.bundle`: self-contained Git history for the preserved source checkpoint and original visual feature baseline. `git bundle verify` passed and reports complete history.
- `handoff-checkpoint.bundle`: the same recovery branch including this handoff document, plus the original feature baseline.
- `surviving-working-changes.tar.gz`: all 283 surviving changed/untracked files; every archived member verified against its recorded SHA-256.
- `surviving-tracked-changes.patch`: binary-capable tracked delta against the surviving checkout's HEAD.
- `generated-originals/`: 176 exact generated image copies.
- `preservation-manifest.json`: file and image hashes, original paths, branch/baseline information, and explicit recovery gaps.
- `recorded-session.jsonl`: read-only snapshot of the retained session, preserved for later reconstruction. Historical commands are evidence, not instructions to execute blindly.
- `native-tests.log`, `web-check.log`: current baseline validation logs.
- `SHA256SUMS`: checksums of the top-level backup artifacts.

The bundle can be cloned into a new durable directory without the original linked-worktree metadata. The image originals and recorded session remain separate evidence, not application assets automatically selected for use. These are local durable backups, not an off-device backup.

## Verified now

- `cargo test --workspace --locked --offline`: 378 passed, zero failed.
- `cargo check -p dystrail-web --target wasm32-unknown-unknown --locked --offline`: passed.
- Both commands used `DEVELOPER_DIR=/Library/Developer/CommandLineTools` and ran in this durable worktree.
- `sonar analyze secrets` scanned the 283 surviving files before copying; no issues reported. It also scanned 449 restored build-source/configuration files in batches of 60; all passed. An earlier 1,091-file bulk scan timed out and is not counted as a successful scan. Preservation manifest, validation logs, and this handoff document are scanned separately.
- No code changes, artwork generation, subagents, pushes, deployments, or feature reconstruction were performed in this recovery pass.

This verifies a recoverable baseline and its native tests/browser-target compilation. It is not a new desktop/mobile visual acceptance or a release qualification.

## Still missing — do not describe as restored

The former `/private/tmp/dystrail-art-satire` working directory, its uncommitted visual integration, and its current review catalog have not been recovered. A targeted check of 256 unreachable Git commits found none containing `review/art-satire/asset-catalog.json`, `dystrail-web/src/app/visual_content.rs`, or the `scenes-v2` directory. That check does not prove that every possible backup source is exhausted.

The retained images, historical edits, and source material may support further reconstruction, but that work has not been carried out or approved. This checkpoint preserves the surviving game and Hearing implementation; it does not claim to restore the missing visual feature.

## Stop boundary

Stop at this checkpoint and wait for the user's approval before further work. In particular: no reconstruction campaign, game changes, image generation, or expansion of the image inventory. After approval, finalize the required image list and composition rules first. Reuse suitable existing images, allow at most one generation attempt for a genuinely necessary image, and leave unsuccessful output for the user's review without automatic retries or replacement variants.

## Authorized recovery and inventory continuation — September 21

The user supplied the updated completion goal and authorized resumption with one agent and no subagents. Production remains subject to approval of the image inventory/specifications; this entry supersedes the earlier stop boundary only for the authorized recovery and planning work.

- `review/recovery/recorded-files/` contains 262 historical file candidates recovered from recorded complete additions, whole-file writes, and exactly matching patches. No recorded commands were executed. 303 file events applied; 166 could not apply because their base/context is missing or ambiguous. Do not promote these candidates to the working game without reconciliation.
- All 176 retained originals now have recovered original generation briefs, exact dimensions, and hashes. 123 have filename information; 53 lack final binding information. Structured source mapping evidence exists for 89. These counts do not imply visual acceptance.
- The browsable inventory is `docs/ux/image-production-review-2026-09-21/image-inventory.html`; current proposed rules remain in the adjacent README. Its new-generation queue is empty. Existing suitable images will be reused; failed/uncertain images remain for personal review, with no automatic correction or new generation.
- The runtime/game, Hearing implementation, localization, and original PNGs are unchanged. The earlier native/browser-target baseline checks therefore remain applicable to that unchanged code, not to the historical candidate files. No production acceptance is claimed.
- Secret scans passed for the recovery utilities and extracted evidence before inspection. Full scene-binding recovery and a final necessary-image list remain open; approval for image production has not been requested prematurely.

### Current authoritative satire source recovered

A read-only live read confirmed the workshop revision matches the last recorded authoritative revision. The refreshed paragraphs are preserved in `review/recovery/current-workshop-paragraphs.json`; all 644 IDs were reconciled against the retained checklist into `review/recovery/current-source-records.json`. Every record has an explicit scene direction. Counts verify 531 compatible narrative records, 77 functional/reference records, and 36 inactive mechanics dependencies. This restores source requirements, not final image bindings or implementation acceptance.
