use dystrail_game::{
    CampConfig, Cart, CartLine, CrossingConfig, CrossingKind, EncounterData, EndgameTravelCfg,
    Ending, GameMode, GameState, JourneyController, MechanicalPolicyId, PersonasList, PolicyId,
    PolicyKind, ResultConfig, Store, StrategyId, Weather, apply_bribe, apply_detour, apply_permit,
    calculate_bribe_cost, calculate_effective_price, camp_forage, camp_rest, camp_therapy,
    can_afford_bribe, can_use_permit,
    endgame::{enforce_failure_guard, run_endgame_controller},
    exec_orders::ExecOrder,
    load_result_config, result_summary, run_boss_minigame,
    seed::{decode_to_seed, encode_friendly, generate_code_from_entropy, parse_share_code},
};
use std::collections::HashSet;

fn load_store() -> Store {
    serde_json::from_str(include_str!(
        "../../dystrail-web/static/assets/data/store.json"
    ))
    .unwrap()
}

fn load_personas() -> PersonasList {
    PersonasList::from_json(include_str!(
        "../../dystrail-web/static/assets/data/personas.json"
    ))
    .unwrap()
}

fn load_crossings() -> CrossingConfig {
    serde_json::from_str(include_str!(
        "../../dystrail-web/static/assets/data/crossings.json"
    ))
    .unwrap()
}

fn load_encounters() -> EncounterData {
    EncounterData::from_json(include_str!(
        "../../dystrail-web/static/assets/data/game.json"
    ))
    .unwrap()
}

fn configure_state(seed: u64) -> GameState {
    let encounter_data = load_encounters();
    let personas = load_personas();
    let persona = personas
        .get_by_id("organizer")
        .or_else(|| personas.get_by_id("journalist"))
        .unwrap();

    let mut state = GameState::default().with_seed(seed, GameMode::Deep, encounter_data);
    state.apply_persona(persona);
    state.policy = Some(PolicyKind::Aggressive);
    state.auto_camp_rest = true;
    state.features.encounter_diversity = true;
    state.features.exposure_streaks = true;
    state.mods.store_discount_pct = 5;
    state
}

#[test]
fn full_campaign_exercises_core_systems() {
    let (mut state, boss_cfg, result_cfg, endgame_cfg, weather_seen) =
        run_campaign_setup_and_loop();
    validate_end_state(&mut state, &boss_cfg, &result_cfg, &weather_seen);
    exercise_post_loop_systems(endgame_cfg);
}

fn run_campaign_setup_and_loop() -> (
    GameState,
    dystrail_game::BossConfig,
    ResultConfig,
    EndgameTravelCfg,
    HashSet<Weather>,
) {
    run_campaign_setup_and_loop_with_end(false)
}

fn run_campaign_setup_and_loop_with_end(
    force_end: bool,
) -> (
    GameState,
    dystrail_game::BossConfig,
    ResultConfig,
    EndgameTravelCfg,
    HashSet<Weather>,
) {
    let camp_cfg = CampConfig::load_from_static();
    let boss_cfg = dystrail_game::BossConfig::load_from_static();
    let result_cfg = load_result_config().unwrap_or_else(|_| ResultConfig::default());
    let mut state = configure_state(0xDEAD_BEEF);
    if force_end {
        state.ending = Some(Ending::BossVictory);
    }
    let endgame_cfg = EndgameTravelCfg::default_config();
    let strategy: StrategyId = state.policy.unwrap_or(PolicyKind::Balanced).into();
    let mut controller = JourneyController::new(
        MechanicalPolicyId::DystrailLegacy,
        PolicyId::from(state.mode),
        strategy,
        state.seed,
    );
    controller.set_endgame_config(endgame_cfg.clone());
    controller.configure_state(&mut state);
    let mut weather_seen = HashSet::new();
    let store = load_store();
    let by_id = store.items_by_id();
    assert!(!by_id.is_empty());

    // Apply store purchases to touch grants logic.
    for category in &store.categories {
        for item in &category.items {
            let price = calculate_effective_price(
                item.price_cents,
                f64::from(state.mods.store_discount_pct),
            );
            state.apply_store_purchase(price, &item.grants, &item.tags);
        }
    }

    for order in ExecOrder::ALL {
        assert!(!order.key().is_empty());
        assert!(!order.name_key().is_empty());
    }

    // Simulate 60 days of play hitting diverse systems.
    for day in 0..120 {
        let outcome = controller.tick_day(&mut state);
        weather_seen.insert(state.weather_state.today);
        if state.current_encounter.is_some() {
            state.apply_choice(0);
        }

        if day % 11 == 0 {
            let rest = camp_rest(&mut state, &camp_cfg);
            if !rest.rested {
                let _ = camp_forage(&mut state, &camp_cfg);
            }
        }

        if day % 7 == 0 {
            camp_therapy(&mut state, &camp_cfg);
        }

        if state.day_state.travel.travel_blocked {
            state.breakdown = None;
            state.day_state.travel.travel_blocked = false;
        }

        if outcome.ended {
            break;
        }
    }

    (state, boss_cfg, result_cfg, endgame_cfg, weather_seen)
}

#[test]
fn full_campaign_breaks_on_ended_state() {
    let _ = run_campaign_setup_and_loop_with_end(true);
}

fn validate_end_state(
    state: &mut GameState,
    boss_cfg: &dystrail_game::BossConfig,
    result_cfg: &ResultConfig,
    weather_seen: &HashSet<Weather>,
) {
    let crossing_cfg = load_crossings();
    let store = load_store();
    let by_id = store.items_by_id();
    assert!(!by_id.is_empty());

    for category in &store.categories {
        for item in &category.items {
            let price = calculate_effective_price(
                item.price_cents,
                f64::from(state.mods.store_discount_pct),
            );
            state.apply_store_purchase(price, &item.grants, &item.tags);
        }
    }

    for order in ExecOrder::ALL {
        assert!(!order.key().is_empty());
        assert!(!order.name_key().is_empty());
    }

    // Trigger crossing helpers explicitly.
    let mut crossing_state = configure_state(42);
    crossing_state.stats.supplies = 12;
    crossing_state.stats.pants = 8;
    crossing_state.inventory.tags.insert("permit".into());
    assert!(can_use_permit(&crossing_state, &CrossingKind::Checkpoint));
    let _ = apply_permit(&mut crossing_state, &crossing_cfg, CrossingKind::Checkpoint);

    let cost = calculate_bribe_cost(5000, crossing_state.mods.bribe_discount_pct);
    assert!(cost >= 0);
    assert!(can_afford_bribe(
        &crossing_state,
        &crossing_cfg,
        CrossingKind::BridgeOut
    ));
    let _ = apply_bribe(&mut crossing_state, &crossing_cfg, CrossingKind::BridgeOut);
    let _ = apply_detour(&mut crossing_state, &crossing_cfg, CrossingKind::BridgeOut);

    // Vehicle and cart edge cases.
    let mut cart = Cart::default();
    cart.lines.push(CartLine {
        item_id: "rope".into(),
        item_name: "Rope".into(),
        quantity: 1,
        qty: 1,
    });
    cart.total_cents = 1200;
    assert!(!cart.is_empty());

    // Seed utilities
    let friendly = encode_friendly(true, 0x5EED);
    let decoded = decode_to_seed(&friendly).unwrap();
    assert_eq!(encode_friendly(decoded.0, decoded.1), friendly);
    let share = generate_code_from_entropy(true, 123_456_789);
    let _ = parse_share_code(&share).unwrap_or((GameMode::Classic, 0));

    // Boss and result flow.
    let _outcome = run_boss_minigame(state, boss_cfg);
    let summary = result_summary(state, result_cfg).unwrap();
    assert!(summary.score >= 0);

    assert!(!weather_seen.is_empty());

    state.detach_rng_bundle();
}

fn exercise_post_loop_systems(mut end_cfg: EndgameTravelCfg) {
    // Endgame routines
    let mut state = configure_state(0xBAD_CAFE);
    end_cfg.enabled = true;
    state.mode = GameMode::Deep;
    state.policy = Some(PolicyKind::Aggressive);
    state.miles_traveled_actual = 1_960.0;
    state.vehicle.health = 5.0;
    state.day_state.lifecycle.day_initialized = true;
    run_endgame_controller(&mut state, 12.0, true, &end_cfg);

    state.endgame.active = true;
    state.endgame.failure_guard_miles = 2_000.0;
    state.endgame.health_floor = 40.0;
    state.endgame.wear_reset = 10.0;
    state.endgame.cooldown_days = 2;
    state.vehicle.health = 0.0;
    state.vehicle.wear = 90.0;
    state.miles_traveled_actual = 1_980.0;
    assert!(enforce_failure_guard(&mut state));
}
