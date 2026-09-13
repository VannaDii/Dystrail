# Completed Activity settlement candidate

Prepared source: `/tmp/dystrail-retune-2026-09-13/pace-crossing-health038-activity-source`.
Base: immutable `pace-crossing-health038-source`; all 538 base input hashes verified before copying. All 537 other inputs remain byte-identical. Only `dystrail-game/src/activities.rs` changes, by additive lines; every existing production/test line is retained verbatim. No shared-source, config, strategy, acceptance, snapshot, preview, or production writes.

Production change: after successful `perform_activity` has applied its existing outcome and called its existing `advance_clock`, call idempotent `start_of_day` only if `self.day > before.day` and `clock_minutes > TRAVEL_DAY_START`. This returns a settled new-day state after real overnight work or gathering. It does not move the activity's effects before/after any existing statements, charge extra action time, remove stationary ledger days, or credit miles/fatigue. The low-level clock stays lazy. Action guards, costs, rewards, durations, cooldown values, and metadata update order remain intact.

This deliberately targets rollover within the Activity itself. It does not settle a same-day Activity simply because another earlier action (such as a town conversation) entered that day. That preserves the requested narrow comparison and existing first-day/same-day semantics; consequently it does not claim to fix the traced Classic day-22 camp. A broader common action transaction remains separate work.

Cost order is preserved: activity reward/cost first, existing clock overflow next, newly owed daily settlement last. A new capped-forage/Doom test makes this material: at sanity 10, the existing capped +1 forage gain occurs before the new-day −2 diet cost, yielding 8. Moving the daily cost ahead of that reward would yield 9 and would change the established transaction, so this candidate does not do that.

Seven new regressions (filter `activity_settlement_`):
- `overnight_work_returns_current_day_costs_and_clock`: both cash and supplies shifts, exact 180 active minutes, work −1 sanity plus next-day Mixed +1, one new daily supply charge, preserved old mileage and new zero-mile ledger.
- `overnight_gathering_keeps_reward_cost_and_cooldown`: Forage and Glean, exact 120 active minutes, original reward/health cost, current daily settlement, unchanged shared cooldown completion day.
- `preserves_capped_reward_before_next_day_diet_cost`: explicit cap/order behavior rather than a test that duplicates the implementation.
- `same_day_actions_do_not_repeat_daily_costs`: already initialized same-day work and gathering retain only their normal action effects.
- `exact_day_end_does_not_start_tomorrow`: completing at 13:00 with no next-day minutes does not introduce an extra day/charge.
- `reload_and_zero_time_do_not_charge_again`: save round-trip, failed duplicate action, zero-time clock write, repeated initialization and a later valid same-day action preserve once-only settlement and no movement.
- `does_not_expand_same_day_lazy_initialization`: first-day same-day gathering retains its existing lazy behavior, explicitly bounding this experiment.

All new tests preserve recorded miles, total driving minutes and the fractional pace remainder. They use local deterministic Clear-weather fixtures with explicit one-supply daily cost and no RNG bundle; no production configuration changes.

Context remains pace-based fatigue, camp +6, Balanced crossing weights 0.70/0.10/0.20, Balanced health decay 0.38, all other source/config unchanged. Daily settlement may now influence the next decision sooner and draw the same configured daily hazards earlier; no RNG domains, probabilities or bot decisions are directly changed. Its effect on outcomes must be measured.

Rustfmt passed. No Rust compile, test, or campaign was run before this freeze; await root review/target coordination. The matching `-inputs.json` supports the established candidate runner after focused regressions are approved.
