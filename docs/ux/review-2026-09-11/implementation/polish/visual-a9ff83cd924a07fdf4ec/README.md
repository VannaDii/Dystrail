# Visual refinement verification — September 13, 2026

Verified game revision: **a9ff83cd924a07fdf4ec**. Immutable artifact: `/tmp/dystrail-visual-refine/unit5-web`. Offline manifest SHA-256: `18d2e99114f6764b0a734cf80e9157ff31d1b526666c338f51d95be19e521c0b`.

Local preview publication completed, with all 119 asset lengths/hashes and the service worker matching the artifact. The publication did not touch browser storage. Production dispatch uses commit `acaa88b1c1f104bf267048d393df2be403f056c9` and [Pages run 34744528068](https://github.com/VannaDii/Dystrail/actions/runs/34744528068). The workflow succeeded at 2026-09-13 07:10:06 UTC. All 194 public payload hashes and MIME types match; all 59 non-game files and 13 older hashed game assets remain exact. The temporary deployment policy was removed and the main-only restriction restored. See the production HTTP, browser and offline-restart reports in this directory.

The preceding production installation updated successfully, preserving the complete tested Day 1, zero-mile save envelope and all six crew members. Object keys are canonically ordered and only inventory HashSet tags are sorted for comparison; no gameplay values or schema changes were ignored. Online update, offline reload and a completely closed-browser offline restart each captured loading-gate removal with all 68 images decoded and all seven prepared/document font faces loaded and registered. All 119 cached asset sizes and hashes match; the selected Van tab survives restart. This proof covers the tested fixture, not every legacy saved-game variant.

## Delivered changes

- Classic retains navy/gold; Deep now applies plum/rose consistently to surfaces, controls, borders, selected states, hover, shadows, focus, help dialogs and generated share images. Physical scenery and weather retain their own colors. High contrast remains available.
- The lower phone HUD is 70px at 393px in clear weather and 100px with an active policy; at 320px it uses 100px. Desktop remains 40px. Cash and vehicle readouts use icons and values, with accessible text retained. Effect costs, modifiers and duration live in their information panels. Labels and values use clearer font weights.
- Seven locally bundled font faces load from verified cached bytes before the loading gate closes. Unused bold, italic and Arabic text causes no late font download. Missing or corrupt font bytes hold the gate; Retry repairs the cache and resumes preparation.
- Phone layouts preserve readable Conditions labels, crew portraits above role/name fields, reachable persona continuation, a contained menu and correct RTL supply fractions. Required encounter choices precede optional Trail history.
- Trail and Journal share the same compact statistic-card dimensions on phone and desktop. Phone store rows retain 11 illustrations, separate 44px quantity controls and readable item descriptions; the existing Pixel 7 workspace-height limit remains unchanged.
- Six additional encounter settings cover motel, cafe, library, museum, farm office and service counter. All 65 encounter IDs have scene mappings, with 28 mappings corrected for the completed writer content. Active crew filtering and scene-caption contrast remain intact.
- Results use all 12 shared statistic cards. The share composer keeps its actions reachable on short phones, supports editable text and offline PNG export, and uses the active palette and bundled fonts.

## Verification and limits

| Check | Result |
| --- | --- |
| Full unit3 browser suite | 405/408 passed. The three failures were mobile Trail/Journal dimensions, store height and the old same-row Arabic store assumption. Original failure log retained. |
| Final unit5 focused browser suite | **80/80 passed**, clean exit 0, including all three corrected cases plus impact help, navigation, history, outcomes, result/share behavior and onboarding. Card equality and `<1700px` Pixel 7 height assertions were not relaxed. |
| Final frontend native tests | **81 passed**, clean exit 0: 78 unit tests and three integration tests. |
| Registered Wasm browser tests | **2 passed**, unchanged, clean exit 0. Tested unit4 Rust is byte-identical to unit5 Rust. The harness required the existing Chrome 152-compatible driver; the original 153 mismatch response is retained. |
| Formatting and strict native/Wasm Clippy | Passed on frozen unit5, clean exits. |
| Required game-client smoke | Passed on unit5, clean exit 0; screenshot inspected. |
| WebKit HUD | All 20 combinations of two modes, clear/policy state and 320/393/430/600/1440px widths passed without document overflow or page errors. Eight phone screenshots and four popups inspected. |
| Final WebKit store/focus | Four mode/phone combinations passed; focus uses a 3px solid Classic gold or Deep rose outline. All 132 settled quantity-control hit checks and real last-item add/remove clicks pass in both modes; an initial smooth-scroll sampling failure is retained as a harness diagnostic. |
| Scene/art/loading audit | 24 combinations on unit3: six settings, two modes, desktop/phone. All required choices and source help remain contained. All 68 images and seven fonts were prepared before launch, with zero later asset downloads during offline checks. Scene source/assets are unchanged in unit5. |
| Result/share audit | 24 unit3 snapshots across two modes, English/Arabic and 1440/393/320px. All values and actions retained. Four actual offline PNG downloads inspected and pixel-checked. Result source/assets are unchanged in unit5; final focus color and sharing regressions pass. |
| Store layout audit | 18 staged combinations across both modes, EN/DE/AR and 412/393/320px. Pixel 7 English height fell from 1984.48px to 1660.48px; all illustrations and 44px quantity controls remain. Final compiled shopping cases pass. |

These are bounded Chrome/WebKit checks, not a claim of physical-device testing or universal pixel perfection. The full 408-case suite was not rerun after the final CSS fixes; the final 80-case run covers the changed and adjacent surfaces.

## Preserved scope

The engine, automated tester, all 20 locale files and all 22 content-data files match the preceding production release source exactly; see `unit5-engine-content-scope.json`. The separate camp+6 experiment is excluded. No gameplay threshold, strategy or experience assertion was weakened.

Overall gameplay retuning remains open, including the three unresolved configuration/replay expectations and campaign acceptance metrics recorded in the current goal. Writer copy still contains fixed six-person references in `beltway_briefing` and `classic_union_blockade`; adapting those references to reduced crews is an outstanding content task. This visual release preserves the supplied wording.

The attached reports distinguish the tested revisions and retain the initial failures, cache-recovery proof, exact frozen source hashes and representative screenshots.


## Review inventory

The local content review is regenerated from the frozen unit5 source: 65 encounters, 187 choices, 51 towns and 18 scene compositions. The generator now parses braced Rust match arms, uses the actual raw-milk scene assignment, and renders all six new interior crops accurately. Generated JSON, CSV and embedded HTML match; desktop/phone catalog rendering passes. Historical editorial and source dates remain intact. This documentation repair did not change the published game artifact.

[Final deployment record](DEPLOYMENT.md) and [publication summary](publication-summary.json) retain the release commit, live integrity checks, saved-state proof and deployment-policy cleanup.
