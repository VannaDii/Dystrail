# Resource Manager Activity-settlement diagnosis

The two Resource Manager regressions reflect real timing changes. **No ledger reset, lost earned mileage, or free stationary-action movement was observed.** The same illness events now become visible before the camp decision, so the unchanged policy rests immediately instead of first driving an hour. Additional activity/hearing time accounts for the other newly stationary days.

## Parity and scope

- Before source: `pace-crossing-health038-source`; after source: `pace-crossing-health038-activity-source`. Exactly the previously measured source variants were copied, with identical test-only instrumentation in two tester modules and no existing lines removed or changed.
- Both immutable reference replays match all 48 fields in all 16 original campaign rows for the two numeric seeds. Both instrumented traces independently match all 48 fields for the selected Classic RM 1351 and Deep RM 1391 cases, plus every encounter decision in day/title/label/order/policy. Choice IDs/indices match unchanged game data (7 Classic decisions, 10 Deep decisions per variant).
- Both diagnostic runs exit 0. Immutable reference CLI runs exit 1 because their small matrix retains the same acceptance failures; that output is preserved, not treated as a parity failure.
- The first before diagnostic did not compile because five extra JSON fields exceeded the existing macro recursion limit. Its original patch/freeze/log are preserved in `trace-rm-json-recursion-attempt/`. Splitting the same distance observations into a nested JSON value fixed only the instrumentation. No suppression or test expectation was added.

| Variant | Diagnostic seconds | Immutable diagnostic SHA256 |
|---|---:|---|
| before | 3.129 | `5ed4d4bdc2d8c100a6a3310eb63cde827da321da841fa9401c88504c47d352c0` |
| after | 4.652 | `562beb685b2a3f76ffdb5490c0f3c00d18445e76ae32d28bd11eb858c2997f31` |

Current identical instrumentation patch SHA256: `d12802f2bceec787c00b30e12932b202938fcb3be55be0e3c5c5f3a4cb5c9f4f`. All 538 input hashes remain unchanged after each successful compile and analysis. Cargo has been released; these hooks are not integrated in the shared tester.

## Exact failed-day accounting

| Case | Before moving/total; stationary days | After moving/total; stationary days |
|---|---|---|
| Classic-1351 | 23/24; [24] | 21/24; [14, 20, 24] |
| Deep-1391 | 24/24; [] | 22/25; [6, 16, 25] |

All observed travel-step settings in these four traces are **Heated / Quiet**, despite the initial strategy setting being Steady. The camp decisions below are illness requests, not low-sanity auto-rest: sanity is 8–10, the automatic-rest threshold is 5, and `should_auto_rest` is false. Increasing camp sanity recovery therefore does not address these events.

| Illness day | Baseline: one driving leg before the same camp | Candidate: immediate camp | Illness at camp |
|---|---|---|---|
| Classic-1351, day 14 | 09:40–10:40: 60 driving minutes / 42.776245 internal miles; camp at 10:40 | Camp at 09:40; 0 driving minutes that day | 3 illness days remaining; rest_requested=true; sanity 9 → 9 |
| Deep-1391, day 6 | 10:10–11:10: 60 driving minutes / 42.776245 internal miles; camp at 11:10 | Camp at 10:10; 0 driving minutes that day | 3 illness days remaining; rest_requested=true; sanity 10 → 10 |
| Deep-1391, day 16 | 09:03–10:03: 60 driving minutes / 36.992920 internal miles; camp at 10:03 | Camp at 10:06; 0 driving minutes that day | 2 illness days remaining; rest_requested=true; sanity 8 → 9 |

The illness hit is the same calendar day in each paired case. In the baseline, the simulator checks camp, then applies pace/diet (which initializes the day and rolls illness), then travels once. On the next loop it sees the pending illness-rest request. In the candidate, overnight work has already initialized that day before returning; the first camp check sees the same request and acts immediately. The earlier one-hour drive was real and correctly recorded in the baseline, but was allowed by stale decision timing—not erased by the candidate.

Classic 1351 day 20 is a separate full active-day schedule, with no camp:

| Action | Time | Active minutes | Mileage/driving change |
|---|---|---:|---|
| crew_care | 08:00–09:00 | 60 | 0.0 miles / 0 driving minutes |
| town_conversation | 09:00–09:30 | 30 | 0.0 miles / 0 driving minutes |
| town_work | 09:30–12:30 | 180 | 0.0 miles / 0 driving minutes |
| encounter | 12:30–13:00 | 30 | 0.0 miles / 0 driving minutes |

Those 300 minutes fill 08:00–13:00 exactly. The encounter is `sat_bridge_bullets`, choice 2, “Wish them luck and take the reopened lane.” Travel begins on day 21. The baseline had reached this town on day 19 and moved its crew-care interaction to day 21, so day 20 retained driving time. This is changed arrival/action timing, not a missing ledger row.

Deep 1391’s final hearing crosses midnight in the candidate’s active-day model:

| Variant | Hearing time | Active minutes | Last-day stationary portion |
|---|---|---:|---:|
| before | Day 24 10:24 → day 24 12:24 | 120 | 0 min |
| after | Day 24 11:18 → day 25 08:18 | 120 | 18 min |

Both hearings cost exactly 120 active minutes, with no driving. The candidate’s 18 minutes on day 25 are therefore a real stationary day and must remain counted. Classic 1351’s day-24 hearing is stationary in both variants.

## Ledger and clock conservation

| Variant/case | Stationary action pairs checked | Maximum recorded-mile conservation error | Lost/decreased earned-day records |
|---|---:|---:|---:|
| before Classic-1351 | 38 | 0.000141144 mi | 0 |
| after Classic-1351 | 38 | 0.000282288 mi | 0 |
| before Deep-1391 | 37 | 0.000062943 mi | 0 |
| after Deep-1391 | 37 | 0.000070572 mi | 0 |

Across **150 stationary action pairs**, actual miles, total driving minutes, and fractional driving-fatigue remainder remain exactly unchanged. No cumulative mileage or driving counter decreases. For every observed day, its largest recorded mileage is retained in the final ledger; no same-day mileage reset occurs. Summing the final per-day f32 mileage differs from cumulative mileage only by ≤0.000283 internal miles. The conservation diagnostic uses a 0.001-mile floating-point comparison tolerance; no acceptance threshold or production value changes.

## Production implications

The Activity correction makes the completed-action state truthful even when the retained movement guard becomes harder to satisfy. A ledger patch, an extra pre-rest drive, deferring illness again, or changing the bot would conceal these real costs. Camp +8 would not resolve these illness requests because recovery strength is not their trigger.

Existing illness controls are `DISEASE_DAILY_CHANCE=0.012`, `DISEASE_COOLDOWN_DAYS=5`, duration 2–4 days, and a 0.85 travel multiplier. Those are genuine model controls if a later isolated recovery-frequency experiment is approved; this diagnosis selects no value. Classic’s day-20 five-hour workload and the final hearing remain real time obligations even if illness frequency changes. These two seeds do not justify a global guarantee about all remaining failed runs.

Relevant unchanged source owners:

- `dystrail-game/src/activities.rs:75–114` in the Activity candidate: completed timed-activity boundary.
- `dystrail-game/src/state.rs:3260`: once-only daily initialization; `:3892–3952`: daily illness and explicit rest request.
- `dystrail-game/src/constants.rs:145–155`: illness frequency, cooldown, duration, and penalties.
- `dystrail-game/src/camp.rs:57–114`: full rest and illness recovery.
- `dystrail-tester/src/logic/simulation.rs:138–186` in the uninstrumented base: camp check before lazy daily initialization; `:242–258`: existing camp policy; `:203–204`: hearing’s 120-minute clock; `:260–295`: encounter and crew-care costs.
- `dystrail-game/src/journal.rs:117`: unchanged low-level active-time accounting.

All referenced source paths resolve under `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-source`, except the Activity boundary under its `-activity-source` sibling. Full immutable before/after references, all day snapshots, exact per-action values and machine-checked conservation results are preserved alongside this report.
