# Measured pace + crossing candidate

**Acceptance remains failed.** The full 2,000-run campaign completed, preserved all output, and exited 1 on the existing individual moving-day guard. No guard, strategy, route scale, time accounting or source file outside the three frozen Balanced crossing weights was changed.

## Commands and immutable scope

- `python3 /tmp/dystrail-retune-2026-09-13/run_candidate.py pace-crossing`
- Seed regression: `cargo test -p dystrail-tester balanced_run_survives_past_day_45` — exit 0, one test passed, 5.265 s.
- Release build: `cargo build --release -p dystrail-tester` — exit 0, 6.558 s.
- Campaign: `pace-crossing-tester --mode logic --scenarios real-game --seeds 1337 --iterations 250 --acceptance --report csv --output pace-crossing-results.csv` — exit 1 after all eight 250-seed scenarios, 12.075 s.
- Native binary SHA-256: `be823287a2e9ec1ec08817c9ccca33d0db7fbda4ee3191b90886d1756481f080`.
- Verified embedded source path: `/private/tmp/dystrail-retune-2026-09-13/pace-crossing-source/dystrail-tester`.
- All 538 frozen source hashes still match; only `balanced.json` differs from pace-source, with crossing weights 0.70 / 0.10 / 0.20. Camp remains +6 and Balanced health decay remains 0.12. The UI overlay is not applied.
- Shared Cargo target was released after the independent binary copy/path verification. No further build is running in this task.

## Complete numeric guard comparison

The native acceptance chain stops at its first failed individual ratio; it does not execute later aggregate checks after that failure. The table below independently computes the existing numeric bands from the complete CSV and lists every failing category, including aggregate checks that the native chain did not reach. It does not claim a native acceptance pass. Duplicated per-run/minimum-ratio assertions are represented by the same individual-ratio category.

| Scenario | Mean miles (1,900–2,100) | Mean days (8–30) | Mean physical miles/moving day (100–350) | Mean moving ratio | Bribe success (≥70%) | Terminal rate | Runs below 90% moving |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Classic - Aggressive | 2086.3 | 24.280 | 133.060 | 93.257% | 83.357% | 1.071% | 40 |
| Classic - Balanced | 2134.3 | 21.368 | 134.809 | 94.295% | 74.111% | 10.811% | 40 |
| Classic - Conservative | 2095.7 | 26.712 | 115.234 | 98.571% | 83.733% | 0.800% | 0 |
| Classic - Resource Manager | 2100.0 | 23.384 | 127.069 | 98.384% | 82.267% | 0.000% | 0 |
| Deep - Aggressive | 2050.2 | 23.324 | 136.661 | 92.960% | 87.408% | 0.671% | 40 |
| Deep - Balanced | 2135.5 | 21.640 | 134.886 | 93.048% | 74.111% | 10.811% | 63 |
| Deep - Conservative | 2100.0 | 26.888 | 114.902 | 98.383% | 86.800% | 0.000% | 0 |
| Deep - Resource Manager | 2100.0 | 23.440 | 126.871 | 98.317% | 82.267% | 0.000% | 0 |

The mean-ratio floor is 90% Classic / 92% Deep; the terminal-rate ceiling is 12% Classic / 16% Deep. All mean-ratio, duration, physical-mileage, bribe-success and terminal-rate guards pass this CSV audit. Both Balanced distance means remain above 2,100. The other six scenario CSV groups—**1,500 rows including every exported field—are identical to pace**.

| Classic Balanced additional guard | Required | Pace | Measured crossing candidate | Result |
| --- | --- | ---: | ---: | --- |
| Mean unique encounters / 20 days | ≥2 | 6.687 | 6.487 | Pass |
| Minimum unique / 20 days + epsilon | ≥2 (with 0.1 epsilon) | 4.000 | 4.000 | Pass |
| 2,000-mile milestone | ≥25% | 85.60% | 69.60% | Pass |
| Boss reach | 30–50% | 85.60% | 69.60% | FAIL |
| Boss wins | 20–35% | 46.80% | 39.20% | FAIL |
| Survived to day 12 or boss reached | 60–87% | 92.40% | 82.40% | Pass |
| Largest tracked failure family | ≤60% | 63.16% | 80.85% | FAIL |

Other scenario-specific numeric guards also pass: Classic Resource Manager ≥70% milestone; Deep Balanced ≥1.5 mean/minimum unique encounters and ≥25% milestone; Deep Conservative ≥25% milestone and ≥90% mean moving ratio; Deep Aggressive ≥65% boss reach, ≥2% wins, ≥1,980 mean miles and ≥70% milestone. No Classic Balanced ending identified by the existing “vehicle” guard occurs before 1,950 miles.

All nine remaining failure categories:

- Classic - Aggressive: each-run moving-day ratio — 40 runs; worst 84.615%, required ≥90%.
- Classic - Balanced: each-run moving-day ratio — 40 runs; worst 81.818%, required ≥90%.
- Classic - Balanced: mean internal campaign distance — 2134.2796.
- Classic - Balanced: boss reach — 69.600%.
- Classic - Balanced: boss wins — 39.200%.
- Classic - Balanced: dominant tracked failure family — 80.851%.
- Deep - Aggressive: each-run moving-day ratio — 40 runs; worst 84.615%, required ≥90%.
- Deep - Balanced: each-run moving-day ratio — 63 runs; worst 81.481%, required ≥90%.
- Deep - Balanced: mean internal campaign distance — 2135.4840.

The deduplicated numeric failure count changes from **12→9 categories**; individual failed runs change **207→183**. There is no all-green claim. The final crossing determinism/consistency validators are not reached after the native ratio failure, and the CSV lacks the full per-event payload to independently execute those exact validators. Their source is unchanged; the stream replay and campaign aggregate agreement are supporting evidence, not substitutes for eventual full acceptance. Existing warning-only messages (including Deep Aggressive field-repair coverage and endgame cooldown) remain in the preserved campaign log.

## Actual behavior and remaining risks

- Both Balanced campaigns exactly match the prior crossing-only forecast: **521/703 bribes succeed (74.111%); 76/703 crossings end a run (10.811%)**. The first checkpoint remains protected. Later terminal counts are 47 at the second crossing and 29 at the third.
- Each mode now has **44 pre-day-12 failures**, all crossings: Classic 28 on day 10 and 16 on day 11; Deep 23 on day 10 and 21 on day 11. Early survival is **82.4%**, in the Classic band.
- Classic mean distance improves **2,280.085→2,134.280**, still 34.280 above the ceiling. Deep improves **2,280.859→2,135.484**, still 35.484 high.
- Classic boss reach improves **85.6→69.6%**, but remains above 50%; wins improve **46.8→39.2%**, but remain above 35%. Deep wins are 27.6%.
- The failure mixture is worse in Classic: **76 crossing / 18 sanity**, so crossing comprises **80.851%** of tracked failures. Deep is 76 crossing / 23 sanity, or 76.768%; the 60% family gate is a Classic Balanced gate. No failure causes are relabeled.
- Classic individual moving failures decline **52→40**, but its worst ratio worsens **84.0→81.818%**. Deep declines **75→63**, but its worst ratio worsens **82.143→81.481%**. All real nonmoving days remain in the CSV and the denominator.
- Concrete short-run failure: seed 1364 in either mode ends at the second crossing on day 11, with 9 moving / 11 ledger days (81.818%), one recorded camp and a terminal crossing day. This needs the parent’s actual day-record diagnostic; this audit does not remove the terminal day or presume an accounting bug.
- Concrete long-run Deep failure: seed 1359 reaches 2,400 miles and loses the boss vote on day 27, with 21 full + 1 partial + 5 nonmoving days (81.481%); its record includes repeated camp days and the protected first-checkpoint refusal.

## Requested later bracket — stream-only, unselected

No new source or build was created for **0.70 / 0.09 / 0.21**. Replaying the same 250 isolated native crossing streams, with first-checkpoint protection and existing bribe settings, gives:

- Bribe success **519/699 = 74.249%**.
- Terminal rate **79/699 = 11.302%**, below the 12% Classic cap in this stream-only replay.
- Second / third terminal rolls: **51 / 28**, compared with 47 / 29 at 0.20; 101 detours versus 106.
- This is three more terminal runs in the stream-only cohort, with only 0.698 percentage points of headroom below Classic’s cap. Changed health, later reach and timing can change the denominator. It does not forecast distance, survival or full-campaign acceptance.
- The 0.22 bracket remains excluded: the prior stream-only terminal rate was 12.230%, already over Classic’s cap.

Root owns the separate common-health 0.38 measurement. No results from it are inferred here. The complete evidence is `pace-crossing-results.csv`, `pace-crossing-analysis.json`, `pace-crossing-campaign.log`, `pace-crossing-inputs.json`, `pace-crossing-binary-ready.json`, and `crossing-policy-021-forecast.json`.
