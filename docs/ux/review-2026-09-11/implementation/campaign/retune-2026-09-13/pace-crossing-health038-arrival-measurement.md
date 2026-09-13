# Checkpoint-arrival measurement

The exact-arrival mechanics pass the focused tests, but the unchanged 2,000-run campaign still fails six numeric guard categories. This is a verified timing correction and an unaccepted balance candidate. No shared source, preview, production, strategy, or guard was changed.

## Proven changes

The cap now includes the next unhandled crossing, carries the exact internal crossing/endpoint distance through physical conversion, and resolves normal driving arrivals after real movement/time but before day closure. A pass or detour that exactly exhausts the day settles once. Legacy already-reached crossings remain supported. First-checkpoint terminal rolls still become the existing alternate-route detour.

Only state.rs and its new test module differ from the health 0.38 + pace + crossing .70/.10/.20 base. Camp remains 6. The common distance limiter also serves encounter rides; that caller now consumes the exact internal distance. Its crossing resolution continues through the pending path, as explicitly approved for this bounded candidate.

## Validation

- Seven new arrival tests pass, covering 81 variants including all routes/paces/milestones, mid/final-minute arrival, real costs, terminal/first-checkpoint consequences, no repeated crossing charge/draw, pending milestones, towns and endpoints.
- The unchanged full game-library regression set passes 196/196. After the later test-only iterator cleanup, all seven arrival tests pass again.
- Strict production-library Clippy passes; strict Clippy including tests has only four inherited pace-fixture initializer errors in travel_time.rs (78, 119, 163, 249). Root already has a checked style-only patch for those exact lines; it was deliberately not mixed into this measurement. No lint suppression was added.
- Final rustfmt check passes. The first failed compile (second tuple-return caller) and first Clippy failure (new-test iterator plus inherited errors) are retained.
- Unchanged seed 4242 test passes. Release build passes, canonical embedded source path and binary SHA are verified. All 539 candidate inputs remain exact after the run.
- Campaign executes all 2,000 records with exit 1 in 13.285 seconds, stopping acceptance at a Classic Balanced individual moving-day ratio of 88%. Later native aggregate/determinism/consistency validation was not reached. The full numeric comparison below is a separate CSV audit against the unchanged guard code, not a claim those native validators passed.

## Before and after

| Scenario | Mean internal miles | Ratio-failing runs | Numeric failures remaining |
| --- | ---: | ---: | --- |
| Classic - Aggressive | 2086.2660 → 2085.8000 | 40 → 51 | each-run moving-day ratio |
| Classic - Balanced | 2104.9396 → 2100.7124 | 9 → 7 | each-run moving-day ratio, mean internal campaign distance |
| Classic - Conservative | 2095.7012 → 2095.2000 | 0 → 0 | None |
| Classic - Resource Manager | 2100.0000 → 2100.0000 | 0 → 0 | None |
| Deep - Aggressive | 2050.1756 → 2049.6248 | 40 → 42 | each-run moving-day ratio |
| Deep - Balanced | 2131.7248 → 2122.3948 | 36 → 23 | each-run moving-day ratio, mean internal campaign distance |
| Deep - Conservative | 2100.0000 → 2100.0000 | 0 → 0 | None |
| Deep - Resource Manager | 2100.0000 → 2100.0000 | 0 → 0 | None |

Total individual ratio failures decrease only 125 → 123. Classic Balanced fixes all nine previously failing seeds but introduces seven different ones. Deep Balanced fixes 29, retains seven, and introduces 16. Counterevidence is material: Classic Aggressive rises 40 → 51 and Deep Aggressive 40 → 42. No numeric guard category is removed or added. All four Conservative/Resource Manager groups remain within every numeric guard.

Classic Balanced now has 42.8% boss reach, 24.0% wins, 81.6% survival-or-12-day duration, and 52.05% dominant tracked failure share. Each remains within its existing band. Both Balanced modes retain exactly 521 successful bribes out of 703 attempts (74.11%) and 76 terminal crossings out of 703 events (10.81%), within the 70% success floor and 12% Classic/16% Deep terminal caps. All 500 Balanced per-seed crossing-count records are unchanged; timing/physical progress and subsequent campaign decisions account for the observed changes.

Both Balanced mean distances still exceed 2,100: Classic by 0.7124 internal miles, Deep by 22.3948. Every scenario meets the existing mean ratio, 8–30-day duration, 100–350 physical-mile/day, bribe and terminal-crossing bounds. Scenario-specific encounter diversity, milestone, boss and failure-family checks have no additional numeric failures. The minimum-per-run moving-day guard remains failed in both Aggressive and both Balanced groups.

## Concrete correction and remaining limitation

Balanced seed 1364 now fails at exactly internal mile 1250 on day 10 in both modes. Previously it overshot to 1261.9 Classic / 1260.0 Deep and failed on day 11. The same terminal consequence and one camp remain; the former second nonmoving day disappears because the crossing resolves on arrival. Its ratio becomes 9/10 rather than 9/11. This supports the timing defect without claiming every stationary day was false.

Encounter rides still need a separate clock-ownership follow-up. Both browser history::record and tester resolve_encounter call advance_clock(&before_choice, 30) after apply_choice(). Resolving a crossing inside apply_encounter_ride before that call could overwrite the crossing wait or rollover. The bounded candidate adapts exact distance only and retains pending handling; no crossing or choice cost is erased.

The remaining failed individual runs must be examined as actual fatigue/recovery and hearing time, with the original ledger and guard definitions intact. Arrival timing alone does not satisfy all experience requirements.

## Frozen artifacts

- Source: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-arrival-source`
- Tree SHA-256: `50ce05dc9e7ab74fee9ce698f749017c06677eba8c360309d9cccb816d159b04`
- Patch SHA-256: `a327e2c0fa9b491dbea5d0b92f36df0c0e62b2326effcb84e8072fbd514ba7dc`
- Binary SHA-256: `192dbddc7ea44cbfa8c4dabd7155ffe93e7412593a3c1bfc4edad3b72938847b`
- Compiled source path: `/private/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-arrival-source/dystrail-tester`
- Detailed paired metrics, introduced/fixed failure seeds, crossing-count differences, and source/output hashes: companion comparison JSON.
