use super::*;

fn approach(
    mode: GameMode,
    pace: PaceId,
    crossing: usize,
    road_miles_before: f32,
    clock: u16,
    route_id: &str,
) -> (GameState, Rc<RngBundle>) {
    let mut state = GameState {
        mode,
        pace,
        trail_distance: 2400.0,
        ..GameState::default()
    };
    state.continuity.route_services.route_id = Some(route_id.into());
    state.crossings_completed = u32::try_from(crossing).unwrap();
    state.miles_traveled_actual = CROSSING_MILESTONES[crossing]
        - crate::route::simulation_distance(&state, road_miles_before);
    state.miles_traveled = state.miles_traveled_actual;
    state.continuity.clock_minutes = clock;
    let bundle = Rc::new(RngBundle::from_user_seed(4242));
    state.attach_rng_bundle(Rc::clone(&bundle));
    state.start_of_day();
    state.weather_state.today = Weather::Clear;
    state.weather_travel_multiplier = 1.0;
    state.exec_travel_multiplier = 1.0;
    state.exec_breakdown_bonus = 0.0;
    state.illness_travel_penalty = 1.0;
    state.stats.hp = 10;
    state.stats.sanity = 10;
    state.stats.supplies = 20;
    state.journey_breakdown.base = 0.0;
    state.journey_breakdown.beta = 0.0;
    state.journey_wear.base = 0.0;
    state.encounters.occurred_today = true;
    state.encounters_today = MAX_ENCOUNTERS_PER_DAY;
    state.inventory.tags.clear();
    state.receipts.clear();
    state.budget_cents = 0;
    state.budget = 0;
    force_crossing(&mut state, crossings::CrossingResult::Pass);
    (state, bundle)
}

fn force_crossing(state: &mut GameState, result: crossings::CrossingResult) {
    let (pass, detour, terminal) = match result {
        crossings::CrossingResult::Pass => (1.0, 0.0, 0.0),
        crossings::CrossingResult::Detour(_) => (0.0, 1.0, 0.0),
        crossings::CrossingResult::TerminalFail => (0.0, 0.0, 1.0),
    };
    state.journey_crossing.pass = pass;
    state.journey_crossing.detour = detour;
    state.journey_crossing.terminal = terminal;
    state.journey_crossing.bribe.pass_bonus = 0.0;
    state.journey_crossing.bribe.detour_bonus = 0.0;
    state.journey_crossing.bribe.terminal_penalty = 0.0;
    state.journey_crossing.detour_hours.min = 2;
    state.journey_crossing.detour_hours.max = 2;
}

fn drive(state: &mut GameState) -> (bool, String, bool) {
    state.prepare_travel_clock();
    state.start_of_day();
    state.apply_pace_and_diet(&crate::PacingConfig::default_config());
    state.travel_next_leg(&EndgameTravelCfg::default())
}

#[test]
fn crossing_arrival_mid_hour_resolves_once_and_preserves_pass_detour_and_terminal_costs() {
    for mode in [GameMode::Classic, GameMode::Deep] {
        for result in [
            crossings::CrossingResult::Pass,
            crossings::CrossingResult::Detour(2),
            crossings::CrossingResult::TerminalFail,
        ] {
            let (mut state, bundle) = approach(
                mode,
                PaceId::Steady,
                1,
                30.0,
                crate::journal::morning(),
                "uninterrupted-test-road",
            );
            force_crossing(&mut state, result);
            state.budget_cents = 5_000;
            state.budget = 50;
            let before = state.clone();
            let (ended, _, breakdown) = drive(&mut state);
            let terminal = matches!(result, crossings::CrossingResult::TerminalFail);
            assert_eq!(ended, terminal);
            assert!(!breakdown);
            assert_eq!(state.miles_traveled_actual.to_bits(), 1250.0_f32.to_bits());
            assert_eq!(state.ledger.current_day_miles.to_bits(), 30.0_f32.to_bits());
            assert_eq!(state.continuity.driving_minutes_total, 30);
            let stationary_minutes = match result {
                crossings::CrossingResult::Pass => 30,
                crossings::CrossingResult::Detour(_) => 120,
                crossings::CrossingResult::TerminalFail => 0,
            };
            assert_eq!(
                state.continuity.clock_minutes,
                before.continuity.clock_minutes + 30 + stationary_minutes
            );
            assert_eq!(state.crossing_events.len(), 1);
            assert_eq!(bundle.crossing().draws(), 1);
            let event = state.crossing_events.last().unwrap();
            let cfg = CrossingConfig::default();
            let cost = cfg.types.get(&event.kind).unwrap().bribe.base_cost_cents;
            assert_eq!(event.bribe_cost_cents, cost);
            assert_eq!(state.budget_cents, before.budget_cents - cost);
            assert_eq!(state.bribes_spent_cents, cost);
            assert_eq!(state.stats, before.stats);
            assert_eq!(state.vehicle.wear.to_bits(), before.vehicle.wear.to_bits());
            assert_eq!(
                event.bribe_success,
                Some(matches!(result, crossings::CrossingResult::Pass))
            );
            assert_eq!(
                event.detour_hours,
                matches!(result, crossings::CrossingResult::Detour(_)).then_some(2)
            );
            assert_eq!(state.crossing_failures, u32::from(terminal));
            assert_eq!(state.day, before.day);
            assert!(state.day_records.is_empty());
            if terminal {
                assert!(matches!(
                    state.ending,
                    Some(Ending::Collapse {
                        cause: CollapseCause::Crossing
                    })
                ));
            } else {
                assert_eq!(state.crossings_completed, 2);
                let paid = state.budget_cents;
                let (ended, _, _) = drive(&mut state);
                assert!(!ended);
                assert_eq!(state.crossing_events.len(), 1);
                assert_eq!(bundle.crossing().draws(), 1);
                assert_eq!(state.budget_cents, paid);
            }
        }
    }
}

#[test]
fn crossing_arrival_in_the_final_minute_keeps_the_exact_boundary_for_every_route_and_pace() {
    for route in crate::route::routes() {
        for pace in [PaceId::Steady, PaceId::Heated, PaceId::Blitz] {
            for (crossing, milestone) in CROSSING_MILESTONES.iter().enumerate() {
                let (mut state, bundle) = approach(
                    GameMode::Classic,
                    pace,
                    crossing,
                    0.125,
                    crate::travel_time::TRAVEL_DAY_END - 1,
                    &route.id,
                );
                let before = state.clone();
                let (ended, _, breakdown) = drive(&mut state);
                assert!(!ended && !breakdown, "{} {pace:?} {crossing}", route.id);
                assert_eq!(
                    state.miles_traveled_actual.to_bits(),
                    milestone.to_bits(),
                    "{} {pace:?} {crossing}",
                    route.id
                );
                assert_eq!(state.continuity.driving_minutes_total, 1);
                assert_eq!(state.leg_minutes, 1);
                assert_eq!(
                    state.crossings_completed,
                    u32::try_from(crossing + 1).unwrap()
                );
                assert_eq!(state.crossing_events.len(), 1);
                assert_eq!(state.crossing_events[0].day, before.day);
                assert_eq!(bundle.crossing().draws(), 1);
                assert_eq!(state.day, before.day + 1);
                assert_eq!(
                    state.continuity.clock_minutes,
                    crate::journal::morning() + 30
                );
                assert_eq!(state.day_records.len(), 1);
                assert_eq!(state.day_records[0].kind, TravelDayKind::Travel);
                assert_eq!(
                    state.day_records[0].miles.to_bits(),
                    (milestone - before.miles_traveled_actual).to_bits()
                );
                let pending = state.ledger.current_day_record.as_ref().unwrap();
                assert_eq!(pending.kind, TravelDayKind::NonTravel);
                assert_eq!(pending.miles.to_bits(), 0.0_f32.to_bits());
            }
        }
    }
}

#[test]
fn crossing_arrival_costs_that_fill_the_day_settle_it_without_driving_during_the_stop() {
    for (result, stationary_minutes) in [
        (crossings::CrossingResult::Pass, 30),
        (crossings::CrossingResult::Detour(2), 120),
    ] {
        let (mut state, bundle) = approach(
            GameMode::Classic,
            PaceId::Steady,
            1,
            1.0,
            crate::travel_time::TRAVEL_DAY_END - stationary_minutes - 1,
            "uninterrupted-test-road",
        );
        force_crossing(&mut state, result);
        let before = state.clone();
        let (ended, _, _) = drive(&mut state);
        assert!(!ended);
        assert_eq!(state.day, before.day + 1);
        assert_eq!(state.continuity.clock_minutes, crate::journal::morning());
        assert_eq!(state.continuity.driving_minutes_total, 1);
        assert_eq!(state.miles_traveled_actual.to_bits(), 1250.0_f32.to_bits());
        assert_eq!(state.day_records.len(), 1);
        assert_eq!(state.day_records[0].kind, TravelDayKind::Travel);
        assert_eq!(state.day_records[0].miles.to_bits(), 1.0_f32.to_bits());
        assert!(state.ledger.current_day_record.is_none());
        assert_eq!(state.crossing_events.len(), 1);
        assert_eq!(state.crossing_events[0].day, before.day);
        assert_eq!(bundle.crossing().draws(), 1);
    }
}

#[test]
fn crossing_arrival_terminal_at_day_end_retains_the_approach_without_an_empty_next_day() {
    for mode in [GameMode::Classic, GameMode::Deep] {
        for crossing in [1, 2] {
            let (mut state, bundle) = approach(
                mode,
                PaceId::Steady,
                crossing,
                1.0,
                crate::travel_time::TRAVEL_DAY_END - 1,
                "uninterrupted-test-road",
            );
            force_crossing(&mut state, crossings::CrossingResult::TerminalFail);
            let before_day = state.day;
            let (ended, message, _) = drive(&mut state);
            assert!(ended);
            assert_eq!(message, LOG_CROSSING_FAILURE);
            assert_eq!(state.day, before_day);
            assert_eq!(
                state.continuity.clock_minutes,
                crate::travel_time::TRAVEL_DAY_END
            );
            assert_eq!(state.continuity.driving_minutes_total, 1);
            assert_eq!(
                state.miles_traveled_actual.to_bits(),
                CROSSING_MILESTONES[crossing].to_bits()
            );
            assert!(state.day_records.is_empty());
            let pending = state.ledger.current_day_record.as_ref().unwrap();
            assert_eq!(pending.kind, TravelDayKind::Travel);
            assert_eq!(pending.miles.to_bits(), 1.0_f32.to_bits());
            assert_eq!(state.crossing_events.len(), 1);
            assert_eq!(state.crossing_events[0].day, before_day);
            assert_eq!(bundle.crossing().draws(), 1);
        }
    }
}

#[test]
fn crossing_arrival_preserves_the_first_checkpoint_alternate_route() {
    let (mut state, bundle) = approach(
        GameMode::Classic,
        PaceId::Steady,
        0,
        30.0,
        crate::journal::morning(),
        "uninterrupted-test-road",
    );
    force_crossing(&mut state, crossings::CrossingResult::TerminalFail);
    let (ended, message, _) = drive(&mut state);
    assert!(!ended);
    assert!(state.ending.is_none());
    assert_eq!(message, "log.crossing.denied");
    assert_eq!(state.crossings_completed, 1);
    assert_eq!(state.crossing_failures, 0);
    assert_eq!(state.crossing_detours_taken, 1);
    assert_eq!(state.miles_traveled_actual.to_bits(), 650.0_f32.to_bits());
    assert_eq!(state.continuity.driving_minutes_total, 30);
    assert_eq!(
        state.continuity.clock_minutes,
        crate::journal::morning() + 150
    );
    assert_eq!(
        state.crossing_events[0].detour_reason,
        Some(CrossingDetourReason::CheckpointDenied)
    );
    assert_eq!(bundle.crossing().draws(), 1);
}

#[test]
fn crossing_arrival_already_reached_milestones_do_not_rewind_or_credit_driving() {
    for overshoot in [0.0, 0.125] {
        let (mut state, bundle) = approach(
            GameMode::Classic,
            PaceId::Steady,
            1,
            -overshoot,
            crate::journal::morning(),
            "uninterrupted-test-road",
        );
        let miles = state.miles_traveled_actual;
        let (ended, _, _) = drive(&mut state);
        assert!(!ended);
        assert_eq!(state.miles_traveled_actual.to_bits(), miles.to_bits());
        assert_eq!(state.continuity.driving_minutes_total, 0);
        assert_eq!(
            state.continuity.clock_minutes,
            crate::journal::morning() + 30
        );
        assert_eq!(state.crossings_completed, 2);
        assert_eq!(state.crossing_events.len(), 1);
        assert_eq!(bundle.crossing().draws(), 1);
    }
}

#[test]
fn crossing_arrival_respects_earlier_towns_and_the_route_endpoint() {
    for route in crate::route::routes() {
        let town = &route.stops[1];
        let (mut state, bundle) = approach(
            GameMode::Classic,
            PaceId::Blitz,
            0,
            0.125,
            crate::journal::morning(),
            &route.id,
        );
        state.miles_traveled_actual =
            crate::route::simulation_distance(&state, f32::from(town.mile) - 0.125);
        state.miles_traveled = state.miles_traveled_actual;
        state.prev_miles_traveled = state.miles_traveled_actual;
        let (ended, _, _) = drive(&mut state);
        assert!(!ended);
        assert!((crate::route::physical_miles(&state) - f32::from(town.mile)).abs() < 0.001);
        assert!(state.miles_traveled_actual < CROSSING_MILESTONES[0]);
        assert_eq!(state.continuity.driving_minutes_total, 1);
        assert!(state.crossing_events.is_empty());
        assert_eq!(bundle.crossing().draws(), 0);

        let (mut state, bundle) = approach(
            GameMode::Classic,
            PaceId::Blitz,
            1,
            0.125,
            crate::journal::morning(),
            &route.id,
        );
        state.trail_distance = 1200.0;
        let last_town_gap = route.total_miles - f32::from(route.stops.last().unwrap().mile);
        let road_remaining = if last_town_gap > 0.0 {
            last_town_gap.min(0.125) / 2.0
        } else {
            0.125
        };
        state.miles_traveled_actual =
            state.trail_distance - crate::route::simulation_distance(&state, road_remaining);
        state.miles_traveled = state.miles_traveled_actual;
        state.prev_miles_traveled = state.miles_traveled_actual;
        let (_, _, _) = drive(&mut state);
        assert_eq!(
            state.miles_traveled_actual.to_bits(),
            state.trail_distance.to_bits()
        );
        assert_eq!(state.continuity.driving_minutes_total, 1);
        assert!(state.crossing_events.is_empty());
        assert_eq!(bundle.crossing().draws(), 0);
    }
}

fn convoy_choice(ratio: f32) -> Encounter {
    Encounter {
        id: "convoy-arrival-test".into(),
        name: "Join the convoy".into(),
        desc: String::new(),
        weight: 1,
        regions: Vec::new(),
        modes: Vec::new(),
        choices: vec![crate::data::Choice {
            label: "Ride along".into(),
            effects: crate::data::Effects {
                travel_bonus_ratio: ratio,
                ..crate::data::Effects::default()
            },
        }],
        hard_stop: false,
        major_repair: false,
        chainable: false,
    }
}

fn elapsed_active_minutes(before: &GameState, after: &GameState) -> i64 {
    i64::from(after.day - before.day) * 300 + i64::from(after.continuity.clock_minutes)
        - i64::from(before.continuity.clock_minutes)
}

#[test]
fn encounter_arrival_includes_choice_and_crossing_time_without_duplicate_resolution() {
    for mode in [GameMode::Classic, GameMode::Deep] {
        for clock in [480, 765, 779] {
            for result in [
                crossings::CrossingResult::Pass,
                crossings::CrossingResult::Detour(2),
                crossings::CrossingResult::TerminalFail,
            ] {
                let (mut state, bundle) = approach(
                    mode,
                    PaceId::Steady,
                    1,
                    0.5,
                    clock,
                    "uninterrupted-test-road",
                );
                force_crossing(&mut state, result);
                state.current_encounter = Some(convoy_choice(0.5));
                let before = state.clone();
                assert!(state.resolve_encounter_choice(0));
                assert_eq!(state.miles_traveled_actual.to_bits(), 1250.0_f32.to_bits());
                assert_eq!(state.continuity.driving_minutes_total, 1);
                assert_eq!(state.crossing_events.len(), 1);
                assert_eq!(bundle.crossing().draws(), 1);
                let delay = match result {
                    crossings::CrossingResult::Pass => 30,
                    crossings::CrossingResult::Detour(_) => 120,
                    crossings::CrossingResult::TerminalFail => 0,
                };
                assert_eq!(elapsed_active_minutes(&before, &state), 30 + delay);
                assert_eq!(
                    state.ending.is_some(),
                    matches!(result, crossings::CrossingResult::TerminalFail)
                );
                assert!(state.current_encounter.is_none());
                let saved = serde_json::to_value(&state).unwrap();
                let mut loaded: GameState = serde_json::from_value(saved.clone()).unwrap();
                assert!(!loaded.resolve_encounter_choice(0));
                assert_eq!(serde_json::to_value(&loaded).unwrap(), saved);
            }
        }
    }
}

#[test]
fn encounter_arrival_opens_the_reached_town_without_passing_it() {
    for mode in [GameMode::Classic, GameMode::Deep] {
        let (mut state, _) = approach(mode, PaceId::Steady, 0, 1.0, 480, "journalist");
        let start = crate::route::simulation_distance(&state, 270.0);
        state.miles_traveled_actual = start;
        state.miles_traveled = start;
        state.prev_miles_traveled = start;
        state.current_encounter = Some(convoy_choice(0.5));
        let before = state.clone();
        assert!(state.resolve_encounter_choice(0));
        assert!((crate::route::physical_miles(&state) - 280.0).abs() < 0.001);
        assert_eq!(state.continuity.route_services.stop, Some(280));
        assert_eq!(elapsed_active_minutes(&before, &state), 30);
        assert_eq!(state.continuity.driving_minutes_total, 10);
        assert!(state.crossing_events.is_empty());
    }
}

#[test]
fn encounter_resolution_rejects_invalid_or_unaffordable_choices_without_spending_time() {
    let (mut state, _) = approach(
        GameMode::Classic,
        PaceId::Steady,
        1,
        0.5,
        779,
        "uninterrupted-test-road",
    );
    state.current_encounter = Some(convoy_choice(0.5));
    state.current_encounter.as_mut().unwrap().choices[0]
        .effects
        .cash_cents = -100;
    let before = serde_json::to_value(&state).unwrap();
    assert!(!state.resolve_encounter_choice(1));
    assert!(!state.resolve_encounter_choice(0));
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
}

#[test]
fn encounter_resolution_preserves_requested_rest_and_does_not_bypass_breakdowns() {
    let (mut state, _) = approach(
        GameMode::Classic,
        PaceId::Steady,
        1,
        0.5,
        480,
        "uninterrupted-test-road",
    );
    state.current_encounter = Some(convoy_choice(0.5));
    state.current_encounter.as_mut().unwrap().choices[0]
        .effects
        .rest = true;
    state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 1,
    });
    let before = state.clone();
    assert!(state.resolve_encounter_choice(0));
    assert_eq!(
        state.miles_traveled_actual.to_bits(),
        before.miles_traveled_actual.to_bits()
    );
    assert_eq!(state.continuity.driving_minutes_total, 0);
    assert_eq!(elapsed_active_minutes(&before, &state), 30);
    assert!(state.day_state.rest.rest_requested);
    assert!(state.breakdown.is_some());
    assert!(state.crossing_events.is_empty());
}

#[test]
fn overnight_encounter_settles_new_day_costs_once() {
    let (mut state, _) = approach(
        GameMode::Classic,
        PaceId::Steady,
        1,
        0.5,
        765,
        "uninterrupted-test-road",
    );
    state.journey_daily.supplies.base = 1.0;
    state.current_encounter = Some(convoy_choice(0.0));
    let before = state.clone();
    assert!(state.resolve_encounter_choice(0));
    assert_eq!(state.day, before.day + 1);
    assert_eq!(state.continuity.clock_minutes, 495);
    assert_eq!(state.continuity.driving_minutes_total, 0);
    assert!(state.day_state.lifecycle.day_initialized);
    assert_eq!(state.stats.supplies, before.stats.supplies - 1);
    let after = serde_json::to_value(&state).unwrap();
    state.start_of_day();
    assert_eq!(serde_json::to_value(&state).unwrap(), after);
}
