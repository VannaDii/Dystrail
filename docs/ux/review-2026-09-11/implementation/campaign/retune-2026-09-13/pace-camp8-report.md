# Pace plus camp +8: rejected measurement

Recorded 2026-09-13T07:57:03.016989+00:00.

The bounded change is only `camp.rest.sanity: 6 → 8` in an isolated copy of the verified pace-source. Balanced daily health decay remains 0.12. Rest duration (one day), supply cost (one), HP recovery (one), cooldown (two days), actual hours/miles, diet, bot decisions, seeds, and every experience threshold remain unchanged.

**Recommendation: reject +8 and retain +6.** The every-run movement failures increase from 207 to 208. The change improves average ratios slightly but fails to improve the pinned per-run experience.

## Validation and identity

- Numeric seed 4242 with the unchanged >=20-day survival assertion: exit 0 (6.968 seconds).
- Release tester build: exit 0 (7.922 seconds).
- Full campaign: exit 1 after all 2,000 rows (12.728 seconds). Every one of eight scenarios retains seeds 1337–1586.
- Immutable tester SHA-256: `721b3c7861f01d3de6e4e3d911c061c9bf43e3803df7d3fe15acbf31924fec83`.
- Compiled source path verified inside binary: `/private/tmp/dystrail-retune-2026-09-13/pace-camp8-source/dystrail-tester`.
- All 538 input hashes match before and after the run; exactly one input differs from pace-source. Shared source and live preview were not changed.

## Paired outcomes

| Scenario | Ratio failures +6 → +8 | Mean moving ratio +6 → +8 | Mean camps +6 → +8 | Mean stationary days +6 → +8 |
|---|---:|---:|---:|---:|
| Classic - Aggressive | 40 → 40 | 93.2567% → 93.3263% | 2.012 → 1.932 | 1.652 → 1.636 |
| Classic - Balanced | 52 → 52 | 93.6524% → 93.6691% | 2.108 → 2.108 | 1.540 → 1.536 |
| Classic - Conservative | 0 → 0 | 98.5707% → 98.5707% | 0.416 → 0.416 | 0.388 → 0.388 |
| Classic - Resource Manager | 0 → 0 | 98.3843% → 98.3843% | 0.376 → 0.376 | 0.384 → 0.384 |
| Deep - Aggressive | 40 → 42 | 92.9604% → 93.0017% | 2.120 → 2.064 | 1.668 → 1.656 |
| Deep - Balanced | 75 → 74 | 92.2739% → 92.3355% | 2.632 → 2.628 | 1.916 → 1.900 |
| Deep - Conservative | 0 → 0 | 98.3830% → 98.3830% | 0.428 → 0.428 | 0.440 → 0.440 |
| Deep - Resource Manager | 0 → 0 | 98.3170% → 98.3170% | 0.424 → 0.424 | 0.400 → 0.400 |

199 rows differ; 1,801 rows are exactly unchanged. Twelve formerly failing rows pass, 13 formerly passing rows fail, and 195 failures persist. Every Conservative and Resource Manager row is identical. Each Balanced mode changes only one row. Detailed paired fields and seed lists are preserved in the comparison JSON.

## Every remaining hard guard

| Scenario | Guard | Observed +8 | Required |
|---|---|---:|---|
| Classic - Aggressive | each-run moving-day ratio | 40 failing runs; minimum 0.840000 | ≥ 0.9 |
| Classic - Balanced | each-run moving-day ratio | 52 failing runs; minimum 0.840000 | ≥ 0.9 |
| Classic - Balanced | mean internal campaign distance | 2280.085200000 | ≥ 1900 and ≤ 2100 |
| Classic - Balanced | crossing bribe success | 0.697260274 | ≥ 0.7 |
| Classic - Balanced | boss reach | 0.856000000 | ≥ 0.3 and ≤ 0.5 |
| Classic - Balanced | boss wins | 0.468000000 | ≥ 0.2 and ≤ 0.35 |
| Classic - Balanced | survival or 12-day run | 0.924000000 | ≥ 0.6 and ≤ 0.87 |
| Classic - Balanced | dominant tracked failure family | 0.631578947 | ≤ 0.6 |
| Deep - Aggressive | each-run moving-day ratio | 42 failing runs; minimum 0.846154 | ≥ 0.9 |
| Deep - Balanced | each-run moving-day ratio | 74 failing runs; minimum 0.821429 | ≥ 0.9 |
| Deep - Balanced | mean internal campaign distance | 2280.859200000 | ≥ 1900 and ≤ 2100 |
| Deep - Balanced | crossing bribe success | 0.697260274 | ≥ 0.7 |

All aggregate moving-ratio checks pass. Mean duration, physical miles per moving day, terminal crossing rates, pinned encounter variety, relevant distance milestones, and Deep Aggressive reach/win/distance checks still pass. Resource Manager and Conservative have no per-run moving-ratio failures.

Classic Balanced remains at 85.6% hearing reach, 46.8% wins, 92.4% survival, and 63.1579% dominant tracked failure family (36 crossing endings among 57 tracked crossing/sanity endings). Its mean distance is 2280.0852; Deep Balanced is 2280.8592. Both Balanced bribe rates remain 509/730 = 69.7260%. These failures are preserved.

Deep Aggressive wins increase from 72.4% to 75.2%, while its per-run movement failures increase from 40 to 42. Thus stronger recovery can change route timing and ending outcomes without addressing the actual stationary-day cause. Additional recovery scalars are not justified until exact action/clock traces identify that cause.

## Commands and evidence

Commands are recorded verbatim with exit statuses and durations in `pace-camp8-inputs.json`. The executed orchestrator was `python3 /tmp/dystrail-retune-2026-09-13/run_candidate.py pace-camp8`; it verifies input hashes, refreshes source mtimes before compiling, runs seed 4242 first, copies the release tester, verifies its embedded source path, and runs the unchanged 2,000-run acceptance matrix.

- `pace-camp8-results.csv`: all raw rows.
- `pace-camp8-analysis.json`: all scenario metrics and failed guards.
- `pace-camp8-comparison.json`: exact paired field differences, resolved/new/persistent failures.
- `pace-camp8-inputs.json`: all source and output hashes, binary identity, commands.
- `pace-camp8-seed4242.log`, `pace-camp8-build.log`, `pace-camp8-campaign.log`: full test/build/run output.
