# Hourly balance measurements — September 13, 2026

These are frozen experiments, not accepted gameplay changes. Production and preview remain on the verified visual release; the narrative-only release is handled independently. All strategy definitions, route scales, test thresholds, seed lists and clock/camp durations are unchanged.

| Candidate | Mode | Mean days | Mean internal miles | Reach | Wins | Survival | Moving-day share | Runs below 90% |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| baseline | Classic - Balanced | 24.244 | 2280.374 | 85.6% | 40.0% | 92.4% | 90.48% | 128 |
| baseline | Deep - Balanced | 24.428 | 2280.645 | 85.6% | 31.2% | 93.2% | 90.06% | 134 |
| health035 | Classic - Balanced | 24.068 | 2275.956 | 83.2% | 40.8% | 92.4% | 92.42% | 80 |
| health035 | Deep - Balanced | 24.320 | 2279.109 | 84.8% | 26.8% | 93.2% | 90.61% | 117 |
| health055 | Classic - Balanced | 18.320 | 1891.041 | 0.0% | 0.0% | 92.4% | 93.04% | 52 |
| health055 | Deep - Balanced | 19.952 | 2006.216 | 0.4% | 0.0% | 93.2% | 92.80% | 41 |

Every candidate completed all 2,000 runs and the original CLI exited1 for unmet acceptance. Seed 4242 passes the retained 20-day minimum in each candidate. The baseline has 477 individual moving-ratio failures across all eight scenarios. Health 0.35 is insufficient: reach and distance remain too high. Health 0.55 overcorrects, eliminating almost all hearing reaches and producing a dominant health-collapse family; it also leaves the early-survival ceiling unchanged. Health 0.45 is queued as a bracket measurement, not assumed acceptable.

Inputs/binaries/outputs and after-run source hashes are recorded in each*-inputs.json. Pace fatigue tied to real driving minutes is a separate frozen prototype pending validation. No golden digest or experience assertion has been changed.

## Verified pace-only improvement

The first release invocation accidentally reused the health045 executable because the queued snapshot's source mtimes predated that build. This was caught by identical binary hashes and a 0.09-second no-compile log. That attempt is preserved under `pace-stale-build-attempt/` and is invalid as a pace measurement. The runner now touches each immutable source immediately before compiling and asserts the binary embeds its expected source path. Every earlier baseline/health binary also passes that explicit path check.

The fresh pace-only binary is `2fcacf5b954ec8eb7a6fcd840ecdccfc5168434a48824191b5b797a7b6034e8d`. It reduces moving-ratio failures 477→207 (Classic Aggressive 40, Classic Balanced 52, Deep Aggressive 40, Deep Balanced 75), clears all Resource Manager runs and all aggregate movement-rate guards, and preserves all eight focused driving-fatigue regressions plus seed4242. The full native suite passes335 and retains only the same three config/replay fingerprint failures; repeated-run equality still passes. No test assertion was edited.

The pace+health040 measurement lowers Classic Balanced reach to 36% but wins are 18.4%, mean distance 2213.486, survival 92.4%, and the dominant health-collapse family 74.84%. Deep Balanced mean distance 2267.984 also fails. All2,000 rows and source/binary/output hashes are retained. This combination is unselected.

The separate camp+8 experiment does not improve the per-run failure count (207→208), so it is rejected; it must not be integrated merely because mean camp frequency falls slightly. No extra Resource Manager or stationary-day accounting change is justified.
