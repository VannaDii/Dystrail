# Share images, header and mobile status — September 13, 2026

Historical release **33dd2b05a6658d330912**, superseded by the [completed focus follow-up](../native-share-focus-d3ed78dac458b64a80ee/README.md). Published by [commit 27f71c1b7f448487da07dced09c61a9b8804903a](https://github.com/VannaDii/Dystrail/commit/27f71c1b7f448487da07dced09c61a9b8804903a) through [successful Pages run 34779820971](https://github.com/VannaDii/Dystrail/actions/runs/34779820971).

## Delivered

- Share PNG places the actual player portrait beside the name, role and mode, with a separate outcome, geographic progress and six results. Long names wrap and shrink within bounds; Arabic mirrors the composition. Both themes use their existing art, fonts and palette, including soft black contrast over the scene.
- The opaque game header stays above the gold divider while the page scrolls underneath. Menu and Save/Load remain reachable and above the page.
- On mobile, the first small status row holds day/time, pace, diet, cash and vehicle condition. City/destination and weather/policies share the lower row, wrapping longer notices within the screen. Cash and vehicle use icons and values; detailed costs remain inside help.
- Daily journal disclosure reads **The Day's Events**, with a distinct translated key in all 20 locale files. It remains separate from the tabs' accessible details label.
- Earlier transparent-tab revision **8b8fcc3f7004fd8ec354** is preserved: no tab background in selected/hover/focus states, while text, underline and accessible keyboard focus remain.

## Verification

79 frontend unit tests and strict native/Wasm lint pass. The twelve existing share-flow cases pass on the new runtime: generated exports, edited copy, downloads, simulated native-sharing responses and failure paths, Arabic and long names. Simulated platform responses do not establish actual OS handoff.

All six final mobile impact/help tests pass. The original 110px height limit is retained, with added bounds/alignment assertions for the requested two-row structure. A prior run exposed real long-policy clipping and two assertions tied to the old wrapper; the final structural fix and updated layout assertions pass without changing gameplay tests. Three additional 320/393px English/German/Arabic checks keep all first-row values and help controls inside the viewport. Four Classic/Deep phone/desktop checks verify the sticky header, open menu, modal stacking, journal disclosure and unchanged game state. The unmodified game-client smoke finishes and its screenshot was inspected.

Share screenshots were generated on the earlier review build d097feac418aaf108816; its share-image source is byte-identical to the final source. The twelve share regression cases were rerun on the final Rust runtime. The final CSS fix was tested against the exact published artifact. `release-browser-tests.log` retains the two pre-fix layout failures, which are superseded only by the six passing `verified-impact-tests.log` cases.

All **205** production file hashes and MIME checks and six canonical routes pass. **59** non-game site files and **28** old hashed assets are preserved. The exact temporary release-branch permission was removed and the original main-only policy restored. The preview was refreshed after each verified unit; the final update changed only eight artifact paths and preserved saves.

A disposable installation using the exact old and new published artifacts on one local origin preserves the complete save through update, offline reload and a closed-browser offline restart. All **119** cached assets match their hashes; **68** images are decoded and **seven** font faces are loaded. The renamed journal disclosure works offline. This is artifact-level update verification, distinct from the live HTTP byte checks; it does not claim a physical phone install.

The accepted hourly balance and final writer copy are unchanged. Their completed 2,000 campaigns, 364 native tests, two Wasm tests and 408 browser cases remain historical release evidence, not newly repeated tests for these presentation changes.

## Device evidence at this release

Native Safari resumed on ef9b4b81cdeb1bc0700d: 25 feedback groups and 33 smoke groups completed, followed by three ending checks and Classic image/copy/draft/focus checks on 8b8fcc3f7004fd8ec354. A stale early-ending route fixture was corrected to the actual West Coast leg. These are completed assertions, not a claim that every Safari harness ended cleanly. The native tab probes all measured 1728×1032; labels referring to narrower sizes do not establish native mobile-layout coverage.

At publication of this artifact, actual OS share-sheet handoff/cancellation and physical phone installation/offline launch were unobserved. They are now resolved in the [final record](../native-share-focus-d3ed78dac458b64a80ee/README.md). A later Safari GUI attempt stopped when the user switched to another app window, without interacting further. No content was posted or sent, and no player browser save was modified. See the [outstanding goal](../../goal-outstanding-2026-09-12.md).
