# ENGINE_KERNEL_PSEUDOCODE_DYSTRAIL_PARITY.md

## Oregon Trail Deluxe–faithful Daily Tick Kernel (Policy-driven, Event-sourced)

This kernel mirrors the Oregon Trail Deluxe (’90s) lineage model (MECC causal ordering) and supports Dystrail extensions:
- RNG streams per domain (weather/health/travel/events/crossing/trade/hunt/vehicle/encounter)
- Policy overlays (base weights, multipliers, thresholds)
- Event bus (all changes are events; UI consumes events; logs derived)

Baseline parity policy:
- `OTDeluxe90sPolicy` is the default parity target. Where Deluxe does not surface an exact numeric, the value is
  still treated as policy-scoped and parity-critical (see `ENGINE_SYSTEMS_SPEC_DYSTRAIL_PARITY.md`).

---

## Extensibility Primitives

RNG:
- rng.weather()
- rng.health()
- rng.travel()
- rng.events()
- rng.crossing()
- rng.trade()
- rng.hunt()
- rng.vehicle()
- rng.encounter()

Event bus:

Event {
  id,
  day,
  kind,
  severity,
  payload,
  tags[],
  ui_surface_hint,
}

Satire/presentation contract:
- `kind` MUST be a mechanical descriptor; satire is applied by UI using `ui_surface_hint` and
  localized `narrative_key` values stored in `payload`.
- Narrative rendering MUST NOT change simulation outcomes, RNG consumption, or phase order.
- Satire targets systems/incentives (bureaucracy, corruption, media cycles), not protected
  traits or marginalized groups.

Policy overlays:

PolicySet {
  base_weights,
  multipliers,
  thresholds,
  feature_toggles,
  per_region_overrides,
  per_season_overrides,
}

---

## Data Structures

DayContext {
  day,
  region,
  season,
  pace,
  rations_or_diet,
  inventory,
  weather_state,
  vehicle_state,
  exec_orders,
  policy,
  mode,
  general_strain,
}

DayOutcome {
  day_record,
  events[],
  travel_kind,
  ui_state,
  ended,
}

TrailGraph {
  nodes[],                      // ordered trail nodes (landmarks, forts, crossings)
  mile_markers_by_route_variant { // Deluxe EXE extracted; see systems spec 8.5
    main[],                        // canonical route
    sublette_cutoff[],             // skips Fort Bridger (0 sentinel at that index)
    dalles_shortcut[],             // skips Fort Walla Walla (0 sentinel at that index)
    sublette_and_dalles_shortcut[] // skips both Fort Bridger + Fort Walla Walla (0 sentinels at both indices)
  }
  store_node_indices[],          // nodes that offer stores (Deluxe resources; see systems spec 4.3.1)
}

Derivations (normative):
- `state.route_variant` selects the active mile-marker list for the remainder of the journey after each branch decision.
- `state.current_node_index` is derived from `state.miles_traveled` vs the active mile-marker list, not from ad-hoc mileage checks.
- Active mile-marker lists contain `0` sentinels for skipped nodes; any derivation MUST treat `0` as “node absent” (skip it),
  or the list ceases to be monotone.
- Store pricing multipliers are indexed by node order (Deluxe EXE extracted; see systems spec 4.3.1).

---

## TICK_DAY Pipeline (Normative)

```
function TICK_DAY(state, controller) -> DayOutcome:
  events = []
  policy = controller.policy   // chosen parity overlay: OTDeluxe90sPolicy

  // 0) Start-of-day bookkeeping
  state.start_of_day()
  ctx = build_context(state, controller)

  // 1) WEATHER (root cause)
  weather = generate_weather(ctx, rng.weather(), controller.weather_model)
  state.weather_today = weather
  update_precip_accumulators(state, weather)
  effects = compute_weather_effects(ctx, weather, controller.weather_model)
  apply_weather_effects(state, effects)
  events += Event(WeatherResolved, payload={weather, effects})

  // 2) CONSUMPTION ("Eat" / supplies burn)
  // Policy decides whether this is itemized (OT: food_lbs) or umbrella (Dystrail: supplies).
  consumption = compute_daily_consumption(ctx, policy, controller.supplies_model)
  apply_daily_consumption(state, consumption, events)
  events += Event(DailyConsumptionApplied, payload={consumption})
  apply_starvation_if_needed(state, events)

  // 3) HEALTH TICK (general + per-person)
  // Deluxe lineage: baseline recovery/decay is policy-defined (see systems spec 16.1)
  health_penalty = compute_health_penalty(ctx, weather, state.party, controller.health_model)
  state.health_general =
    max(0, policy.HEALTH_RECOVERY_BASELINE(state.health_general) + health_penalty)
  events += Event(HealthTick, payload={health_penalty, state.health_general})

  // 3b) Derived strain for Dystrail (parity-driver for odds)
  state.general_strain = compute_general_strain(state, policy.strain_weights)
  events += Event(GeneralStrainComputed, payload={state.general_strain})

  // 3c) Dystrail pre-travel checks (clamps + ally attrition)
  // Must not consume RNG unless explicitly documented in the phase contract.
  apply_pre_travel_checks(state, events)

  // 4) AFFLICTION ROLL (illness/injury)
  p_afflict =
    clamp(
      policy.affliction_odds_driver == HealthGeneral
        ? policy.affliction_curve_pwl(state.health_general)     // OTDeluxe90sPolicy (Q2)
        : f_strain_to_prob(state.general_strain, policy),
      0.0,
      policy.P_AFFLICTION_MAX
    )
  if rng.health().roll() < p_afflict:
    (person, affliction) = pick_affliction(ctx, rng.health())
    events += Event(AfflictionTriggered, payload={person, affliction})
    apply_affliction(state, person, affliction, events)

  // 5) BOSS GATE (blocks the day)
  // Dystrail extension: if the boss gate is active, it takes precedence over all other intents.
  if boss_gate_blocks(state):
    events += Event(BossAwait)
    return finalize_day(state, events, outcome=StoppedNeedsChoice)

  // 5b) SCHEDULED NON-TRAVEL DAYS (Deluxe-style waits)
  // Example: ferry queues can strand you for days; those days are non-travel but still run root-cause ticks.
  if state.ferry_wait_days_remaining > 0:
    state.ferry_wait_days_remaining -= 1
    events += Event(FerryWaitDay, payload={remaining: state.ferry_wait_days_remaining})
    return finalize_day(state, events, outcome=NonTravelDay)

  // 6) INTENT RESOLUTION (rest/trade/hunt/continue)
  intent = resolve_player_intent(state)
  events += Event(IntentResolved, payload={intent})

  if intent == REST:
    // REST is day-atomic. Deluxe UX allows selecting N rest days; the controller should track
    // `rest_days_remaining` and resolve the daily intent to REST until the counter reaches 0.
    // Each rest day still runs root-cause ticks (weather/consumption/health) before applying rest effects.
    apply_rest_day(state, policy, events)
    return finalize_day(state, events, outcome=NonTravelDay)

  if intent == TRADE:
    offer = generate_trade_offer(ctx, rng.trade(), controller.trade_model)
    events += Event(TradeOffer, payload=offer)
    if state.trade_accept:
      apply_trade(state, offer, events)
    // Trade is a non-travel day under OTDeluxe90sPolicy (TRADE_COST_DAYS = 1).
    return finalize_day(state, events, outcome=NonTravelDay)

  if intent == HUNT:
    hunt_result = resolve_hunt_minigame(ctx, rng.hunt())
    carry_cap = compute_hunt_carry_cap(state, policy)
    wagon_food_space_remaining = compute_wagon_food_space_remaining(state, policy)
    food_gained = min(hunt_result.food_shot, carry_cap, wagon_food_space_remaining)
    state.food_lbs += food_gained
    state.bullets -= hunt_result.bullets_used
    events += Event(HuntOutcome, payload={hunt_result, carry_cap, food_gained})
    return finalize_day(state, events, outcome=NonTravelDay)

  // 7) VEHICLE TICK (breakdowns)
  if roll_breakdown(state, ctx, rng.vehicle(), controller.breakdown_model):
    resolve_breakdown(state, events)
  if vehicle_blocks_travel(state):
    events += Event(TravelBlocked, payload={reason:"vehicle"})
    return finalize_day(state, events, outcome=BlockedDay)

  // 8) TRAVEL BLOCKS (prior delays)
  if state.blocked_days_remaining > 0:
    state.blocked_days_remaining -= 1
    state.wagon_state = Blocked
    events += Event(TravelBlocked, payload={remaining: state.blocked_days_remaining})
    return finalize_day(state, events, outcome=BlockedDay)

  // 9) ENCOUNTERS (contextual events)
  encounter_chance = compute_encounter_chance(ctx, state.general_strain, controller.encounter_model)
  if rng.encounter().roll() < encounter_chance:
    enc = pick_encounter(ctx, rng.encounter(), controller.encounter_pool)
    if enc:
      apply_encounter(state, enc, events)
      return finalize_day(state, events, outcome=EncounterDay)

  // 10) COMPUTE MILES TODAY (travel progress)
  // Dystrail note: keep both "raw" and "actual" distances explicit; policy defines aggregation semantics.
  (distance_today_raw, distance_today) = compute_distance_today(ctx, weather, rng.travel(), controller.travel_model)
  computed_miles_today = policy.compute_miles_today(distance_today_raw, distance_today)

  // Navigation hard-stops (lost/wrong/impassable/snowbound)
  nav_event = maybe_roll_navigation_event(ctx, rng.events(), controller.event_model)
  if nav_event in {LostTrail, WrongTrail, Impassable, Snowbound}:
    computed_miles_today = 0
    apply_delay(nav_event, state)
    events += Event(nav_event, severity=Critical)
    return finalize_day(state, events, outcome=DelayedDay)

  state.miles_traveled += computed_miles_today
  events += Event(TravelProgress, payload={computed_miles_today, state.miles_traveled, distance_today_raw, distance_today})

  // 11) TRAVEL WEAR (Dystrail extension)
  // Wear is applied based on actual travel this day (0 if hard-stopped / non-travel).
  apply_travel_wear(state, computed_miles_today, controller.vehicle_model, events)

  // 12) ENDGAME
  run_endgame_controller(state, computed_miles_today)

  // 13) CROSSINGS / LANDMARKS
  if reached_crossing_or_landmark(state):
    node = current_node(state)
    events += Event(ArrivedAtNode, payload=node)
    if node.type == RIVER:
      state.wagon_state = Stopped
      events += Event(CrossingChoiceNeeded, payload=node.river_state)
      return finalize_day(state, events, outcome=StoppedNeedsChoice)
    if node.type == DALLES_ENDGAME_GATE:
      // Deluxe: The Dalles requires choosing between Columbia River rafting vs Barlow Toll Road.
      state.wagon_state = Stopped
      events += Event(DallesEndgameChoiceNeeded, payload={node:"The Dalles"})
      return finalize_day(state, events, outcome=StoppedNeedsChoice)

  // 14) RANDOM EVENTS (non-navigation)
  extra_events = roll_random_events(ctx, rng.events(), controller.event_model)
  for e in extra_events:
    apply_event(state, e)
    events += e

  // 15) TERMINAL CHECKS
  if terminal_failure(state):
    events += Event(GameOver, severity=Critical)
    return finalize_day(state, events, outcome=GameOver)

  return finalize_day(state, events, outcome=TraveledDay)
```

---

## Crossing Resolver (Normative)

```
function RESOLVE_CROSSING(state, choice, controller):
  policy = controller.policy
  river = state.river_state
  ctx = build_crossing_context(state, river, policy)

  // Crossing resolution is a non-travel day. The surrounding controller/kernel is responsible for spending
  // `policy.CROSSING_COST_DAYS` days as non-travel while resolving the crossing UI flow.

  // Base outcome weights are policy-defined and then modified by context (depth/swiftness/weather/assistance).
  // Outcome families MUST include: safe, stuck_in_mud, supplies_wet, tipped, sank.
  // Drownings are required for caulk/float outcomes (Deluxe strings); OTDeluxe90sPolicy default excludes deaths for ferry.
  weights = policy.CROSSING_OUTCOME_WEIGHTS[choice]
  weights = adjust_crossing_outcome_weights(weights, ctx, policy)

  // Enforce method availability gates and deterministic "risk cliff" effects (MECC Appendix B; adopted for Deluxe parity).
  if choice == FERRY and ctx.depth_ft < policy.FERRY_MIN_DEPTH_FT:
    emit(Event(CrossingChoiceInvalid, payload={choice, reason:"too_shallow_for_ferry", depth_ft: ctx.depth_ft}))
    return
  if choice == CAULK_FLOAT and ctx.depth_ft < policy.CAULK_FLOAT_MIN_DEPTH_FT:
    emit(Event(CrossingChoiceInvalid, payload={choice, reason:"too_shallow_to_caulk_float", depth_ft: ctx.depth_ft}))
    return

  if choice == FORD:
    // MECC explicit depth cliffs:
    if policy.FORD_WET_GOODS_MIN_DEPTH_FT <= ctx.depth_ft && ctx.depth_ft <= policy.FORD_SWAMP_DEPTH_FT:
      // Deterministic consequence (in addition to any accident sample): supplies get wet and an extra day is lost drying.
      state.delay_days_remaining += policy.DRYING_COST_DAYS
      emit(Event(SuppliesWet, payload={choice:"ford", depth_ft: ctx.depth_ft, drying_days: policy.DRYING_COST_DAYS}))
    if ctx.depth_ft > policy.FORD_SWAMP_DEPTH_FT:
      // Wagon swamps past 3.0 ft; make severe outcomes much more likely (exact scaling is policy-defined).
      weights = bias_toward_severe_outcomes(weights, policy, ctx)

  if choice == CAULK_FLOAT:
    // Mechanical minimum is 1.5 ft; Deluxe help recommends > 2.5 ft (guidance only).
    if ctx.depth_ft < policy.CAULK_FLOAT_HELP_RECOMMENDED_MIN_DEPTH_FT:
      weights = bias_toward_severe_outcomes(weights, policy, ctx)

  if choice == FERRY:
    // Deluxe UI confirms: $5.00. MECC: accident risk is 0%..10% by swiftness; OTDeluxe90sPolicy default (Q5) treats
    // ferry accidents as non-lethal until confirmed (wet/time loss/minor loss only).
    cost_cash_cents(policy.FERRY_COST_CENTS)
    // Determinism note: draw wait-days before drawing the crossing outcome.
    wait_days = sample_ferry_wait_days(policy, rng.crossing())
    state.ferry_wait_days_remaining += wait_days

  if choice == HIRE_GUIDE:
    // Deluxe UI confirms: 3 sets of clothes. Risk reduction factor is policy-defined.
    cost_clothes(policy.GUIDE_COST_CLOTHES)
    weights = scale_bad_outcomes(weights, policy.GUIDE_RISK_MULT)

  outcome = weighted_choice(weights, rng.crossing())
  apply_crossing_outcome(outcome, state, policy)
  emit(Event(CrossingOutcomeResolved, payload={choice, outcome, ctx}))
```

---

## The Dalles Endgame Resolver (Normative; Deluxe)

```
function RESOLVE_DALLES_ENDGAME(state, choice, controller):
  policy = controller.policy

  // Scope note (Q19/Q20): Dystrail parity v1 may defer implementing this resolver entirely if the
  // journey ends earlier (e.g., via boss gate/endgame) and never reaches The Dalles.
  // Keep the interface and event surface stable so the subsystem can be added later without refactoring the kernel.

  // This gate exists at node 16 (The Dalles). It MUST block travel beyond The Dalles until resolved.
  // Both options advance time and can cause losses; the exact distributions remain policy-defined until extracted.

  if choice == BARLOW_TOLL_ROAD:
    if state.cash_cents < policy.BARLOW_TOLL_ROAD_COST_CENTS:
      emit(Event(ChoiceRejected, payload={choice, reason:"insufficient_cash"}))
      return
    cost_cash_cents(policy.BARLOW_TOLL_ROAD_COST_CENTS)
    advance_days(policy.BARLOW_TOLL_ROAD_TIME_DAYS)  // each day runs the root-cause ticks
    outcome = sample_barlow_outcome(policy, rng.crossing())
    apply_barlow_outcome(outcome, state, policy)
    emit(Event(DallesGateResolved, payload={choice, outcome}))
    return

  if choice == RAFT_DOWN_COLUMBIA:
    advance_days(policy.RAFTING_TIME_DAYS)           // if rafting spans multiple days; policy-defined
    outcome = weighted_choice(policy.RAFTING_OUTCOME_WEIGHTS, rng.crossing())
    apply_rafting_outcome(outcome, state, policy)
    emit(Event(DallesGateResolved, payload={choice, outcome}))
    return
```

---

## Determinism Contract

- Each phase consumes only its RNG stream.
- Any early return still finalizes day state and emits consistent events.
- Event logs are derived, not causal.
- Policy overlays are applied before phase evaluation, not mid-phase.
