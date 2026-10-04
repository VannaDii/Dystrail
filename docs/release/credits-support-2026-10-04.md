# Credits and support release verification — October 4, 2026

## Prepared source

- Release branch: `release/complete-game-source-credits-2026-10-04`.
- Published-source baseline: `9e9cd11ff8ce1d5051323032430751f282e62ad9`.
- Credits feature commit: `f16df44a89120f8c4437684282056bccc3e6ce47`.
- Validated source revision: `c39bf95db4d97b51daadab2874339b10efa75a4d`.
- CI artifact offline revision: `1aad1b566c4b0b93bda0`, 181 assets, 234,188,352 bytes.
- [CI run](https://github.com/VannaDii/Dystrail/actions/runs/37230990404).

## Passing checks

All 404 native workspace tests passed locally. CI passed native/browser Rust tests, native and WebAssembly lint, locale coverage, security checks, the optimized build, and the 1,000-campaign QA sweep. The dependency lock now uses rustls 0.23.45 to address RUSTSEC-2026-0285.

Direct browser checks verified the title, Menu, and results entry points; desktop and mobile layout; Escape, Close, backdrop dismissal, focus restoration, keyboard activation, offline opening, and external-link destinations and attributes. A direct check against the exact CI artifact held the hearing at `RoundRolling(0)` for 4.5 seconds while credits were open. The setup-screen regression test passed against that artifact.

## Full browser suite and publication hold

The broad browser suite did not pass. Its third group reported 41 failed and 36 passed tests; other groups were cancelled while investigating the long run. Failures include expectations for old copy, scene assignments, and control labels. No test checks were disabled or marked successful.

A representative writer-catalog failure was reproduced on the currently published game before this release: for `classic_service_station`, the test expects “Service Station Handoff”; the live game displays “Count the fingers.” The live offline revision was `85ed2e2f73064aeba9a7`. That same mismatch appears in the candidate artifact. The existing regression test and deployment workflow were unchanged by the credits feature.

Vanna deferred publication on October 4, 2026 because the satire rewrite is still in progress and explains the expectation drift. The credits release remains committed and pushed on the release branch; it has not been deployed. Resume release preparation after the rewrite is ready, reconcile this branch with the completed work, and rerun the required gates before publishing. The remaining behavioral failures and timeouts have not been individually diagnosed. No gate exception was granted, and no donation or payment was submitted.
