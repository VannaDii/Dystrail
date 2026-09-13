# Balanced crossing and early-survival diagnosis

Read-only audit, 2026-09-13. No engine, configuration, strategy, test band, route, clock, build or preview was changed for this analysis.

## Recommendation for the next measurement

Measure only the existing Balanced crossing distribution at **pass 0.70 / detour 0.10 / terminal 0.20**, in place of 0.66 / 0.24 / 0.10. Keep its bribe settings, permits, crossing locations, first-checkpoint escape and detour durations/accounting unchanged. This is one coherent probability distribution with three explicit normalized weights; it avoids changing normalization accidentally through a single unbalanced raw weight.

This candidate has not passed a campaign. It is selected for measurement because the source-traced crossing stream predicts more losses at the second crossing, fewer timed detours, bribe success above 70%, and terminal crossings below both existing aggregate caps. Do not use 0.70 / 0.08 / 0.22: its crossing-only rate is already above Classic's 12% cap.

## Measured campaign evidence

Every cell below comes from the existing analysis JSON/CSV files, with 250 seeds (1337–1586) per mode/strategy. Distance remains the existing campaign metric; the route scale is unchanged.

| Variant | Mode | Mean miles | Survived to day 12 or reached boss | Boss reach | Boss wins | Largest tracked failure family |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | Classic | 2,280.4 | 92.4% | 85.6% | 40.0% | Sanity 56.1% |
| Pace | Classic | 2,280.1 | 92.4% | 85.6% | 46.8% | Crossing 63.2% |
| Pace + health 0.40 | Classic | 2,213.5 | 92.4% | 36.0% | 18.4% | Vehicle 74.8% |
| Health 0.45 | Classic | 2,182.5 | 92.8% | 30.8% | 14.4% | Vehicle 75.1% |
| Baseline | Deep | 2,280.6 | 93.2% | 85.6% | 31.2% | Crossing 52.9% |
| Pace | Deep | 2,280.9 | 92.0% | 85.6% | 31.6% | Crossing 52.9% |
| Pace + health 0.40 | Deep | 2,268.0 | 92.0% | 74.0% | 24.8% | Crossing 48.0% |
| Health 0.45 | Deep | 2,253.4 | 93.2% | 67.2% | 22.4% | Vehicle 44.2% |

The relevant unchanged Classic Balanced bands include distance 1,900–2,100, early survival 60–87%, boss reach 30–50%, boss wins 20–35%, and no tracked failure family over 60%. Bribe success must be at least 70%; terminal crossing events must stay at most 12% Classic / 16% Deep. Per-run moving-day gates remain authoritative even when the mean passes.

Every pre-day-12 loss in these four datasets is a crossing loss. Classic baseline, pace and pace+health0.40 each lose 19/250 before day 12; health0.45 loses 18/250. Deep loses 17, 20, 20 and 17 respectively. Thus the health increases have not created the earlier attrition required by the survival band.

Health-driven `collapse_breakdown` endings in Classic pace+health0.40 occur on days 19–27 (median 24), averaging 2,269.9 miles. At health0.45 they occur on days 17–28 (median 22), averaging 2,228.9 miles. They mostly cut late progress/boss outcomes, rather than the earlier survival horizon. There are 116 and 130 such endings respectively. The tester classifies Breakdown/Vehicle as Vehicle, and Disease/Panic/Hunger as Other; its dominant-family denominator includes Vehicle, Sanity, Exposure and Crossing, not every ending. This explains the reported concentration without changing or relabeling any failure family.

Pace reduces actual nonmoving days (Classic mean 2.40→1.54; Deep 2.524→1.916), but 52 Classic and 75 Deep individual pace runs still miss their moving-day minimum. These records must not be deleted, normalized away or relabeled by the tuning change.

## Source mechanics

- `dystrail-game/src/constants.rs:104`: crossings are at 650, 1,250 and 1,900 miles.
- `dystrail-game/src/state.rs:4272`: each eligible crossing resolves once using its own RNG stream and the prior bribe-attempt count.
- `dystrail-game/src/state.rs:4362`: a terminal result at the first 650-mile checkpoint becomes an alternate-route detour, using the configured maximum duration. It remains a failed bribe, preserves its random draw, and cannot end the run. **Keep this protection.**
- `dystrail-game/src/state.rs:4375`: passage advances 30 minutes; detours advance their real configured hours. Both retain telemetry and day reasons. Later terminal crossings end the run.
- `dystrail-game/src/crossings/resolver.rs:70`: for each bribe, add pass bonus and subtract terminal penalty, each scaled by `1 / (1 + diminishing_returns × prior_attempts)`; normalize the resulting weights. Success means a Pass roll, not just avoiding death.
- `dystrail-game/src/journey/mod.rs:1450`: the crossing stream seed is HMAC-SHA256 of the domain tag with the little-endian user seed. Each crossing consumes one u32. Native `rand` 0.8.5 SmallRng uses the locally inspected PCG seed expansion and Xoshiro256++ draw implementation.
- `dystrail-tester/src/logic/game_tester.rs:1216`: early survival means boss readiness reached **or day ≥12**. This is distinct from finishing the journey alive.
- `dystrail-tester/src/logic/playability.rs:1445`: the crossing rate denominator is all recorded crossing events; first-checkpoint refusals count as detours, not terminal events.

All 730 baseline/pace Balanced crossing events are bribed, and all 250 runs reach the first two crossings. The current 509/730 success rate is 69.726%; 36/730 terminal rate is 4.932%. The 36 losses divide into 20 at the second crossing and 16 at the third. The second-crossing losses occur around days 10–12, providing an existing risk point that can affect the early-survival band without moving the route or removing the first protection.

## Bounded crossing-only replay, not a campaign forecast

`crossing-policy-audit.py` reconstructs native crossing samples and checks each existing pace/Classic Balanced seed against all five recorded counters: event count, bribe attempts, bribe successes, detours and terminal failures. **All 250 per-run comparisons match exactly.** It then evaluates changed thresholds on that same isolated stream. No game loop, browser, compiler or gameplay test was run. JSON results are in `crossing-policy-audit.json`.

| Existing control changed | Bribe successes/events | Terminal/events | Second / third terminal rolls | Detours including protected refusals |
| --- | ---: | ---: | ---: | ---: |
| Current 0.66 / 0.24 / 0.10 | 509/730 = 69.726% | 36/730 = 4.932% | 20 / 16 | 185 |
| Bribe pass bonus only, 0.192→0.22 | 513/730 = 70.274% | 36/730 = 4.932% | 20 / 16 | 181 |
| Raw terminal only, 0.10→0.20 | 463/709 = 65.303% | 67/709 = 9.450% | 41 / 26 | 179 |
| Raw detour only, 0.24→0.08 | 602/725 = 83.034% | 42/725 = 5.793% | 25 / 17 | 81 |
| Move detour to terminal, 0.66 / 0.14 / 0.20 | 489/703 = 69.559% | 76/703 = 10.811% | 47 / 29 | 138 |
| **Recommended 0.70 / 0.10 / 0.20** | **521/703 = 74.111%** | **76/703 = 10.811%** | **47 / 29** | **106** |
| Rejected 0.70 / 0.08 / 0.22 | 515/695 = 74.101% | 85/695 = 12.230% | 55 / 30 | 95 |

The bribe-only control is sufficient for that one band but leaves early attrition unchanged. The terminal-only control damages bribe reliability; reducing detours alone has little attrition effect. The chosen distribution gives a useful bribe margin while moving risk to the already-existing second crossing, with fewer actual detours rather than concealed stop days.

These roll counts **do not predict final campaign distance, day-12 survival, boss reach, wins or family shares**. Changed first-checkpoint refusals can change detour duration, health/camp decisions and later clock timing; earlier losses change who reaches the third crossing and therefore the aggregate denominator. Applying the proposed thresholds only to crossings reached in each old CSV produces terminal rates 10.78–10.86% and bribe rates 73.99–74.11%, but that is still conditional replay of old reach, not a rerun. The first protected refusal can also convert an old short detour into a maximum-duration detour, so reduced total detour count alone does not prove every individual moving ratio improves.

The next full isolated measurement must retain all scenario bands and inspect actual second-crossing day distribution, full/partial/nonmoving records, bribe and terminal denominators, boss outcomes, mean distance, and the complete failure-family mixture. This crossing change can complement a milder late-health control; it is not evidence that the existing health0.40/0.45 candidates now pass, and no such combined candidate is selected here.
