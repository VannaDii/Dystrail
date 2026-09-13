# Individual moving-day timing audit

Read-only audit of the frozen pace-crossing source and complete 2,000-run CSV, 2026-09-13. The parent subsequently authorized an isolated, unbuilt checkpoint-arrival prototype based on this finding. No strategy, test band, route scale, configured crossing weight, shared product source or build was changed by this audit.

## Confirmed ordering defect: checkpoint detection follows the leg that reaches it

`dystrail-game/src/state.rs::compute_miles_for_today` computes a physical leg and converts it to internal distance. `distance_before_next_stop` limits it to the next town and route endpoint, but does not limit it to the next unhandled crossing. In `travel_next_leg`, `handle_crossing_event` tests **existing** miles before the computed leg is credited. A leg can therefore pass the milestone, and the crossing is deferred to a later tick. If that leg exhausts the driving window, the old day is finalized before the crossing is recognized.

This is distinct from the Aggressive 1480 pending-day settlement investigation. The arrival ordering can allow unrelated new-day resource effects, an encounter, town activity, or a camp decision before resolving a checkpoint already crossed geographically. It also creates a possible route to a genuine stationary terminal day after yesterday's crossing approach. The source ordering is proven; the exact amount of the campaign's moving-ratio failure attributable to it requires the isolated prototype/trace, not a CSV-only assertion.

Balanced seed 1364 is a useful case: Classic ends at 1,261.9 internal miles on day 11; Deep at 1,260.0. Both have 9 moving + 2 nonmoving days, one recorded camp and a terminal crossing record. The second crossing's actual milestone is 1,250. These records retain all elapsed time; they should not be fixed by omitting the final day or inventing movement.

The bounded engine correction to measure is to stop the leg at the earliest town, endpoint or crossing and resolve a crossing on actual arrival before finishing that driving day. A pass or detour still spends its actual stationary minutes, including any real overnight spill. A terminal crossing retains the actual approach miles/time already driven that day. Existing already-reached milestones need a safe path with no rewind or fabricated extra mileage. First-checkpoint denial remains an alternate-route detour, and every crossing retains one random draw and unchanged bribe/permit rules.

The physical/internal conversion can round below 650/1,250/1,900 at f32 precision. Carry the exact internal limiting distance when that boundary wins, rather than using a broad epsilon or a second tiny leg. Cover all route personas and speed caps as well as priority relative to towns and the route endpoint.

## Real stationary time remains part of the challenge

- In the measured crossing candidate, 40 Classic Balanced and 63 Deep Balanced runs miss 90%. Their causes are Classic: 19 victories, 14 boss losses, 4 crossing failures, 3 sanity losses; Deep: 27 victories, 27 boss losses, 7 crossing failures, 2 sanity losses. Fixing checkpoint timing alone cannot be assumed to fix long-run failures.
- 18/40 Classic and 19/63 Deep failing Balanced runs have more nonmoving days than camp-tagged days. The tags/counts alone cannot identify each untagged action: the CSV does not contain every day's clock, miles and full action list.
- `camp_rest` intentionally records a real stationary rest and ends the day. `day_accounting::record_travel_day` explicitly preserves driving already completed earlier in that day. The source does not erase those miles or downgrade that already-moving day simply because camp is selected.
- The unchanged tester's actions have actual costs: local conversation 30 minutes, town shift 180, forage/glean 120, crew care 60, repair its specific duration, encounter decision 30, boss hearing 120. These can use an entire driving window or leave a genuine final stationary hearing day. They must not be relabeled as driving.
- Long-run example: Deep Balanced seed 1359 reaches 2,400 miles and loses the boss vote on day 27 with 21 full + 1 partial + 5 nonmoving days. Its history includes five camps, a protected checkpoint denial and later endgame actions. This remains a recovery/timing case for diagnosis; it is not evidence of five falsely counted days.

The complete measured report, all failed seeds, source hashes and deterministic compressed CSV are preserved beside this audit. The prototype needs its own focused engine checks and full campaign measurement before any acceptance claim.
