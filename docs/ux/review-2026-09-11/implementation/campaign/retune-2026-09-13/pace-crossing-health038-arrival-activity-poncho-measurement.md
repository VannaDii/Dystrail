# Combined arrival, Activity and poncho diagnostic

The combined candidate passes its code regressions but fails campaign acceptance. Across all 2,000 records, individual moving-day failures fall from 123 to 101 compared with arrival alone, while the number of failing numeric guard categories increases from six to nine. This does not establish acceptance or integration of the Activity change.

## Immutable scope and checks

The final arrival candidate is the base. Exactly four additional files contain the previously reviewed Activity settlement plus tests, the two-file poncho fix, and four test-only pace initializer cleanups. Balanced health decay stays 0.38, crossing weights stay 0.70/0.10/0.20, camp sanity stays 6, and all tester strategies/guards remain byte-identical. No shared source, preview or production is changed.

- 203 game-library tests pass, including every new arrival/Activity contract and existing clock/pace/crossing/ride test.
- All five weather integration tests pass, retaining unprotected Storm loss and unaffected weather/resource/speed/chance behavior.
- Strict game Clippy including tests passes with no suppressions. The four inherited initializer errors are resolved by the exact checked root test-only changes.
- Standalone formatting, the unchanged seed 4242 test, and release build pass.
- Canonical embedded source path and copied binary hash verify; all 539 input hashes remain exact after the campaign. Cargo was released before executing the copied binary.
- All eight scenario groups execute 250 seeds each, 1337–1586. Campaign exit 1 in 12.746 seconds, first failure Classic Balanced individual travel ratio 88%. Later native acceptance validators short-circuit; the remaining numeric results below are independently audited from the complete CSV, not claimed native passes.

## Compared with arrival alone

| Scenario | Mean internal miles | Ratio-failing runs | Remaining numeric failures |
| --- | ---: | ---: | --- |
| Classic - Aggressive | 2085.8000 → 2075.6000 | 51 → 37 | each-run moving-day ratio |
| Classic - Balanced | 2100.7124 → 2100.0656 | 7 → 12 | each-run moving-day ratio, mean internal campaign distance |
| Classic - Conservative | 2095.2000 → 2095.2000 | 0 → 1 | each-run moving-day ratio |
| Classic - Resource Manager | 2100.0000 → 2100.0000 | 0 → 0 | None |
| Deep - Aggressive | 2049.6248 → 2059.7456 | 42 → 26 | each-run moving-day ratio |
| Deep - Balanced | 2122.3948 → 2123.1192 | 23 → 22 | each-run moving-day ratio, mean internal campaign distance |
| Deep - Conservative | 2100.0000 → 2100.0000 | 0 → 1 | each-run moving-day ratio |
| Deep - Resource Manager | 2100.0000 → 2100.0000 | 0 → 2 | each-run moving-day ratio |

The six prior failure categories persist. Three new categories are individual moving-day failures in Classic Conservative, Deep Conservative, and Deep Resource Manager. Classic Resource Manager remains within all numeric guards. Classic Balanced has five more failing runs (7 → 12); both Aggressive groups improve in count, but Classic Aggressive’s worst ratio deteriorates from 84.0% to 81.48%. These counterexamples are retained in the comparison JSON.

Classic Balanced remains within its boss/attrition/failure-family bands: 43.2% boss reach, 22.0% wins, 81.6% survival-or-12-day duration, and 48.10% dominant tracked failure share. Balanced crossing results stay 521/703 successful bribes (74.11%) and 76/703 terminal crossings (10.81%) in both modes. Every scenario passes its mean ratio, 8–30-day duration, 100–350 physical-mile/day, bribe and terminal-crossing bounds. Encounter diversity, milestone, and the remaining boss/scenario-specific numeric checks add no failures.

The two distance failures remain: Classic Balanced 2100.0656 and Deep Balanced 2123.1192 exceed the unchanged 2100 ceiling. The small Classic excess is still a failure; no rounding exception is applied.

## New concrete counterexamples

| Scenario / seed | Moving / total days | Camp days | Outcome |
| --- | ---: | ---: | --- |
| Classic - Conservative / 1374 | 25/28 | 2 | boss_vote_failed |
| Deep - Conservative / 1516 | 25/28 | 2 | boss.victory |
| Deep - Resource Manager / 1391 | 22/25 | 2 | boss_vote_failed |
| Deep - Resource Manager / 1499 | 21/24 | 2 | boss_vote_failed |

Each of these runs has two camp days and three genuinely recorded nonmoving days. That establishes the elapsed-day burden; the CSV alone does not establish the exact action that owns the third stationary day. A full per-action trace is required before changing any clock or ledger behavior. Three nonmoving days in 24–28 days fail a 90% minimum even though average travel remains high. The next balance decision should address measured fatigue/recovery burden, preserving actual waiting, rest, repair and hearing time.

This combined comparison cannot attribute every change separately to Activity versus ponchos: altered daily settlement and gear effects change subsequent decisions and random-event exposure. Companion comparisons against the original health-0.38 base and Activity-only measurement are included as evidence, not an isolated causal estimate. The separately documented encounter-ride choice-clock limitation remains: do not resolve its crossing before the caller settles the 30-minute choice cost.

## Artifacts

- Source: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-arrival-activity-poncho-source`
- Tree SHA-256: `a2626beef408903e3d5672de1e81dbda6ce75801ca98fc9a48fce1e58e04e267`
- Patch SHA-256: `11d6782e2f6bccdb01375dcd354a5ae4a80896e9859b568e90b602ddf05731c2`
- Tester SHA-256: `8766f1f155c8d333ab90de9e629773e42a2859b1a7b7249c8824f330987049eb`
- Embedded path: `/private/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-arrival-activity-poncho-source/dystrail-tester`
- Full metrics, fixed/introduced/retained seed sets, new counterexamples, and input/output hashes are in the companion comparison and inputs JSON.
