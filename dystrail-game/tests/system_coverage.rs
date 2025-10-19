#![allow(deprecated)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::float_cmp)]
#![allow(clippy::redundant_clone)]

use dystrail_game::state::Season;
use dystrail_game::store::StoreCategory;
use dystrail_game::vehicle::{PartWeights, process_daily_breakdown, weighted_pick};
use dystrail_game::weather::{
    Weather, WeatherConfig, apply_weather_effects, process_daily_weather, select_weather_for_today,
};
use dystrail_game::{
    BossConfig, Breakdown, CampConfig, Cart, GameMode, GameState, Grants, PaceCfg, PacingConfig,
    Part, PolicyKind, Region, Store, StoreItem, Vehicle, boss::BossOutcome,
    calculate_effective_price, camp_forage, camp_repair_hack, camp_repair_spare, camp_rest,
    camp_therapy, can_repair, can_therapy, run_boss_minigame,
};
use rand::rngs::mock::StepRng;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

#[test]
fn boss_minigame_exercises_outcomes() {
    let mut pants_cfg = BossConfig::default();
    pants_cfg.rounds = 1;
    pants_cfg.pants_gain_per_round = 10;
    pants_cfg.sanity_loss_per_round = 0;

    let mut pants_state = GameState::default();
    pants_state.stats.pants = 95;
    let outcome = run_boss_minigame(&mut pants_state, &pants_cfg);
    assert_eq!(outcome, BossOutcome::PantsEmergency);

    let mut exhaust_cfg = BossConfig::default();
    exhaust_cfg.rounds = 2;
    exhaust_cfg.pants_gain_per_round = 0;
    exhaust_cfg.sanity_loss_per_round = 7;

    let mut exhaust_state = GameState::default();
    exhaust_state.stats.sanity = 5;
    let outcome = run_boss_minigame(&mut exhaust_state, &exhaust_cfg);
    assert_eq!(outcome, BossOutcome::Exhausted);

    let mut win_cfg = BossConfig::default();
    win_cfg.rounds = 1;
    win_cfg.pants_gain_per_round = 0;
    win_cfg.sanity_loss_per_round = 0;
    win_cfg.max_chance = 0.8;

    let mut win_state = GameState::default();
    win_state.mode = GameMode::Deep;
    win_state.policy = Some(PolicyKind::Aggressive);
    win_state.stats.supplies = 10;
    win_state.stats.pants = 10;
    win_state.day = 180;
    win_state.encounters_resolved = 30;
    win_state.logs.clear();

    let outcome = run_boss_minigame(&mut win_state, &win_cfg);
    assert_eq!(outcome, BossOutcome::PassedCloture);
    assert!(win_state.boss_victory);
    assert!(
        win_state.logs.iter().any(|line| line == "log.boss.compose"),
        "aggressive compose sequence should log when triggered"
    );

    let mut lose_cfg = BossConfig::default();
    lose_cfg.rounds = 1;
    lose_cfg.pants_gain_per_round = 0;
    lose_cfg.sanity_loss_per_round = 0;
    lose_cfg.max_chance = 0.2;
    lose_cfg.base_victory_chance = 0.0;

    let mut lose_state = GameState::default();
    lose_state.logs.clear();
    let rng = ChaCha20Rng::seed_from_u64(12345);
    let preview = rng.clone().random::<u32>() % 100;
    assert!(preview > 20);
    lose_state.rng = Some(rng);

    let outcome = run_boss_minigame(&mut lose_state, &lose_cfg);
    assert_eq!(outcome, BossOutcome::SurvivedFlood);
    assert!(
        lose_state
            .logs
            .iter()
            .any(|line| line == "log.boss.failure"),
        "expected boss failure to be logged"
    );
}

#[test]
fn camp_actions_cover_key_paths() {
    let mut state = GameState::default();
    state.stats.supplies = 6;
    state.stats.hp = 5;
    state.stats.sanity = 5;

    let mut config = CampConfig::default_config();
    config.rest.day = 1;
    config.rest.supplies = -2;
    config.rest.hp = 2;
    config.rest.sanity = 1;
    config.rest.pants = 3;
    config.rest.recovery_day = true;
    config.rest.cooldown_days = 2;

    let rest_outcome = camp_rest(&mut state, &config);
    assert!(rest_outcome.rested);
    assert_eq!(rest_outcome.supplies_delta, -2);
    assert_eq!(state.camp.rest_cooldown, 2);
    assert!(
        state.logs.iter().any(|line| line == "log.camp.rest"),
        "resting should push a log entry"
    );

    let cooldown_result = camp_rest(&mut state, &config);
    assert!(!cooldown_result.rested);
    assert_eq!(cooldown_result.message, "log.camp.rest.cooldown");

    state.camp.rest_cooldown = 0;
    config.rest.day = 0;
    let disabled = camp_rest(&mut state, &config);
    assert!(!disabled.rested);
    assert_eq!(disabled.message, "log.camp.rest.disabled");

    let mut forage_cfg = CampConfig::default_config();
    forage_cfg.forage.day = 1;
    forage_cfg.forage.supplies = 4;
    forage_cfg.forage.cooldown_days = 3;
    forage_cfg
        .forage
        .region_multipliers
        .insert("heartland".into(), 1.5);

    state.region = Region::Heartland;
    state.camp.forage_cooldown = 0;
    let forage = camp_forage(&mut state, &forage_cfg);
    assert_eq!(forage.message, "log.camp.forage");
    assert!(forage.supplies_delta > 0);
    assert_eq!(state.camp.forage_cooldown, 3);

    state.camp.forage_cooldown = 1;
    let forage_cd = camp_forage(&mut state, &forage_cfg);
    assert_eq!(forage_cd.message, "log.camp.forage.cooldown");

    assert_eq!(
        camp_therapy(&mut state, &forage_cfg).message,
        "log.camp.therapy"
    );
    assert_eq!(
        camp_repair_hack(&mut state, &forage_cfg).message,
        "log.camp.repair.hack"
    );
    assert_eq!(
        camp_repair_spare(&mut state, &forage_cfg, Part::Battery).message,
        "log.camp.repair"
    );

    let mut repair_state = GameState::default();
    repair_state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 3,
    });
    assert!(can_repair(&repair_state, &forage_cfg));
    assert!(can_therapy(&state, &forage_cfg));
}

#[test]
fn store_cart_covers_operations() {
    let mut cart = Cart::new();
    assert!(cart.is_empty());
    assert_eq!(cart.add_item("rope", 2), 2);
    assert_eq!(cart.add_item("rope", 1), 3);
    assert_eq!(cart.get_quantity("rope"), 3);
    assert_eq!(cart.remove_item("rope", 1), 2);
    cart.remove_all_item("rope");
    assert!(cart.is_empty());

    cart.add_item("rope", 1);
    cart.clear();
    assert!(cart.is_empty());

    let grants = Grants {
        supplies: 2,
        credibility: 1,
        spare_tire: 1,
        spare_battery: 0,
        spare_alt: 0,
        spare_pump: 0,
        enabled: true,
    };

    let gear_item = StoreItem {
        id: "rope".into(),
        name: "Rope".into(),
        desc: "Sturdy rope".into(),
        price_cents: 1_200,
        unique: false,
        max_qty: 5,
        grants: grants.clone(),
        tags: vec!["gear".into()],
        category: "supplies".into(),
    };

    let spare_item = StoreItem {
        id: "map".into(),
        name: "Map".into(),
        desc: "Shows the way".into(),
        price_cents: 900,
        unique: true,
        max_qty: 1,
        grants,
        tags: vec!["navigation".into()],
        category: "misc".into(),
    };

    let store = Store {
        categories: vec![StoreCategory {
            id: "supplies".into(),
            name: "Supplies".into(),
            items: vec![gear_item.clone()],
        }],
        items: vec![spare_item.clone()],
    };

    assert!(store.find_item("rope").is_some());
    assert!(store.find_item("map").is_some());
    let items = store.items_by_id();
    assert_eq!(items.len(), 2);
    assert_eq!(items.get("rope").unwrap().name, "Rope");

    assert_eq!(calculate_effective_price(1_000, 0.0), 1_000);
    assert_eq!(calculate_effective_price(1_000, 15.0), 850);
    assert_eq!(calculate_effective_price(999, 12.5), 875);
}

#[test]
fn vehicle_system_behaviour() {
    let mut vehicle = Vehicle::default();
    vehicle.apply_damage(30.0);
    assert!(vehicle.health <= 70.0);
    vehicle.repair(15.0);
    assert!(vehicle.health <= 100.0);
    vehicle.ensure_health_floor(95.0);
    assert!(vehicle.health >= 95.0);
    vehicle.reset_wear();
    vehicle.set_wear(5.0);
    let wear_applied = vehicle.apply_scaled_wear(4.0);
    assert!(wear_applied > 0.0);

    vehicle.set_wear_multiplier(-1.0);
    assert_eq!(vehicle.wear_multiplier, 0.0);
    vehicle.clear_wear_multiplier();
    assert_eq!(vehicle.wear_multiplier, 1.0);

    vehicle.set_breakdown_cooldown(2);
    vehicle.tick_breakdown_cooldown();
    assert!(vehicle.breakdown_suppressed());
    vehicle.tick_breakdown_cooldown();
    assert!(!vehicle.breakdown_suppressed());

    let mut rng = StepRng::new(0, 1);
    let options = [
        (Part::Tire, PartWeights::default().tire),
        (Part::Battery, PartWeights::default().battery),
    ];
    assert_eq!(weighted_pick(&options, &mut rng), Some(Part::Tire));
    let none_pick = weighted_pick::<Part, _>(&[], &mut StepRng::new(0, 1));
    assert!(none_pick.is_none());

    assert!(dystrail_game::vehicle::breakdown_roll(
        1.0,
        &mut StepRng::new(0, 1)
    ));
    assert!(!dystrail_game::vehicle::breakdown_roll(
        0.0,
        &mut StepRng::new(0, 1)
    ));

    let mut state = GameState::default();
    process_daily_breakdown(&mut state, &mut StepRng::new(0, 1));
    assert!(state.breakdown.is_some());
    assert!(state.travel_blocked);
}

#[test]
fn pacing_accessors_are_resilient() {
    let pace = PaceCfg {
        id: "steady".into(),
        name: "Steady".into(),
        dist_mult: 1.0,
        distance: 12.0,
        sanity: 0,
        pants: 0,
        encounter_chance_delta: 0.0,
    };

    let cfg = PacingConfig {
        pace: vec![pace.clone()],
        diet: vec![dystrail_game::DietCfg {
            id: "quiet".into(),
            name: "Quiet".into(),
            sanity: 1,
            pants: -1,
            receipt_find_pct_delta: 2,
        }],
        limits: dystrail_game::PacingLimits {
            encounter_base: 0.25,
            distance_base: 10.0,
            distance_penalty_floor: 0.6,
            encounter_floor: 0.05,
            encounter_ceiling: 0.95,
            pants_floor: -10,
            pants_ceiling: 100,
            passive_relief: 0,
            passive_relief_threshold: 0,
            boss_pants_cap: 3,
            boss_passive_relief: 0,
        },
        enabled: true,
    };

    assert_eq!(cfg.get_pace_safe("heated").id, "steady");
    assert_eq!(cfg.get_diet_safe("missing").id, "quiet");
}

#[test]
fn weather_effects_and_selection() {
    let mut cfg = WeatherConfig::default_config();
    cfg.limits.max_extreme_streak = 1;

    let mut state = GameState::default();
    state.region = Region::Heartland;
    state.season = Season::Summer;
    state.rng = Some(ChaCha20Rng::seed_from_u64(2));
    state.inventory.tags.clear();
    state.stats.sanity = 4;
    state.stats.hp = 3;
    state.stats.pants = 10;
    state.weather_state.today = Weather::HeatWave;
    state.weather_state.yesterday = Weather::HeatWave;
    state.exposure_streak_heat = 2;

    apply_weather_effects(&mut state, &cfg);
    assert!(
        state
            .logs
            .iter()
            .any(|line| line == "log.weather.heatstroke"),
        "prolonged heat without gear should trigger heatstroke"
    );

    state.features.exposure_streaks = false;
    state.weather_state.today = Weather::ColdSnap;
    state.weather_state.yesterday = Weather::ColdSnap;
    state.exposure_streak_cold = 2;
    apply_weather_effects(&mut state, &cfg);
    assert!(
        state.logs.iter().any(|line| line == "log.weather.exposure"),
        "cold streak without mitigation should log exposure"
    );

    let selected = select_weather_for_today(&mut state, &cfg).expect("weather selection works");
    assert!(cfg.effects.contains_key(&selected));

    let prior_today = state.weather_state.today;
    process_daily_weather(&mut state, &cfg);
    assert_eq!(state.weather_state.yesterday, prior_today);
}

#[test]
fn weather_config_validation_flags_missing_data() {
    let invalid = r#"{
        "limits": {
            "max_extreme_streak": 1,
            "encounter_cap": 1.0,
            "pants_floor": 0,
            "pants_ceiling": 100
        },
        "effects": {
            "Clear": { "supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0, "travel_mult": 1.0 }
        },
        "mitigation": {},
        "weights": {
            "Heartland": { "Clear": 1 },
            "RustBelt": { "Clear": 1 },
            "Beltway": { "Clear": 1 }
        },
        "exec_mods": {}
    }"#;

    assert!(WeatherConfig::from_json(invalid).is_err());
}

#[test]
fn encounter_data_static_loader_is_empty() {
    let encounters = dystrail_game::EncounterData::load_from_static();
    assert!(encounters.encounters.is_empty());
}
