# Seed 1480: Aggressive stationary-day diagnostic

## Scope and reproducibility

Fresh runtime observation of the unmodified real-game plans for seed 1480 in Classic and Deep, using isolated `trace-source` copied from frozen `pace-source`. The only additions are test-only observations in two tester files. Every existing source line, all engine/config inputs, the automated decisions, and all experience guards remain unchanged. No diagnostic hooks were integrated into shared source, preview, or production.

Command: `cargo test -p dystrail-tester trace_aggressive_stationary_days_seed_1480 -- --nocapture`.

Exit 0; 1 diagnostic test passed; 4.521 seconds including compilation. Source fingerprints remained unchanged. The copied executable embeds `/private/tmp/dystrail-retune-2026-09-13/trace-source/dystrail-tester`; SHA256 `586b8a19ab005f8e3f50849ec2d78e7c280e770d67f27d82ceb36f4a63d91dd1`. Cargo was released immediately after the run.

Before interpreting the trace, all **20 observed final metric/result fields** were compared with the immutable release replay CSV at its exact exported precision. All matched. All 10 Classic and 9 Deep decisions matched their exact day, encounter title, label, order, and policy; choice indices were independently matched to the unchanged data. Fields not printed by this bounded diagnostic are not claimed to have been independently re-compared.

| Mode | Moving days | Stationary days | Ratio | Camps | Repairs/detours | Ending |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| Classic | 21 of 24 | 3 | 87.5% | 2 | 0 / 0 | Hearing victory |
| Deep | 20 of 23 | 3 | 86.9565% | 2 | 0 / 0 | Hearing victory |

Both retain the pinned per-run failure against 90%; the diagnostic test does not exempt either run. Classic finishes at 2100.0 internal route miles and Deep at 2056.169677734375. No distance meaning was changed.

## Exact stationary days

These are actual clock and ledger observations. Active hours run 08:00–13:00; unfinished actions resume next morning. Work below is the unchanged three-hour paid cash shift, with +$18 and sanity −1. Shopping spends no time.

| Mode / stationary day | Preceding actions and clock | Camp or hearing | Actual settings / sanity |
| --- | --- | --- | --- |
| Classic day 12 | Conversation day 11 11:00–11:30; work day 11 11:30 → day 12 09:30 | Camp day 12 09:30 → day 13 08:00; 210 active minutes remain when camp starts | Heated / Mixed. Work 4→3; camp 3→9. |
| Classic day 22 | Conversation day 21 12:59 → day 22 08:29; work day 22 08:29–11:29 | Camp day 22 11:29 → day 23 08:00; 91 active minutes remain | Steady / Mixed. Work 4→3; camp 3→9. |
| Classic day 24 | Final driving ends day 23 12:50 at the endpoint | Hearing day 23 12:50 → day 24 09:50, exactly 120 active minutes; final day contains 110 stationary minutes | Heated / Mixed. Hearing 8→2. |
| Deep day 12 | Conversation day 11 11:05–11:35; work day 11 11:35 → day 12 09:35 | Camp day 12 09:35 → day 13 08:00; 205 active minutes remain | Heated / Mixed. Work 4→3; camp 3→9. |
| Deep day 21 | Conversation day 20 12:07–12:37; work day 20 12:37 → day 21 10:37 | Camp day 21 10:37 → day 22 08:00; 143 active minutes remain | Steady / Mixed. Work 4→3; camp 3→10. |
| Deep day 23 | Last drive day 22 12:47–13:00 reaches the existing DA readiness threshold and closes that driving day | Hearing day 23 08:00–10:00, exactly 120 active minutes | Heated / Mixed. Hearing 8→3. |

All four camps are the same existing auto-rest decision: sanity 3, threshold 3, cooldown 0, no pending rest request, no illness. They are not forced encounter rests or hearing-preparation rests. The resulting cooldown is 2. On the day-12 camps and Classic day-22 camp, newly settled Storm −1 offsets daily Mixed +1, leaving +6 net camp recovery. Deep day 21 is Clear; daily Mixed +1 plus camp +6 produces 3→10. HP remains 10.

The terminal hearing days are real, correctly recorded stationary time. The Deep final day is not a phantom ledger entry: the hearing uses two hours after the previous five-hour driving window has ended. Removing that day or inventing movement would misstate the experience.

## Crossings and repeated charging

All three crossings pass in each mode. The first checkpoint is Classic day 5 12:30–13:00 and Deep day 5 12:37 → day 6 08:07. Both consume exactly 30 active minutes, preserve prior mileage, and add no driving fatigue. Deep's seven minutes on the next day do not make that whole day stationary because subsequent driving occurs. The other two crossings also consume exactly 30 minutes, with no detours, repairs, or terminal failures.

Across all **97 observed stationary action boundaries** (Classic 50, Deep 47), the driving-minute total, fractional pace-fatigue remainder, and internal miles remain exactly unchanged. There is no evidence of repeat direct pace charging during camp, work, care, conversation, shopping, or the hearing. The trace observes actual pace switches: Classic 56 Heated / 13 Steady travel-step attempts; Deep 62 Heated / 3 Steady. Every observed diet is Mixed. Initial strategy setup alone would have missed the later Steady periods.

## Smallest supported follow-up

The trace identifies a **daily-settlement ordering concern**, not a supported reason to raise camp recovery again. All four camp decisions occur with `day_initialized=false`, after real active minutes have already been spent on the new day. The auto-rest decision consequently sees the previous day's unsettled diet/weather state. The current engine deliberately creates the new ledger while keeping resource initialization lazy.

The clearest case is Deep day 21: the unchanged check sees sanity 3 and commits to a full camp; that day's still-pending Clear/Mixed settlement would first raise sanity to 4. This is source-and-trace-supported arithmetic, **not a measured counterfactual run**. The other three camp days have Storm −1 offsetting Mixed +1, so their threshold would remain 3 under the same observed daily effects. Earlier settlement can also affect later state/RNG timing; it must be measured globally rather than presumed to fix every seed.

A bounded next model experiment is to settle the current day's existing effects exactly once before a committed positive-time action returns to its caller. Start with the public engine `GameState::perform_activity` boundary, which both UI and tester already use, preserving the low-level lazy `advance_clock` contract. Preserve each action's full minutes, all weather/diet amounts, costs, RNG domains and distributions, policy thresholds, and true stationary ledgers. Moving settlement earlier can change subsequent RNG ordering; this is an explicit measurement risk, not proof of replay parity after the proposed change. Do not modify the tester's choice policy or exempt hearing days. Preserve the 4242 survival guard and rerun all 2000 pinned scenarios after focused ordering regressions. A separate fresh trace is required to claim its effect on the remaining Classic failures.

Relevant frozen-source points:
- `dystrail-game/src/activities.rs:75–104`: applies work/forage effects then advances actual clock; does not initialize the new day.
- `dystrail-game/src/journal.rs:117–145`: overflow records the new stationary day but intentionally leaves daily resource costs lazy.
- `dystrail-game/src/state.rs:3260–3309`: once-only daily initialization includes weather, generic daily effects, and information diet.
- `dystrail-tester/src/logic/simulation.rs:155–180`: town work and auto-rest happen before daily pace/diet initialization.
- `dystrail-game/src/camp.rs:57–118`: existing rest initializes the day only after the recovery decision has been made, then closes the day.
- `dystrail-tester/src/logic/simulation.rs:200–211`: hearing uses exactly 120 active minutes after readiness.

Paths above refer to `/tmp/dystrail-retune-2026-09-13/pace-source`, whose existing production lines are unchanged in `trace-source`; diagnostic insertions shift tester line numbers there.

## UI parity and engine ownership

This section is **read-only current-source analysis**, not a new browser observation. Nine of ten inspected UI/engine files still byte-match the traced frozen source. The one changed file is Conditions copy (`travel_panel/pace.rs`); its existing `day_initialized` locking behavior is retained. Hashes are in `trace-aggressive-1480-ui-owner.json`.

The real player sees the same unsettled post-work state:

1. `dystrail-web/src/app/activities.rs:65–69` snapshots the session and calls the same public `GameState::perform_activity(action)`. The engine changes the clock and returns without initializing the active day. The handler builds its outcome from that returned state, calls `history::record(...,0)` at lines 90–92, then publishes it and saves the session at lines 94–96.
2. `dystrail-web/src/app/history.rs:5–11` passes that zero-minute request to `advance_clock`; its early return does not settle anything. The recorded result therefore uses the already advanced day/time with uninitialized daily resources.
3. `dystrail-web/src/app/view/phases/aftermath.rs:10–22` reads the session and Continue changes presentation/phase only. `WorldView` displays the supplied current day/clock, stats, and `weather_state.today` directly (`components/ui/world_view.rs:40–68`). The new day's HUD can consequently retain yesterday's weather and a pre-diet sanity value. The weather detail can still correctly identify its latest recorded cost as the prior day; that does not make the current strip a settled next-day reading.
4. `dystrail-web/src/app/recovery.rs:76–121` serializes the same session/aftermath state for autosave. Restore/rehydration does not initialize a day, so this is persistent state, not a temporary rendering race.
5. `dystrail-web/src/components/ui/camp_panel/mod.rs:71–104` calculates the offered recovery from those visible stats. Only the Rest callback (`:50–57`) invokes `camp_rest`, whose `start_of_day` then applies the pending daily effects before camp recovery. For the Deep day-21 example, visible sanity 3 becomes 10 with configured +6 camp recovery because daily Mixed +1 is included only after the click.
6. `dystrail-web/src/components/ui/travel_panel/pace.rs:14–16` and `app/view/handlers/travel.rs:97–125` gate pace/diet changes on `day_initialized`. Thus several already spent hours can still present the day as not started. Travel (`handlers/travel.rs:30–32`) eventually settles it, or Camp settles it after the player commits to rest.

The automated runner's **choice to camp immediately at <=3** is tester-specific. There is no corresponding `should_auto_rest` call in web UI source. The stale stats/weather and recovery-preview inconsistency arise from the shared engine action result and are product issues regardless of that automatic choice.

The correct ownership is a **committed timed action transaction in `dystrail-game`**, initially `GameState::perform_activity` (`activities.rs:75–105`), using existing idempotent `start_of_day` (`state.rs:3260–3309`). It should return a coherent current-day result before the UI renders/records it or an unchanged automated policy evaluates it. A screen render, Camp-open, Continue, save/load, or zero-minute journal call must never initialize/pay daily effects. Only real committed time should do that. A morning at 08:00 with no active minutes spent should remain available for existing pre-action settings choices; opening a menu is not authorization to begin the day.

**Keep the low-level clock's tested contract.** The existing `unfinished_work_carries_into_tomorrow_and_daily_costs_apply_once` and `stationary_clock_chunks_record_only_days_with_elapsed_time` tests explicitly assert that raw `advance_clock` can create a ledger while leaving initialization lazy (`travel_time.rs:346–411` in frozen pace source). Do not rewrite those expectations to make a new model pass. Add settlement at the higher committed-action boundary, where no caller should receive time advanced into a day whose effects are still pending. That fixes both callers through one engine contract and permits focused tests for updated HUD-ready results, exactly-once costs, unchanged actual minutes/miles, save/reload stability, and inert navigation. Expand the same transaction boundary to care/encounter/repair/hearing only with explicit outcome-order review; a terminal outcome must not be silently repriced by a generic render-time hook.

## Evidence

- `trace-diagnostic.patch` and `trace-diagnostic-freeze.json`: reviewed additive hooks and all 538 source hashes.
- `trace-aggressive-1480-run.json`: command, exit, timing, log and binary fingerprints.
- `trace-aggressive-1480.log`: complete compiler/test output.
- `trace-aggressive-1480.jsonl`: all 1805 structured observations.
- `trace-aggressive-1480-equivalence.json`: pre-interpretation metric and decision comparisons.
- `trace-aggressive-1480-stationary.json`: exact before/after snapshots for the six stationary days, all crossings, and zero-movement checks.
- `pace-aggressive-1480-results.csv` and `pace-aggressive-1480-verbose.log`: prior immutable release replay.
