# Activity settlement: pinned campaign comparison

Candidate `pace-crossing-health038-activity` versus `pace-crossing-health038`. Completed 2,000 unchanged runs (8 scenarios × seeds 1337–1586), campaign exit **1**. All focused regressions, seed 4242 survival guard, and release build exited **0**. No source/config/test adjustment was made after the reviewed freeze.

Moving-ratio failures decreased **125 → 104**: 76 resolved, 55 newly failing, 49 persist. The new failures include two Classic Resource Manager seeds and one Deep Resource Manager seed. Both Balanced distance means remain above 2,100. All previously passing aggregate mean/outcome targets still pass; the new failed guard categories are the two Resource Manager per-run/minimum-ratio checks.

## Source and execution proof

- Frozen patch: `e16c96424bce041058749c4f7bb8bd9ad31c4b53bda0aff1e3b988fc70bf227b`; only `dystrail-game/src/activities.rs` differs from the base, with 537 other input hashes unchanged.
- All 538 input hashes verified after the run. All Rust files were touched immediately before each compile to avoid stale shared-target artifacts.
- Immutable binary SHA256: `bc49a7ce2d5dafc67831a140d73c0d2688d21aff716461a43792f6c743fcf392`.
- Embedded compilation path verified: `/private/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-source/dystrail-tester`. Cargo was released to root after the binary copy/verification, before this report.
- CSV SHA256: `ff8478e44ee0b96d25245cd22426f0312847a05f427418a0a6bca7d4c682cfa7`.
- Analysis SHA256: `a53d18d5695af3f638e08c0848ac4abfb567d31b194c455ab4914f764148959c`.
- Complete failed campaign log SHA256: `66e2a2328dde0c25dfc9ff3e1624b75780b66af12146da300eb7cc5ebf9cf091`.

| Command/test filter | Exit | Tests passed | Seconds |
|---|---:|---:|---:|
| settlement-regressions | 0 | 7 new | 5.609 |
| activity-existing | 0 | 10 (includes the same 7 new) | 3.691 |
| clock-existing | 0 | 13 | 3.647 |
| journal-existing | 0 | 5 | 3.290 |
| ledger-existing | 0 | 7 | 3.435 |
| seed4242 | 0 | 1 unchanged | 4.374 |
| build | 0 | — | 6.447 |
| campaign | 1 | 2,000 runs completed | 11.802 |

The independent focused modules contain 35 distinct engine tests in total; the first seven-test invocation repeats the seven included in the Activity module. None failed. The campaign native validator stops at its first record violation (Classic Balanced 88.0%); the complete numerical-band comparison below is reconstructed from all exported rows using the unchanged analyzer. CSV mileage uses the existing one-decimal export; moving ratios and unique-per-20 figures are reconstructed from exact ledger counts, not rounded display ratios.

## Numerical guard changes

| Scenario | Failing runs before → after | Worst moving ratio before → after | Mean moving ratio before → after | Mean internal distance before → after |
|---|---:|---:|---:|---:|
| Classic - Aggressive | 40 → 37 | 84.61538% → 85.71429% | 93.25668% → 93.62223% | 2086.2660 → 2086.2204 |
| Classic - Balanced | 9 → 11 | 81.81818% → 81.81818% | 95.53529% → 95.81649% | 2104.9396 → 2102.0052 |
| Classic - Conservative | 0 → 0 | 92.85714% → 92.59259% | 98.57069% → 98.17125% | 2095.7012 → 2095.7556 |
| Classic - Resource Manager | 0 → 2 | 91.66667% → 87.50000% | 98.38432% → 97.78035% | 2100.0000 → 2100.0000 |
| Deep - Aggressive | 40 → 33 | 84.61538% → 84.00000% | 92.96044% → 93.37662% | 2050.1756 → 2056.5588 |
| Deep - Balanced | 36 → 20 | 81.81818% → 81.81818% | 94.56482% → 95.59367% | 2131.7248 → 2131.8124 |
| Deep - Conservative | 0 → 0 | 92.59259% → 90.00000% | 98.38304% → 98.01980% | 2100.0000 → 2100.0000 |
| Deep - Resource Manager | 0 → 1 | 91.66667% → 88.00000% | 98.31704% → 97.75464% | 2100.0000 → 2100.0000 |

Each-run/minimum ratio must be at least 90%; aggregate means must be at least 90% Classic / 92% Deep. Mean campaign distance must remain 1,900–2,100 internal miles. The per-run and aggregate-minimum validators represent the same failed record sets, not additional failed runs.

| Remaining failed numerical guard | Before | After |
|---|---:|---:|
| Classic - Aggressive: every-run/minimum moving-day ratio | 0.846154 | 0.857143 |
| Classic - Balanced: every-run/minimum moving-day ratio | 0.818182 | 0.818182 |
| Classic - Balanced: mean internal campaign distance | 2104.939600 | 2102.005200 |
| Classic - Resource Manager: every-run/minimum moving-day ratio | 0.916667 | 0.875000 |
| Deep - Aggressive: every-run/minimum moving-day ratio | 0.846154 | 0.840000 |
| Deep - Balanced: every-run/minimum moving-day ratio | 0.818182 | 0.818182 |
| Deep - Balanced: mean internal campaign distance | 2131.724800 | 2131.812400 |
| Deep - Resource Manager: every-run/minimum moving-day ratio | 0.916667 | 0.880000 |

| Other global targets (all pass before and after) | Mean days (8–30) | Physical miles/moving day (100–350) | Bribe success (≥70%) | Terminal crossings (≤12% Classic / 16% Deep) |
|---|---:|---:|---:|---:|
| Classic - Aggressive | 24.280 → 24.068 | 133.060 → 133.668 | 83.357% → 83.408% | 1.071% → 1.071% |
| Classic - Balanced | 21.052 → 20.916 | 133.515 → 133.526 | 74.111% → 74.111% | 10.811% → 10.811% |
| Classic - Conservative | 26.712 → 26.724 | 115.234 → 115.660 | 83.733% → 83.733% | 0.800% → 0.800% |
| Classic - Resource Manager | 23.384 → 23.384 | 127.069 → 127.881 | 82.267% → 82.267% | 0.000% → 0.000% |
| Deep - Aggressive | 23.324 → 23.272 | 136.661 → 136.596 | 87.408% → 87.097% | 0.671% → 0.402% |
| Deep - Balanced | 21.644 → 21.472 | 132.596 → 132.226 | 74.111% → 74.111% | 10.811% → 10.811% |
| Deep - Conservative | 26.888 → 26.900 | 114.902 → 115.284 | 86.800% → 86.800% | 0.000% → 0.000% |
| Deep - Resource Manager | 23.440 → 23.460 | 126.871 → 127.508 | 82.267% → 82.267% | 0.000% → 0.000% |

| Scenario-specific target (all still pass) | Allowed | Before | After |
|---|---|---:|---:|
| Classic - Balanced: unique encounters/20 mean | ≥2 | 6.767684 | 6.813036 |
| Classic - Balanced: unique encounters/20 minimum with epsilon | ≥2 | 4.100000 | 4.100000 |
| Classic - Balanced: 2000-mile milestone | ≥0.25 | 0.696000 | 0.688000 |
| Classic - Balanced: boss reach | 0.3–0.5 | 0.404000 | 0.416000 |
| Classic - Balanced: boss wins | 0.2–0.35 | 0.232000 | 0.224000 |
| Classic - Balanced: survival or 12 days | 0.6–0.87 | 0.824000 | 0.832000 |
| Classic - Balanced: dominant tracked failure | ≤0.6 | 0.490323 | 0.484076 |
| Classic - Resource Manager: 2000-mile milestone | ≥0.7 | 1.000000 | 1.000000 |
| Deep - Balanced: unique encounters/20 mean | ≥1.5 | 6.589019 | 6.595464 |
| Deep - Balanced: unique encounters/20 minimum | ≥1.5 | 4.000000 | 4.000000 |
| Deep - Balanced: 2000-mile milestone | ≥0.25 | 0.696000 | 0.696000 |
| Deep - Conservative: 2000-mile milestone | ≥0.25 | 1.000000 | 1.000000 |
| Deep - Conservative: additional mean moving ratio | ≥0.9 | 0.983830 | 0.980198 |
| Deep - Aggressive: boss reach | ≥0.65 | 0.980000 | 0.988000 |
| Deep - Aggressive: boss wins | ≥0.02 | 0.724000 | 0.704000 |
| Deep - Aggressive: additional mean distance | ≥1980 | 2050.175600 | 2056.558800 |
| Deep - Aggressive: 2000-mile milestone | ≥0.7 | 0.980000 | 0.988000 |

Classic Balanced vehicle-ending-before-1,950 count remains 0 → 0. The per-event crossing determinism/consistency checks are downstream of the native first-failure return and cannot be independently proven from the aggregate CSV; neither campaign log reports the existing crossing-telemetry anomaly warnings. This report does not claim those unexecuted validators passed. All numeric guard values, bot policies, seed lists, and mileage definitions remain unchanged.

## Paired results and limits

- 630 rows are byte-identical across all CSV fields; 1370 rows have at least one changed field. The comparison JSON retains every paired field delta and every failing seed.
- Classic Aggressive seed 1480 is identical in all 48 exported fields and remains 21/24 = 87.5% moving. The bounded Activity-only correction does not settle a day that a prior conversation already entered.
- Deep Aggressive seed 1480 improves 20/23 = 86.9565% → 21/23 = 91.3043%, with internal distance 2,056.2 → 2,067.6. Camp count remains 2; the second camp tag moves from day 21 to day 22 and total stationary days decrease 3 → 2. The CSV does not export the individual daily movement kind, so this result alone does not establish the full changed action path. The day ledger is retained.
- These are measured candidate results, not proof that all remaining failures share that cause. No new action-by-action trace was run for the new Resource Manager cases.

| New Resource Manager failure | Before moving/ledger | After moving/ledger | Before → after camp days | Before → after ending |
|---|---:|---:|---:|---|
| Classic - Resource Manager 1351 | 23/24 | 21/24 | 1 → 1 | boss.victory → boss.victory |
| Classic - Resource Manager 1443 | 24/25 | 22/25 | 3 → 2 | boss.victory → boss.victory |
| Deep - Resource Manager 1391 | 24/24 | 22/25 | 2 → 2 | boss_vote_failed → boss_vote_failed |

Camp counts do not increase in the new Resource Manager failures, so a stronger camp recovery scalar is not supported by these rows. All three still reach the endpoint and the final distance remains 2,100. Their exact action timing and newly visible daily costs need a bounded trace before attributing the additional stationary days to rest, time accounting, or the existing resource policy. A lower average failure count alone is insufficient to select this candidate.

The patch fixes a real completed-action stale-state boundary and retains reward-before-daily-cost ordering, but it remains an isolated, unselected experiment. It does not alter the lazy low-level clock, does not grant driving miles, does not erase genuine hearing/rest days, and does not settle unrelated conversation/care/repair actions. Further common-boundary work needs explicit scope review rather than a tester-order workaround.

## Evidence

- Inputs/commands/hashes: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-inputs.json`
- Complete run output: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-campaign.log`
- Complete rows: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-results.csv`
- All scenarios/failures: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-analysis.json`
- Every numerical guard and paired row/seed change: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-comparison.json`
- Scope/order rationale: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-scope.md`
- Reviewed patch: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity.patch`
