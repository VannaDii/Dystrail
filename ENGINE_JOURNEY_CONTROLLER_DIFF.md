# ENGINE_JOURNEY_CONTROLLER_DIFF.md

## Mapping Current Dystrail JourneyController to Oregon Trail Deluxe (’90s) Parity Kernel

---

## Current travel_next_leg() Order (state.rs)

travel_next_leg():
1) start_of_day()
   - reset per-day flags, counters, and record
   - tick exec orders
   - apply_starvation_tick()
   - roll_daily_illness()
   - apply_deep_aggressive_sanity_guard()
   - process_daily_weather()
   - clamp stats
   - (classic) apply_travel_wear_scaled(1.0)
2) guard_boss_gate()
3) pre_travel_checks() (ally attrition + clamp + failure)
4) vehicle_roll() -> resolve_breakdown() -> handle_vehicle_state()
5) handle_travel_block()
6) process_encounter_flow()
7) (travel_v2) apply_travel_wear()
8) endgame::run_endgame_controller()
9) handle_crossing_event()
10) record_travel_day() + end_of_day()

---

## Required OTDeluxe-Parity Order (Normative)

1) StartOfDay
2) WeatherTick
3) SuppliesBurnTick
4) HealthTick (general health / general_strain)
5) BossGateTick
6) IntentTick (rest/trade/hunt/continue)
7) VehicleTick
8) TravelBlockTick
9) EncounterTick (single derivation + roll)
10) ComputeMilesToday
11) TravelWearTick
12) EndgameTick
13) CrossingTick
14) RandomEventTick (non-navigation)
15) RecordDay + TerminalChecks + EndOfDay

---

## Phase Mapping Matrix

| OTDeluxe Kernel Phase (MECC model) | Current Dystrail Location | Delta / Gap |
| ----------------- | ------------------------- | ---------- |
| StartOfDay | start_of_day() | OK, but currently includes starvation + illness + weather (ordering issue) |
| WeatherTick | start_of_day() -> process_daily_weather() | Should run before starvation/illness and before vehicle/encounters |
| SuppliesBurnTick | (not explicit) | Missing as first-class tick; must occur before HealthTick |
| HealthTick | start_of_day() -> roll_daily_illness() | Disease roll currently before weather; general_strain not computed |
| BossGateTick | guard_boss_gate() | OK, but should occur after Weather + Supplies + Health |
| IntentTick | (no single phase today) | Missing: rest/trade/hunt/continue must be first-class intents that consume days |
| VehicleTick | vehicle_roll/resolve_breakdown | OK but must be after WeatherTick |
| TravelBlockTick | handle_travel_block() | OK; should record NonTravel day + delay credit for all block reasons |
| EncounterTick | process_encounter_flow() | OK, but encounter chance derivation must be single-source and policy-driven |
| ComputeMilesToday | distance_today/distance_today_raw | Currently computed earlier; should be explicit phase |
| TravelWearTick | apply_travel_wear() | OK (travel_v2), but should be post-travel (after ComputeMilesToday) |
| EndgameTick | endgame::run_endgame_controller() | OK, but should use computed miles for the day |
| CrossingTick | handle_crossing_event() | OK; must hard-stop travel until resolved |
| RandomEventTick | process_encounter_flow() | Needs separation between navigation hard-stops and non-navigation events |
| RecordDay + Terminal | record_travel_day() + failure_log_key() | OK |

---

## Mandatory Deltas (OTDeluxe Parity)

1) Weather must be the root cause of the day
- Today: start_of_day applies starvation/illness before weather.
- Required: WeatherTick precedes supplies burn, health, vehicle, encounters.

2) Explicit SuppliesBurnTick
- Oregon Trail Deluxe lineage: rations affect both consumption and downstream health/incident odds.
- Required: daily supplies burn before health tick, even on rest/trade/hunt days.

3) General health scalar or general_strain
- Oregon Trail Deluxe lineage uses a numeric general health accumulator (UI labels; scalar arithmetic is policy-defined).
- OTDeluxe90sPolicy parity: `health_general` is the authoritative odds/scoring scalar; Dystrail may still compute
  `general_strain` as an internal translation aid, but must not let it override Deluxe arithmetic unless explicitly
  chosen (see systems spec 16.15).

4) Affliction probability must map from health/strain
- Today: disease chance uses supplies/hp/starvation and behind-schedule modifiers.
- OTDeluxe90sPolicy: compute `p_affliction_today = clamp(AFFLICTION_CURVE_PWL(health_general), 0..0.40)` (see systems spec 6.3).

5) Hard-stop semantics for navigation events
- Deluxe lineage: lost/wrong/impassable/snowbound halt progress for several days.
- Dystrail must set miles_today = 0 and apply delay_days_remaining for these events.

6) Trail node mile markers and shortcut deltas must match Deluxe
- Deluxe EXE encodes the canonical landmark mile markers (main route endpoint `2083` at Willamette Valley) and two
  optional shortcut tables, plus an explicit combined table:
  - Sublette Cutoff: skips Fort Bridger; saves 94 miles; South Pass -> Green River is 125 miles.
  - Shortcut to The Dalles: skips Fort Walla Walla; saves 50 miles; Blue Mountains -> The Dalles is 125 miles.
  - Sublette + Dalles shortcut combined: skips both Fort Bridger and Fort Walla Walla; saves 144 miles; The Dalles becomes 1839; Willamette becomes 1939.
- Next code round must ensure crossing triggers, store availability, and victory/endgame thresholds are derived from
  these mile markers (see systems spec 8.5).

7) Intent-based non-travel days
- Deluxe lineage: rest/trade/hunt are day-atomic actions (progress = 0) and still run daily root-cause ticks.
- Dystrail must expose intents as first-class phases that short-circuit travel.

8) Encounter chance derivation is single-source and policy-driven
- Today: weather and exec orders adjust encounter_chance_today in multiple places.
- Required: compute once per day from context, then roll once.

9) RNG consumption must be phase-scoped
- Weather uses rng.weather; illness uses rng.health; encounters use rng.encounter; etc.
- No cross-phase draws that break replay determinism.

---

## What Already Matches OTDeluxe Well

- Single daily entrypoint (travel_next_leg)
- Domain RNG bundle (weather/encounter/breakdown/crossing)
- Vehicle breakdown logic (circumstance dependent)
- Encounter flow as modern analog to random events
- Partial-travel wear scaling (travel_v2) analogous to mid-day incidents in Oregon Trail’s model

---

## Political Satire Alignment (Documentation Contract)

Parity work must preserve the Oregon Trail / MECC simulation contract while allowing Dystrail's political
satire to read clearly in UI/logs.

Rules:
- Satire MUST be presentation-layer only (i18n copy, UI framing, character VO).
- Simulation events MUST remain mechanically named (resource shortage, navigation delay,
  corruption attempt, mutual aid) and carry a deterministic payload.
- Satire MUST NOT add hidden modifiers, extra RNG draws, or reorder phases.

---

## OTDeluxe Cycle Accuracy Checklist (next code round)

- WeatherTick is the day's root cause and happens before supplies burn, health, vehicle rolls,
  encounter probability derivation, and crossings.
- Health update matches OTDeluxe90sPolicy (baseline recovery choice is explicit; see systems spec 16.1).
- Affliction odds are derived from the selected odds driver (health_general or general_strain) and clamped to 0..P_AFFLICTION_MAX.
- Navigation hard-stops (lost/wrong/impassable/snowbound) set miles to 0 and apply multi-day delay.
- Trade/rest/hunt are explicit intents that consume full days (still run daily ticks).
- Encounter chance is derived once per day (single-source) and rolled once (caps/cooldowns respected).
- RNG usage is phase-scoped and stable for replay determinism.

Deluxe-specific parity checks:
- Occupation bonus multiplier is applied to the final endgame score (see systems spec 12.3).
- Crossing economics match Deluxe UI: ferry costs $5; local guide costs 3 sets of clothes (see policy table).
- Crossing choice set matches Deluxe UI/help: Attempt to ford vs Caulk & float; thresholds are respected with correct semantics:
  - Caulk/float: mechanical minimum depth 1.5 ft; Deluxe help recommends > 2.5 ft (guidance only).
  - Ford: 2.5 ft and 3.0 ft are mechanical cliffs (wet goods + drying day at 2.5..3.0; swamping past 3.0).
- Hunting enforces (a) gating (location and severe-weather blocks) and (b) caps (carry cap and per-item wagon caps; no total-weight wagon capacity unless proven).
- Stores/resupply match Deluxe constraints: supplies can only be bought at forts, ammo is 20 bullets/box, and wagon
  capacity prevents over-buying.
- Stores also match Deluxe EXE economics: base prices in cents and per-node price multipliers (see systems spec 4.3.1).
- Store UI/input caps match Deluxe EXE: oxen ≤ 20, ammo_boxes ≤ 50, clothes_sets ≤ 99, each spare part type ≤ 3, food_lbs ≤ 2000 (see systems spec 4.3.1).
- Trail progression matches Deluxe EXE landmark mile markers and shortcut semantics (see systems spec 8.5).
- (Optional / later) The Dalles endgame gate: “Raft down the River” (Columbia) vs “Take the Barlow Toll Road” (cash-gated; see systems spec 9.5). Not required for parity v1 if Dystrail ends earlier (boss gate).
