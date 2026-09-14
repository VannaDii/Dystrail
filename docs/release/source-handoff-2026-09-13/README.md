# Source handoff audit — September 13, 2026

Status: complete. Source and release artifacts are committed and pushed; the verified build is live in preview and production.

The earlier completion statement covered the published artifact but omitted committing and pushing its source workspace. Commit `bc5e664396c3c588f69978f9844f2f543e3faf4d` captures that source, artwork, tests and delivery evidence. Follow-ups through `9622e73a4f8fa250097efde226ac22dcc645b206` contain the final feedback and verification corrections below. They are on `release/complete-game-source-2026-09-13`; the independent kernel refactor on `main` is preserved without rewriting its history.

The [goal record](../../ux/review-2026-09-11/implementation/goal-outstanding-2026-09-12.md) retains the original F01–F12 review, consolidated requirements and later corrections. Evidence remains attributed to the build it actually tested. Physical-phone installation/offline success is user-reported; actual OS sharing and controlled browser sharing results remain distinct.

## Final feedback

| Change | Verification |
| --- | --- |
| Desktop status spacing | Removed the hidden help slot before destination; both modes checked at seven widths. Eight existing status/help regressions and original game-client smoke pass. |
| Language menu | All 20 choices share the containing menu's scroll area. The panel fits the actual remaining viewport, including enlarged header text. Ten English/Arabic geometry cases, short screens, focus, arrows, Home/End, Escape and manual scrolling pass. |
| Offline-ready icon | Shared SVG device/check replaces the circled font glyph; desktop Classic and phone Deep renders inspected after actual caching completes. |
| Crew and character navigation | Back and Continue share edge alignment and sizing across character and crew screens; phone layout and keyboard order put Continue first. The redundant visible character summary is removed while its screen-reader announcement remains. Four desktop/phone cases cover mode/persona/name preservation, reload and Arabic layout. |
| Homepage | Approved Play in browser / even offline treatment retained; obsolete Pants copy and screenshot replaced. Desktop and phone renders inspected. |
| Expanded campaign acceptance | All 8,000 campaigns pass the existing experience gates; no threshold or automated-strategy changes. The failing seed 1660 has a new regression. |

The expanded campaign run exposed a real 87.5% moving-day ratio against the unchanged 90% floor. The hearing-hall overnight briefing is now restricted to the Beltway where its story occurs. Its full-rest behavior and actual hearing-day accounting remain intact. Balanced health decay changes from 0.38 to 0.39, with a 1.1 Deep health factor. Attempting road travel while awaiting the hearing no longer starts another day. Config and deterministic replay fingerprints record those intentional changes; experience assertions remain pinned.

## Verification and release provenance

- Local preview: `97cc2071c96acca22247` (exact CI-built artifact; the focused CSS preview was `e81ca52a01c19a37704b`), updated on port 8180 after each verified UI unit. Its complete offline package has 119 assets.
- Local checks: 366 workspace tests, strict Clippy, formatting and release build pass. The full 8,000-campaign command exits successfully. Advisory balance warnings remain distinct from hard experience gates.
- Browser harness: 15 focused cases and three default-build fixture cases pass. Initial assertions now wait for the existing launch promise, image snapshots decode one stable batch, and offline fixtures resolve from the repository build. The map timer measures actual DOM duration, retaining the original limits. Manual-map cases are independent, retaining every wait and state assertion. Clipboard success explicitly grants that capability. No product assertion was removed or relaxed.
- [Final source CI](https://github.com/VannaDii/Dystrail/actions/runs/34792852543) passed all 414 browser cases in eight parallel jobs, all 8,000 campaigns, and every required build, lint, native/Wasm, security and language-coverage gate. Rust, Trunk, Binaryen and browser selection use pinned, verified tools. Browser checks consume the release artifact.
- [Production](https://dystrail.com/play/) is `97cc2071c96acca22247`, published by [Pages run 34793790091](https://github.com/VannaDii/Dystrail/actions/runs/34793790091). All 213 live file hashes and response types and six canonical routes pass. All 58 protected site files and 33 older hashed assets are preserved. The original main-only Pages deployment policy is restored.
- The early CSS-only candidate `837bb4f404afd0f3f7fc` and its composition/update evidence are historical, superseded by the final Rust changes. It was never published. Its files are retained for traceability, not presented as final-release verification.

[Final local verification](validation/final-feedback/verification.json) records counts, fingerprints and focused results. All validation commands are retained in the associated logs and workflow. The initial source capture preserves raw logs, licenses and patches. Newly curated text copies normalize trailing whitespace; original and normalized hashes are recorded in [the normalization manifest](validation/text-normalization.json).

The four new `docs/content-*` requirements/hit-list drafts belong to another task and are excluded from this release. They do not expand this goal or establish an uncommitted release change.

The exact CI artifact is published as `46a43c6efd6ba0b6d91f1d816c43846db942363e`: 213 files, 119 offline assets, 58 protected site files and 33 preserved hashed assets. Saved-install update, offline reload and closed-browser offline restart pass with all images and fonts ready. The expected active ruleset refresh is explicitly limited to `state.journey_daily.health.decay` (0.38 to 0.39); every other saved field remains unchanged apart from the existing set-order normalization. The initial unrestricted comparison is retained alongside the final strict migration check.

Two full CI runs exposed verification-fixture problems after the campaign checks passed. First, the headless browser still fetched service-worker requests during simulated offline mode; the fixture now refuses those connections and verifies actual cache eviction. Second, interrupted-update and writer checks began their screen assertions before the loading promise completed. The actual traces show the correct fallback version and writer content. Ordinary restored-screen checks now await the existing launch-ready signal; explicit loading-gate checks remain separate. A normalized comparison verifies that all existing assertions, strings and timeouts are retained. Four writer cases, two update cases and eight journey/camp cases pass locally in the installed headless browser. Failure evidence and the corrections are retained under `validation/full-browser-initial/` and `validation/readiness-initial/`.

The next full run passed 413 of 414 browser cases and all 8,000 campaigns. Its single mobile geometry failure compared button positions across two frames while the document scrolled from 237 px to 201 px. The 36 px movement exactly explains the apparent overlap; the actual button gap remained 20 px. The check now reads both button rectangles and their container edges in one evaluation, retaining every layout tolerance and keyboard assertion. Three desktop and three phone repetitions pass; `validation/geometry-initial/` preserves the trace measurements and targeted results.

No implementation, validation or publication items remain from this goal. [Final verification](validation/final-release/verification.json) links the exact source, release, live-file and saved-install evidence. This completion record is included in the final evidence commit on the source branch.
