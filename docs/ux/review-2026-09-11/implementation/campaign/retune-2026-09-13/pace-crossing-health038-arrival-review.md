# Checkpoint arrival candidate

Status: isolated and frozen for parent review. Formatting passes; no compilation, native test, browser, campaign, preview, or deployment claim is made.

Original task: cap travel at the next unhandled crossing and resolve it on arrival before ending the driving day, retaining true miles/time and every existing cost, consequence, choice, random weight, and protected first-checkpoint detour.

## Scope

Base: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-source`.
Candidate: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-arrival-source`.
Only `dystrail-game/src/state.rs` changes, plus the new `state/crossing_arrival_tests.rs`. All 538 base files remain unchanged in the base directory. No config, strategy, established test, route data, UI, shared product source, or acceptance threshold is edited.

The distance limiter chooses the earliest town, next unresolved crossing, or route endpoint. When the crossing or endpoint limits the drive, its exact internal distance is carried alongside physical miles rather than converted back with rounding. Physical miles still determine the rounded-up driving minute budget. No tolerance is enlarged and no stationary miles are credited.

The existing pre-drive crossing handler stays available for already-reached legacy milestones. A second check runs after credited movement and driving time, after the existing health-failure guard, before day closure. Existing encounter, breakdown, and endgame priority is unchanged. The crossing handler settles a nonterminal result when its actual waiting/detour cost exactly exhausts the day; spilling work still uses the existing clock rollover. Terminal results retain the actual approach in the open day rather than moving the failure to a later empty day.

## Focused contracts

Seven focused tests contain 81 scenario variants:

- Mid-hour pass, detour, and terminal outcomes in both modes: exact approach miles, actual drive minutes, unchanged stats/wear, exact bribe charge, waiting/detour duration, one random draw, and no repeated charge or resolution on continuation.
- Final-minute arrival for all six physical routes, all three pace caps, and all three milestones: exact internal boundary, one real driving minute, no overshoot or deferred arrival, old-day approach record, and only genuine stationary time on the next day.
- Pass/detour costs finishing exactly at the day boundary: settle once, retain only driven miles, and leave no fabricated next-day record.
- Final-minute terminal arrival in both modes at each lethal milestone: preserve that day's real approach and terminal consequence without a later empty travel attempt.
- First-checkpoint terminal roll still becomes the existing alternate-route detour.
- Already-reached and slightly overshot legacy milestones resolve without rewinding or adding driving.
- Earlier towns and a route endpoint before the next crossing win the distance limit on all six routes.

These tests deliberately force outcomes only inside their fixtures; shipped resolver weights, bribe modifiers, and first-checkpoint protection are untouched.

## Validation and remaining work

Ran standalone `rustfmt --edition 2024 dystrail-game/src/state.rs`, then the same command with `--check`; both exit 0. Verified the complete candidate hash set, unchanged base hashes, and a two-file allowlist. Parent review precedes compilation. Focused engine tests, existing crossing/fractional-distance checks, strict Clippy, seed 4242, and the unchanged 2,000-run experience campaign remain required. Browser/preview validation follows only if the candidate is selected.

## Completed validation follow-up

Parent review approved the initial patch and the exact-distance encounter-ride caller adaptation. Read `pace-crossing-health038-arrival-measurement.md` for all completed tests, retained failures, the campaign comparison, and the separately scoped encounter-choice clock limitation. This original review is retained as preparation history; its initial unbuilt status is superseded only by that dated evidence.
