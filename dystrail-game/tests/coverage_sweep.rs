use dystrail_game::TravelDayKind;
use dystrail_game::boss::{BossConfig, BossOutcome, run_boss_minigame};
use dystrail_game::camp::{self, CampConfig, CampState, RestConfig};
use dystrail_game::crossings::{
    CrossingConfig, CrossingContext, CrossingKind, CrossingResult, apply_bribe, apply_detour,
    apply_permit, resolve_crossing,
};
use dystrail_game::data::{Encounter as EncounterDef, EncounterData};
use dystrail_game::disease::{DiseaseCatalog, DiseaseDef, DiseaseEffects, DiseaseKind};
use dystrail_game::encounters::{EncounterRequest, pick_encounter};
use dystrail_game::endgame::{
    self, EndgamePolicyCfg, EndgameState, EndgameTravelCfg, ResourceKind,
};
use dystrail_game::journey::{Event, EventId, RngBundle, UiSurfaceHint};
use dystrail_game::otdeluxe_random_events::{
    OtDeluxeRandomEventCatalog, OtDeluxeRandomEventContext, OtDeluxeRandomEventDef,
    OtDeluxeRandomEventVariant, pick_random_event_with_trace,
};
use dystrail_game::otdeluxe_state::{OtDeluxeAfflictionKind, OtDeluxeCalendar, OtDeluxePartyState};
use dystrail_game::pacing::PacingConfig;
use dystrail_game::personas::PersonasList;
use dystrail_game::result::{ResultConfig, ResultSummary, result_summary, select_ending};
use dystrail_game::seed::{
    decode_to_seed, encode_friendly, generate_code_from_entropy, parse_share_code,
};
use dystrail_game::state::{
    CollapseCause, Ending, GameMode, GameState, PaceId, PolicyKind, RecentEncounter, Region, Season,
};
use dystrail_game::store::{Cart, Grants, StoreItem};
use dystrail_game::vehicle::{Breakdown, Part, PartWeights, Vehicle, weighted_pick};
use dystrail_game::weather::{
    DystrailRegionalWeather, Weather, WeatherConfig, WeatherModel, apply_weather_effects,
    process_daily_weather, select_weather_for_today,
};
use dystrail_game::{DataLoader, GameEngine, GameStorage};
use dystrail_game::{
    DayRecord, JourneyCfg, JourneyController, MechanicalPolicyId, PolicyId, StrategyId,
    compute_day_ledger_metrics,
};
use rand::rngs::SmallRng;
use rand::rngs::mock::StepRng;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

fn empty_state() -> GameState {
    let data = EncounterData::empty();
    GameState::default().with_seed(0x1234_5678, GameMode::Deep, data)
}

fn load_encounter_data() -> EncounterData {
    EncounterData::from_json(include_str!(
        "../../dystrail-web/static/assets/data/game.json"
    ))
    .unwrap()
}

fn load_crossing_config() -> CrossingConfig {
    serde_json::from_str(include_str!(
        "../../dystrail-web/static/assets/data/crossings.json"
    ))
    .unwrap()
}

fn load_store() -> dystrail_game::store::Store {
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

fn journey_cfg_for(policy: PolicyKind, mode: GameMode) -> JourneyCfg {
    let policy_id = if mode.is_deep() {
        PolicyId::Deep
    } else {
        PolicyId::Classic
    };
    let strategy = match policy {
        PolicyKind::Balanced => StrategyId::Balanced,
        PolicyKind::Aggressive => StrategyId::Aggressive,
        PolicyKind::Conservative => StrategyId::Conservative,
        PolicyKind::ResourceManager => StrategyId::ResourceManager,
    };
    JourneyController::new(MechanicalPolicyId::DystrailLegacy, policy_id, strategy, 0)
        .config()
        .clone()
}

fn encounter_def(id: &str, regions: &[&str]) -> EncounterDef {
    EncounterDef {
        id: id.to_string(),
        name: format!("Encounter {id}"),
        desc: String::new(),
        weight: 1,
        regions: regions.iter().map(|region| (*region).to_string()).collect(),
        modes: vec![String::from("classic")],
        choices: Vec::new(),
        hard_stop: false,
        major_repair: false,
        chainable: false,
    }
}

#[test]
fn boss_outcomes_cover_all_paths() {
    // Pants emergency path.
    let mut pants_state = empty_state();
    pants_state.stats.pants = 96;
    pants_state.policy = Some(PolicyKind::Balanced);
    pants_state.detach_rng_bundle();
    let mut pants_cfg = BossConfig::load_from_static();
    pants_cfg.rounds = 1;
    pants_cfg.pants_gain_per_round = 10;
    pants_cfg.sanity_loss_per_round = 0;
    assert_eq!(
        run_boss_minigame(&mut pants_state, &pants_cfg),
        BossOutcome::PantsEmergency
    );

    // Exhausted branch.
    let mut exhausted_state = empty_state();
    exhausted_state.stats.sanity = 3;
    exhausted_state.detach_rng_bundle();
    let mut exhausted_cfg = BossConfig::load_from_static();
    exhausted_cfg.rounds = 2;
    exhausted_cfg.sanity_loss_per_round = 4;
    exhausted_cfg.pants_gain_per_round = 0;
    assert_eq!(
        run_boss_minigame(&mut exhausted_state, &exhausted_cfg),
        BossOutcome::Exhausted
    );

    // Victory branch with deterministic win roll.
    let mut victory_state = empty_state();
    victory_state.stats.supplies = 20;
    victory_state.stats.morale = 20;
    victory_state.stats.credibility = 20;
    victory_state.stats.allies = 10;
    victory_state.day = 200;
    victory_state.encounters_resolved = 150;
    victory_state
        .receipts
        .extend(vec!["a".into(), "b".into(), "c".into()]);
    victory_state.vehicle_breakdowns = 0;
    victory_state.miles_traveled_actual = 2_400.0;
    victory_state.mode = GameMode::Deep;
    victory_state.policy = Some(PolicyKind::Aggressive);
    victory_state.detach_rng_bundle();
    let mut victory_cfg = BossConfig::load_from_static();
    victory_cfg.rounds = 1;
    victory_cfg.pants_gain_per_round = 0;
    victory_cfg.sanity_loss_per_round = 0;
    victory_cfg.base_victory_chance = 1.0;
    victory_cfg.min_chance = 1.0;
    victory_cfg.max_chance = 1.0;
    assert_eq!(
        run_boss_minigame(&mut victory_state, &victory_cfg),
        BossOutcome::PassedCloture
    );

    // Failure branch by clamping win chance.
    let mut fail_state = empty_state();
    fail_state.stats.supplies = 0;
    fail_state.stats.morale = 0;
    let mut fail_cfg = BossConfig::load_from_static();
    fail_cfg.rounds = 1;
    fail_cfg.pants_gain_per_round = 0;
    fail_cfg.base_victory_chance = 0.0;
    fail_cfg.min_chance = 0.0;
    fail_cfg.max_chance = 0.0;
    assert_eq!(
        run_boss_minigame(&mut fail_state, &fail_cfg),
        BossOutcome::SurvivedFlood
    );
}

#[test]
fn boss_probability_edges_cover_low_and_high() {
    let data = load_encounter_data();

    let mut fail_state = GameState::default().with_seed(0xABCD, GameMode::Deep, data.clone());
    fail_state.stats.supplies = 0;
    fail_state.stats.morale = 0;
    fail_state.stats.credibility = 0;
    fail_state.stats.allies = 0;
    fail_state.stats.sanity = 6;
    fail_state.miles_traveled_actual = 50.0;
    fail_state.encounters_resolved = 0;
    fail_state.receipts.clear();
    let mut fail_cfg = BossConfig::load_from_static();
    fail_cfg.rounds = 0;
    fail_cfg.distance_required = 5_000.0;
    fail_cfg.max_chance = 0.0;
    fail_cfg.min_chance = 0.0;
    fail_cfg.base_victory_chance = 0.0;
    let fail_outcome = run_boss_minigame(&mut fail_state, &fail_cfg);
    assert!(matches!(fail_outcome, BossOutcome::SurvivedFlood));
    assert!(
        fail_state
            .logs
            .last()
            .is_some_and(|log| log.contains("failure"))
    );

    let mut win_state = GameState::default().with_seed(0x1234, GameMode::Deep, data);
    win_state.stats.supplies = 40;
    win_state.stats.morale = 35;
    win_state.stats.credibility = 30;
    win_state.stats.allies = 12;
    win_state.stats.sanity = 10;
    win_state.encounters_resolved = 80;
    win_state
        .receipts
        .extend((0..5).map(|idx| format!("receipt-{idx}")));
    win_state.vehicle_breakdowns = 1;
    win_state.miles_traveled_actual = 2_400.0;
    win_state.detach_rng_bundle();
    let mut win_cfg = BossConfig::load_from_static();
    win_cfg.rounds = 0;
    win_cfg.distance_required = 800.0;
    win_cfg.base_victory_chance = 1.0;
    win_cfg.min_chance = 1.0;
    win_cfg.max_chance = 1.0;
    let win_outcome = run_boss_minigame(&mut win_state, &win_cfg);
    assert!(matches!(win_outcome, BossOutcome::PassedCloture));
    assert!(
        win_state
            .logs
            .last()
            .is_some_and(|log| log.contains("victory"))
    );
}

#[test]
fn camp_actions_cover_branches() {
    let mut state = empty_state();
    state.stats.supplies = 5;
    state.stats.hp = 6;
    state.stats.sanity = 6;
    state.region = Region::Heartland;

    let mut camp_cfg = CampConfig::default_config();
    camp_cfg.rest.day = 1;
    camp_cfg.rest.cooldown_days = 1;
    camp_cfg.rest.supplies = -2;
    camp_cfg.forage.day = 1;
    camp_cfg.forage.supplies = 2;
    camp_cfg
        .forage
        .region_multipliers
        .insert("heartland".into(), 1.5);

    // Disabled rest.
    let mut disabled_cfg = camp_cfg.clone();
    disabled_cfg.rest.day = 0;
    let disabled = camp::camp_rest(&mut state, &disabled_cfg);
    assert!(!disabled.rested);

    // Successful rest followed by cooldown branch.
    let rest = camp::camp_rest(&mut state, &camp_cfg);
    assert!(rest.rested);
    let cooldown = camp::camp_rest(&mut state, &camp_cfg);
    assert!(!cooldown.rested);

    // Forage success and cooldown branch.
    state.camp = CampState::default();
    let forage = camp::camp_forage(&mut state, &camp_cfg);
    assert!(forage.supplies_delta > 0);
    let forage_cd = camp::camp_forage(&mut state, &camp_cfg);
    assert_eq!(forage_cd.message, "log.camp.forage.cooldown");

    assert_eq!(
        camp::camp_therapy(&mut state, &camp_cfg).message,
        "log.camp.therapy"
    );
    assert_eq!(
        camp::camp_repair_hack(&mut state, &camp_cfg).message,
        "log.camp.repair.hack"
    );
    state.inventory.spares.tire = 1;
    state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 0,
    });
    assert_eq!(
        camp::camp_repair_spare(&mut state, &camp_cfg, Part::Tire).message,
        "log.camp.repair"
    );
    assert!(camp::can_repair(&state, &camp_cfg));
    assert!(camp::can_therapy(&state, &camp_cfg));
}

#[test]
fn camp_multi_day_sequences_cover_loops() {
    let mut state = empty_state();
    state.stats.supplies = 8;
    state.stats.hp = 4;
    state.stats.sanity = 5;
    state.region = Region::Heartland;

    let mut cfg = CampConfig::default_config();
    cfg.rest.day = 2;
    cfg.rest.recovery_day = true;
    cfg.rest.cooldown_days = 1;
    cfg.rest.supplies = -3;
    cfg.rest.hp = 2;
    cfg.rest.sanity = 3;
    cfg.forage.day = 2;
    cfg.forage.cooldown_days = 1;
    cfg.forage.supplies = 4;
    cfg.forage
        .region_multipliers
        .insert("Heartland".into(), 1.5);

    let rest = camp::camp_rest(&mut state, &cfg);
    assert!(rest.rested);
    assert_eq!(state.camp.rest_cooldown, 1);
    assert!(state.logs.iter().any(|log| log == "log.camp.rest"));

    // Cooldown prevents immediate repeat.
    let cooldown = camp::camp_rest(&mut state, &cfg);
    assert_eq!(cooldown.message, "log.camp.rest.cooldown");

    // Clear cooldown and rest again to walk the recovery path.
    state.camp.rest_cooldown = 0;
    state.day_state.rest.rest_requested = true;
    let rest_again = camp::camp_rest(&mut state, &cfg);
    assert!(rest_again.rested);
    assert!(!state.day_state.rest.rest_requested);

    // Positive supply path without recovery day hits alternate branch.
    cfg.rest.recovery_day = false;
    cfg.rest.supplies = 2;
    cfg.rest.hp = 0;
    cfg.rest.sanity = 0;
    cfg.rest.pants = 0;
    state.camp.rest_cooldown = 0;
    let rest_positive = camp::camp_rest(&mut state, &cfg);
    assert!(rest_positive.rested);

    // Forage with positive gain and region multiplier.
    state.camp.forage_cooldown = 0;
    let forage = camp::camp_forage(&mut state, &cfg);
    assert!(forage.supplies_delta > 0);
    assert_eq!(state.camp.forage_cooldown, 1);

    // Cooldown path.
    let forage_cooldown = camp::camp_forage(&mut state, &cfg);
    assert_eq!(forage_cooldown.message, "log.camp.forage.cooldown");

    // Negative forage branch ensures rounding and sign handling.
    let mut neg_cfg = cfg.clone();
    neg_cfg.forage.supplies = -2;
    let negative = camp::camp_forage(&mut state, &neg_cfg);
    assert!(negative.supplies_delta <= 0);
}

#[test]
fn journey_controller_tick_yields_day_record() {
    let mut state = empty_state();
    let mut controller = JourneyController::new(
        MechanicalPolicyId::DystrailLegacy,
        PolicyId::Classic,
        StrategyId::Balanced,
        0xD15E_u64,
    );
    controller.configure_state(&mut state);
    let outcome = controller.tick_day(&mut state);

    assert!(!state.day_records.is_empty());
    let record = outcome.record.expect("recorded day");
    assert_eq!(record.day_index, 0);
    assert_eq!(record.kind, TravelDayKind::Travel);
    assert_eq!(state.travel_days, 1);
    let expected_ratio = controller.config().partial_ratio;
    assert!((state.journey_partial_ratio - expected_ratio).abs() < f32::EPSILON);
}

#[test]
fn journey_controller_applies_partial_ratio() {
    let mut state = empty_state();
    let endgame = EndgameTravelCfg::default_config();
    let mut controller = JourneyController::with_config(
        MechanicalPolicyId::DystrailLegacy,
        PolicyId::Classic,
        StrategyId::Balanced,
        JourneyCfg {
            partial_ratio: 0.8,
            ..JourneyCfg::default()
        },
        123,
        endgame,
    );
    controller.configure_state(&mut state);
    let _ = controller.tick_day(&mut state);
    assert!((state.journey_partial_ratio - 0.8).abs() < 1e-6);
    let _ = journey_cfg_for(PolicyKind::Conservative, GameMode::Classic);
    let _ = journey_cfg_for(PolicyKind::ResourceManager, GameMode::Deep);
}

#[test]
fn full_content_walkthrough() {
    let encounters = load_encounter_data();
    let crossings = load_crossing_config();
    let store = load_store();
    let personas = load_personas();
    let camp_cfg = CampConfig::default_config();
    let mut endgame_cfg = EndgameTravelCfg::default_config();
    endgame_cfg.enabled = true;

    for (idx, persona) in personas.iter().enumerate().take(3) {
        let mut state =
            GameState::default().with_seed(0xABC0 + idx as u64, GameMode::Deep, encounters.clone());
        state.apply_persona(persona);
        let mut controller = JourneyController::new(
            MechanicalPolicyId::DystrailLegacy,
            PolicyId::from(state.mode),
            StrategyId::Balanced,
            state.seed,
        );
        controller.set_endgame_config(endgame_cfg.clone());
        controller.configure_state(&mut state);

        for day in 0..80 {
            let outcome = controller.tick_day(&mut state);

            if let Some(enc) = state.current_encounter.clone() {
                for choice_idx in 0..enc.choices.len() {
                    let mut probe = state.clone();
                    probe.current_encounter = Some(enc.clone());
                    probe.apply_choice(choice_idx);
                }
                state.apply_choice(0);
            }

            match day % 7 {
                0 => {
                    let _ = camp::camp_rest(&mut state, &camp_cfg);
                }
                1 => {
                    let _ = camp::camp_forage(&mut state, &camp_cfg);
                }
                2 => {
                    let _ = camp::camp_therapy(&mut state, &camp_cfg);
                }
                _ => {}
            }

            if outcome.ended {
                break;
            }
        }

        for &kind in crossings.types.keys() {
            let _ = apply_bribe(&mut state, &crossings, kind);
            let _ = apply_detour(&mut state, &crossings, kind);
            let _ = apply_permit(&mut state, &crossings, kind);
        }

        for category in &store.categories {
            for item in &category.items {
                state.apply_store_purchase(item.price_cents, &item.grants, &item.tags);
            }
        }

        let _outcome = run_boss_minigame(&mut state, &BossConfig::load_from_static());
        let _summary = result_summary(&state, &ResultConfig::default()).unwrap();
    }
}

#[derive(Default)]
struct TestLoader {
    data: EncounterData,
    configs: HashMap<String, String>,
}

impl dystrail_game::DataLoader for TestLoader {
    type Error = std::convert::Infallible;

    fn load_encounter_data(&self) -> Result<EncounterData, Self::Error> {
        Ok(self.data.clone())
    }

    fn load_config<T>(&self, config_name: &str) -> Result<T, Self::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let configs = &self.configs;
        let json = configs.get(config_name).cloned();
        let json = json.unwrap_or_else(empty_json_string);
        serde_json::from_str(&json).map_or_else(
            |_| Ok(serde_json::from_str("{}").unwrap()),
            |parsed| Ok(parsed),
        )
    }
}

fn empty_json_string() -> String {
    String::from("{}")
}

#[derive(Default)]
struct TestStorage {
    slot: RefCell<Option<GameState>>,
}

impl dystrail_game::GameStorage for TestStorage {
    type Error = std::convert::Infallible;

    fn save_game(&self, _save_name: &str, game_state: &GameState) -> Result<(), Self::Error> {
        self.slot.replace(Some(game_state.clone()));
        Ok(())
    }

    fn load_game(&self, _save_name: &str) -> Result<Option<GameState>, Self::Error> {
        Ok(self.slot.borrow().clone())
    }

    fn delete_save(&self, _save_name: &str) -> Result<(), Self::Error> {
        self.slot.replace(None);
        Ok(())
    }
}

#[test]
fn game_engine_smoke_and_storage_paths() {
    let mut loader = TestLoader::default();
    let _ = loader
        .load_config::<CampConfig>("missing")
        .expect("missing config");
    loader
        .configs
        .insert(String::from("bad"), String::from("{"));
    let _ = loader
        .load_config::<CampConfig>("bad")
        .expect("bad config fallback");
    let config_json = serde_json::to_string(&CampConfig::default()).expect("config json");
    loader.configs.insert(String::from("camp"), config_json);
    loader
        .configs
        .insert(String::from("broken"), String::from("{bad json"));
    let _ = loader
        .load_config::<CampConfig>("camp")
        .expect("camp config");
    let broken = loader
        .load_config::<CampConfig>("broken")
        .expect("broken config");
    assert_eq!(broken, CampConfig::default());
    let storage = TestStorage::default();
    storage.delete_save("slot1").unwrap();
    let engine = GameEngine::new(loader, storage);
    assert!(engine.load_game("missing").unwrap().is_none());
    let game = engine.create_game(42, GameMode::Classic).expect("game");
    let _ = engine
        .create_session(7, GameMode::Classic, StrategyId::Balanced)
        .expect("session");
    engine.save_game("slot1", &game).unwrap();
    let restored = engine.load_game("slot1").unwrap();
    assert!(restored.is_some());
    assert!(engine.load_game("slot1").unwrap().is_some());
}

#[test]
fn legacy_log_event_includes_ui_surface() {
    let event = Event::legacy_log_key(EventId::new(1, 1), 1, "log.test");
    assert_eq!(event.ui_surface_hint, Some(UiSurfaceHint::Log));
}

#[test]
fn pacing_and_personas_cover_fallbacks() {
    let pacing = PacingConfig::default_config();
    let steady = pacing.get_pace_safe("steady");
    assert!(steady.distance >= 0.0);
    let fallback = pacing.get_pace_safe("unknown");
    assert_eq!(fallback.id, PaceId::Steady.as_str());

    let personas = load_personas();
    assert!(personas.get_by_id("organizer").is_some());
    assert!(personas.get_by_id("missing").is_none());
    assert!(PersonasList::from_json("{]").is_err());
}

#[test]
fn seed_and_result_paths_cover_branches() {
    assert!(decode_to_seed("bad").is_none());
    let _friendly = encode_friendly(false, 0xABCD);
    let entropy_code = generate_code_from_entropy(true, 987_654_321);
    assert!(parse_share_code(&entropy_code).is_some());

    let mut state = empty_state();
    state.mode = GameMode::Classic;
    state.stats.hp = 8;
    state.stats.sanity = 7;
    state.stats.pants = 80;
    state.stats.supplies = 5;
    state.stats.morale = 6;
    state.stats.credibility = 4;
    state.stats.allies = 3;
    state.day = 30;
    state.encounters_resolved = 12;
    state.receipts.extend(vec!["a".into(), "b".into()]);
    state.vehicle_breakdowns = 2;
    state.miles_traveled_actual = 800.0;
    state.malnutrition_level = 3;
    state.ending = Some(Ending::BossVictory);

    let cfg = ResultConfig::default();
    let summary: ResultSummary = dystrail_game::result::result_summary(&state, &cfg).unwrap();
    assert_eq!(summary.ending, Ending::BossVictory);

    state.ending = Some(Ending::Collapse {
        cause: CollapseCause::Hunger,
    });
    let collapse = select_ending(&state, &cfg, false);
    assert!(matches!(collapse, Ending::Collapse { .. }));

    state.ending = None;
    let vote_fail = select_ending(&state, &cfg, false);
    assert_eq!(vote_fail, Ending::BossVoteFailed);
}

#[test]
fn vehicle_and_weather_paths() {
    let mut vehicle = Vehicle::default();
    vehicle.apply_damage(10.0);
    vehicle.repair(5.0);
    vehicle.ensure_health_floor(50.0);
    vehicle.set_wear(20.0);
    let wear = vehicle.apply_scaled_wear(5.0);
    assert!(wear >= 0.0);
    vehicle.set_breakdown_cooldown(2);
    vehicle.tick_breakdown_cooldown();
    assert!(vehicle.breakdown_suppressed());
    vehicle.set_wear_multiplier(-1.0);
    vehicle.clear_wear_multiplier();

    let weights = PartWeights::default();
    let mut options = vec![(Part::Tire, weights.tire), (Part::Battery, weights.battery)];
    assert_eq!(Part::Tire.key(), "vehicle.parts.tire");
    assert_eq!(Part::Battery.key(), "vehicle.parts.battery");
    assert_eq!(Part::Alternator.key(), "vehicle.parts.alt");
    assert_eq!(Part::FuelPump.key(), "vehicle.parts.pump");
    let mut rng = SmallRng::seed_from_u64(0);
    assert!(weighted_pick(&options, &mut rng).is_some());
    for (_, weight) in &mut options {
        *weight = 0;
    }
    assert!(weighted_pick(&options, &mut rng).is_none());

    let mut state = empty_state();
    state.weather_state.today = Weather::Storm;
    state.weather_state.extreme_streak = 2;
    state.weather_travel_multiplier = 0.5;
    state.vehicle.wear = 30.0;
    state.vehicle.health = 40.0;
    state.stats.supplies = 2;
    state.stats.sanity = 5;
    state.stats.hp = 6;
    state.features.travel_v2 = true;
    state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(0xDEAD)));

    let cfg = WeatherConfig::load_from_static();
    let model = DystrailRegionalWeather::new(cfg.clone());
    let rng_shared = state
        .rng_bundle
        .as_ref()
        .map(Rc::clone)
        .expect("rng attached for weather");
    let _today = select_weather_for_today(&mut state, &cfg, rng_shared.as_ref());
    let sample = model.sample_from_weather(&state, state.weather_state.today);
    apply_weather_effects(&mut state, &cfg, sample);
    process_daily_weather(&mut state, &model, Some(rng_shared.as_ref()));
}

#[test]
fn endgame_controller_and_failure_guard() {
    let mut state = empty_state();
    state.mode = GameMode::Deep;
    state.policy = Some(PolicyKind::Aggressive);
    state.vehicle.wear = 50.0;
    state.vehicle.health = 80.0;
    state.miles_traveled_actual = 1_980.0;
    state.vehicle_breakdowns = 3;
    state.endgame = EndgameState::default();
    state.endgame.health_floor = 40.0;
    state.endgame.failure_guard_miles = 2_000.0;
    state.endgame.cooldown_days = 1;
    state.budget_cents = 20_000;
    state.budget = 200;

    let mut cfg = EndgameTravelCfg::default_config();
    cfg.enabled = true;
    cfg.policies.insert(
        "deep_aggressive".into(),
        EndgamePolicyCfg {
            mi_start: 1_850.0,
            failure_guard_miles: 1_990.0,
            health_floor: 35.0,
            wear_reset: 5.0,
            cooldown_days: 1,
            partial_ratio: 0.42,
            wear_multiplier: 1.1,
            resource_priority: vec![ResourceKind::MatchingSpare, ResourceKind::Emergency],
            travel_bias: 1.05,
            stop_cap_window: 10,
            stop_cap_max_full: 2,
            breakdown_scale: 0.5,
            wear_shave_ratio: 0.7,
        },
    );
    state.vehicle.set_breakdown_cooldown(2);
    state.day_state.lifecycle.day_initialized = true;
    endgame::run_endgame_controller(&mut state, 100.0, true, &cfg);
    assert!((state.endgame.partial_ratio - 0.42).abs() < f32::EPSILON);
    let _ = endgame::enforce_failure_guard(&mut state);

    cfg.policies.insert(
        "deep_balanced".into(),
        EndgamePolicyCfg {
            mi_start: 1_800.0,
            failure_guard_miles: 1_950.0,
            health_floor: 30.0,
            wear_reset: 4.0,
            cooldown_days: 2,
            partial_ratio: 0.5,
            wear_multiplier: 1.0,
            resource_priority: vec![ResourceKind::Emergency],
            travel_bias: 1.04,
            stop_cap_window: 10,
            stop_cap_max_full: 2,
            breakdown_scale: 0.5,
            wear_shave_ratio: 0.7,
        },
    );
    state.policy = Some(PolicyKind::Balanced);
    state.miles_traveled_actual = 1_860.0;
    endgame::run_endgame_controller(&mut state, 60.0, false, &cfg);

    cfg.policies.remove("deep_balanced");
    let log_count = state.logs.len();
    endgame::run_endgame_controller(&mut state, 60.0, false, &cfg);
    assert_eq!(log_count, state.logs.len());
}

#[test]
fn store_cart_and_inventory_flows() {
    let mut cart = Cart::new();
    assert!(cart.is_empty());
    assert_eq!(cart.add_item("rope", 2), 2);
    assert_eq!(cart.add_item("rope", 1), 3);
    cart.total_cents = 1_200;
    assert!(!cart.is_empty());
    assert_eq!(cart.remove_item("rope", 3), 0);
    assert!(cart.lines.is_empty());
    assert_eq!(cart.remove_item("rope", 1), 0);

    let item = StoreItem {
        id: "grant".into(),
        name: "Grant".into(),
        desc: String::new(),
        price_cents: 1_000,
        unique: false,
        max_qty: 5,
        grants: Grants {
            supplies: 1,
            credibility: 1,
            spare_tire: 1,
            enabled: true,
            ..Grants::default()
        },
        tags: vec!["legal_fund".into()],
        category: "general".into(),
    };

    let mut state = empty_state();
    state.apply_store_purchase(item.price_cents, &item.grants, &item.tags);
    assert!(state.inventory.has_tag("legal_fund"));
}

#[test]
fn state_apply_choice_handles_missing_encounter() {
    let mut state = empty_state();
    state.current_encounter = None;
    state.apply_choice(0);
    assert!(state.current_encounter.is_none());
}

#[test]
fn crossing_resolution_covers_branches() {
    let cfg = load_crossing_config();

    let classic_crossing = journey_cfg_for(PolicyKind::Balanced, GameMode::Classic).crossing;
    let mut permit_rng = FixedRng::with_draw(0.1);
    let permit_ctx = CrossingContext {
        policy: &classic_crossing,
        kind: CrossingKind::Checkpoint,
        has_permit: true,
        bribe_intent: false,
        prior_bribe_attempts: 0,
    };
    let permit = resolve_crossing(permit_ctx, &mut permit_rng);
    assert!(permit.used_permit);
    assert!(matches!(permit.result, CrossingResult::Pass));

    let deep_balanced = journey_cfg_for(PolicyKind::Balanced, GameMode::Deep).crossing;
    let detour_draw = deep_balanced.detour.mul_add(0.5, deep_balanced.pass);
    let mut detour_rng = FixedRng::with_draw(detour_draw);
    let detour_ctx = CrossingContext {
        policy: &deep_balanced,
        kind: CrossingKind::BridgeOut,
        has_permit: false,
        bribe_intent: false,
        prior_bribe_attempts: 0,
    };
    let detour = resolve_crossing(detour_ctx, &mut detour_rng);
    assert!(matches!(detour.result, CrossingResult::Detour(_)));

    let deep_aggressive = journey_cfg_for(PolicyKind::Aggressive, GameMode::Deep).crossing;
    let mut bribe_rng = FixedRng::with_draw(0.05);
    let bribe_ctx = CrossingContext {
        policy: &deep_aggressive,
        kind: CrossingKind::BridgeOut,
        has_permit: false,
        bribe_intent: true,
        prior_bribe_attempts: 0,
    };
    let bribe_outcome = resolve_crossing(bribe_ctx, &mut bribe_rng);
    assert!(bribe_outcome.bribe_attempted);
    assert!(bribe_outcome.bribe_succeeded);

    let mut fail_rng = FixedRng::with_draw(0.99);
    let fail_ctx = CrossingContext {
        policy: &deep_balanced,
        kind: CrossingKind::BridgeOut,
        has_permit: false,
        bribe_intent: true,
        prior_bribe_attempts: 0,
    };
    let fail_outcome = resolve_crossing(fail_ctx, &mut fail_rng);
    assert!(fail_outcome.bribe_attempted);
    assert!(!fail_outcome.bribe_succeeded);

    let mut gs = empty_state();
    gs.inventory.tags.insert("permit".into());
    gs.budget_cents = 2_000;
    gs.budget = 20;
    assert!(dystrail_game::crossings::can_use_permit(
        &gs,
        &CrossingKind::Checkpoint
    ));
    assert!(dystrail_game::crossings::can_afford_bribe(
        &gs,
        &cfg,
        CrossingKind::Checkpoint
    ));
    let cost = dystrail_game::crossings::calculate_bribe_cost(1_200, 10);
    assert!(cost < 1_200);
    let result = apply_bribe(&mut gs, &cfg, CrossingKind::Checkpoint);
    assert_eq!(result, "crossing.result.bribe.success");
    let detour_msg = apply_detour(&mut gs, &cfg, CrossingKind::BridgeOut);
    assert_eq!(detour_msg, "crossing.result.detour.success");
    let permit_msg = apply_permit(&mut gs, &cfg, CrossingKind::BridgeOut);
    assert_eq!(permit_msg, "crossing.result.permit.success");
}

#[test]
fn camp_disabled_and_negative_paths() {
    let mut state = empty_state();
    let cfg = CampConfig::default_config();
    let disabled_rest = CampConfig {
        rest: RestConfig {
            day: 0,
            ..cfg.rest.clone()
        },
        ..cfg.clone()
    };
    assert_eq!(
        camp::camp_rest(&mut state, &disabled_rest).message,
        "log.camp.rest.disabled"
    );

    let mut disabled_forage = cfg.clone();
    disabled_forage.forage.day = 0;
    assert_eq!(
        camp::camp_forage(&mut state, &disabled_forage).message,
        "log.camp.forage.disabled"
    );

    state.inventory.spares.battery = 1;
    state.breakdown = Some(Breakdown {
        part: Part::Battery,
        day_started: 0,
    });
    assert_eq!(
        camp::camp_repair_hack(&mut state, &cfg).message,
        "log.camp.repair.hack"
    );
}

#[test]
fn result_endings_branching() {
    let cfg = ResultConfig::default();
    let mut state = empty_state();

    state.ending = Some(Ending::Exposure {
        kind: dystrail_game::state::ExposureKind::Heat,
    });
    let exposure = result_summary(&state, &cfg).unwrap();
    assert_eq!(exposure.ending_cause.as_deref(), Some("exposure_heat"));

    state.ending = Some(Ending::SanityLoss);
    let sanity = result_summary(&state, &cfg).unwrap();
    assert!(sanity.headline_key.contains("sanity"));

    state.ending = Some(Ending::Collapse {
        cause: CollapseCause::Panic,
    });
    let collapse = result_summary(&state, &cfg).unwrap();
    assert_eq!(collapse.ending_cause.as_deref(), Some("pants"));

    state.ending = None;
    state.mode = GameMode::Classic;
    state.score_mult = 1.0;
    let summary = result_summary(&state, &cfg).unwrap();
    assert_eq!(summary.mode, "Classic");
}

#[test]
fn seed_error_paths_cover_invalid_cases() {
    assert!(decode_to_seed("CL-BAD").is_none());
    assert!(decode_to_seed("CL-").is_none());
    assert!(parse_share_code("INVALID").is_none());
    assert!(generate_code_from_entropy(false, 0).starts_with("CL-"));
    let friendly = encode_friendly(true, (123u64 << 9) | 1);
    assert!(friendly.starts_with("DP-"));
}

#[test]
fn store_pricing_variants() {
    assert_eq!(
        dystrail_game::store::calculate_effective_price(1_000, 0.0),
        1_000
    );
    assert_eq!(
        dystrail_game::store::calculate_effective_price(1_000, -5.0),
        1_000
    );
}

#[test]
fn weather_variants_cover_branches() {
    let cfg = WeatherConfig::default_config();
    let model = DystrailRegionalWeather::new(cfg.clone());
    let mut state = empty_state();
    state.stats.hp = 3;
    state.stats.sanity = 3;
    state.stats.supplies = 1;
    state.vehicle.wear = 40.0;

    for weather in [
        Weather::Clear,
        Weather::Storm,
        Weather::HeatWave,
        Weather::ColdSnap,
        Weather::Smoke,
    ] {
        state.weather_state.today = weather;
        state.weather_state.yesterday = Weather::Clear;
        state.weather_state.extreme_streak = 3;
        state.weather_state.neutral_buffer = 0;
        let sample = model.sample_from_weather(&state, weather);
        apply_weather_effects(&mut state, &cfg, sample);
        state.weather_state.yesterday = weather;
    }

    assert!(Weather::Storm.is_extreme());
    assert_eq!(Weather::ColdSnap.i18n_key(), "weather.states.ColdSnap");
}

#[test]
fn encounter_rotation_and_weights_cover_branches() {
    let data = load_encounter_data();
    let mut queue = VecDeque::new();
    let mut request = dystrail_game::encounters::EncounterRequest {
        region: Region::Heartland,
        is_deep: true,
        malnutrition_level: 3,
        starving: true,
        data: &data,
        recent: &[
            dystrail_game::state::RecentEncounter::new("alpha".into(), 1, Region::Heartland),
            dystrail_game::state::RecentEncounter::new("beta".into(), 2, Region::RustBelt),
        ],
        current_day: 15,
        policy: Some(PolicyKind::Conservative),
        force_rotation: false,
    };
    let mut rng = ChaCha20Rng::seed_from_u64(0);
    let _ = dystrail_game::encounters::pick_encounter(&request, &mut queue, &mut rng);
    request.force_rotation = true;
    let _ = dystrail_game::encounters::pick_encounter(&request, &mut queue, &mut rng);
}

#[test]
fn crossing_config_thresholds_cover_branches() {
    let cfg = CrossingConfig::default();
    for season in [Season::Spring, Season::Summer, Season::Fall, Season::Winter] {
        assert!(
            cfg.thresholds
                .lookup(Region::Heartland, season)
                .cost_multiplier
                >= 100
        );
    }
    let beltway_summer = cfg.thresholds.lookup(Region::Beltway, Season::Summer);
    assert!(beltway_summer.success_adjust <= 0.0);

    let mut gs = empty_state();
    gs.mods.bribe_discount_pct = 10;
    gs.budget_cents = 100;
    gs.budget = 1;
    let failed = apply_bribe(&mut gs, &cfg, CrossingKind::Checkpoint);
    assert_eq!(failed, "crossing.result.bribe.fail");
    gs.budget_cents = 20_000;
    gs.receipts.push("press_pass".into());
    let success = apply_bribe(&mut gs, &cfg, CrossingKind::Checkpoint);
    assert_eq!(success, "crossing.result.bribe.success");
    assert!(dystrail_game::crossings::can_use_permit(
        &gs,
        &CrossingKind::Checkpoint
    ));

    let mut empty_cfg = CrossingConfig::default();
    empty_cfg.types.remove(&CrossingKind::BridgeOut);
    assert!(!dystrail_game::crossings::can_afford_bribe(
        &gs,
        &empty_cfg,
        CrossingKind::BridgeOut
    ));
    assert!(dystrail_game::crossings::calculate_bribe_cost(1_000, 25) < 1_000);

    let cfg = journey_cfg_for(PolicyKind::Balanced, GameMode::Deep);
    let span = u32::from(
        cfg.crossing
            .detour_days
            .max
            .saturating_sub(cfg.crossing.detour_days.min),
    ) + 1;
    let remainder = span.saturating_sub(1);
    let mut detour_policy = cfg.crossing.clone();
    detour_policy.pass = 0.0;
    detour_policy.detour = 1.0;
    detour_policy.terminal = 0.0;
    detour_policy.sanitize();
    let sample = if span > 0 {
        sample_with_remainder(0.9, span, remainder)
    } else {
        sample_for(0.9)
    };
    let mut rng = FixedRng::with_value(sample);
    let detour_ctx = CrossingContext {
        policy: &detour_policy,
        kind: CrossingKind::BridgeOut,
        has_permit: false,
        bribe_intent: false,
        prior_bribe_attempts: 0,
    };
    let detour_outcome = resolve_crossing(detour_ctx, &mut rng);
    assert!(
        matches!(
            detour_outcome.result,
            CrossingResult::Detour(days) if days == cfg.crossing.detour_days.max
        ),
        "expected detour days to reach configured maximum"
    );
}

#[test]
fn day_accounting_transition_matrix_covers_edges() {
    let records = vec![
        DayRecord::new(0, TravelDayKind::NonTravel, 0.0),
        DayRecord::new(1, TravelDayKind::Partial, 1.0),
        DayRecord::new(2, TravelDayKind::Travel, 5.0),
        DayRecord::new(3, TravelDayKind::NonTravel, 0.0),
        DayRecord::new(4, TravelDayKind::Travel, 4.0),
        DayRecord::new(5, TravelDayKind::Partial, 1.5),
        DayRecord::new(6, TravelDayKind::NonTravel, 0.0),
    ];
    let metrics = compute_day_ledger_metrics(&records);
    assert_eq!(metrics.non_travel_days, 3);
    assert_eq!(metrics.partial_days, 2);
    assert_eq!(metrics.travel_days, 2);
}
fn sample_for(draw: f32) -> u32 {
    let clamped = draw.clamp(0.0, 1.0 - f32::EPSILON);
    let denom = f64::from(u32::MAX) + 1.0;
    let value = f64::from(clamped).mul_add(denom, -0.5).max(0.0);
    value.floor().to_string().parse::<u32>().unwrap_or(u32::MAX)
}

fn sample_with_remainder(draw: f32, span: u32, remainder: u32) -> u32 {
    if span == 0 {
        return sample_for(draw);
    }
    let span_u64 = u64::from(span);
    let base = u64::from(sample_for(draw));
    let bucket = base / span_u64;
    let rem = u64::from(remainder % span);
    let mut candidate = bucket.saturating_mul(span_u64).saturating_add(rem);
    if candidate > u64::from(u32::MAX) && bucket > 0 {
        candidate = (bucket - 1)
            .saturating_mul(span_u64)
            .saturating_add(rem)
            .min(u64::from(u32::MAX));
    }
    u32::try_from(candidate.min(u64::from(u32::MAX))).unwrap_or(u32::MAX)
}

#[derive(Clone)]
struct FixedRng {
    value: u32,
}

impl FixedRng {
    fn with_draw(draw: f32) -> Self {
        Self {
            value: sample_for(draw),
        }
    }

    const fn with_value(value: u32) -> Self {
        Self { value }
    }
}

impl RngCore for FixedRng {
    fn next_u32(&mut self) -> u32 {
        self.value
    }

    fn next_u64(&mut self) -> u64 {
        u64::from(self.value)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let bytes = self.value.to_le_bytes();
        for (idx, slot) in dest.iter_mut().enumerate() {
            *slot = bytes[idx % bytes.len()];
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[test]
fn helpers_cover_fixed_rng_and_sampling_edges() {
    let direct = sample_with_remainder(0.75, 0, 0);
    assert_eq!(direct, sample_for(0.75));

    let mut rng = FixedRng::with_value(0xA5A5_A5A5);
    assert_eq!(rng.next_u64(), 0xA5A5_A5A5_u64);
    let mut bytes = [0_u8; 4];
    rng.fill_bytes(&mut bytes);
    assert!(bytes.iter().any(|byte| *byte != 0));
    let mut bytes = [0_u8; 3];
    rng.try_fill_bytes(&mut bytes).expect("fill bytes");
}

#[test]
fn sample_with_remainder_caps_overflow() {
    let draw = 1.0;
    let base = sample_for(draw);
    let mut chosen = None;
    for span in 2..=10_000_u32 {
        let bucket = u64::from(base) / u64::from(span);
        if bucket == 0 {
            continue;
        }
        let remainder = span - 1;
        let overflowed = bucket
            .saturating_mul(u64::from(span))
            .saturating_add(u64::from(remainder));
        if overflowed > u64::from(u32::MAX) {
            chosen = Some((span, remainder, bucket, overflowed));
            break;
        }
    }
    let (span, remainder, bucket, overflowed) = chosen.expect("expected overflow parameters");
    assert!(overflowed > u64::from(u32::MAX));

    let expected = (bucket - 1)
        .saturating_mul(u64::from(span))
        .saturating_add(u64::from(remainder))
        .min(u64::from(u32::MAX));
    let expected = u32::try_from(expected).unwrap_or(u32::MAX);
    let capped = sample_with_remainder(draw, span, remainder);
    assert_eq!(capped, expected);
}

#[test]
fn otdeluxe_party_state_affliction_records_disease() {
    let mut party = OtDeluxePartyState::from_names(["Ada", "Lin"]);
    assert_eq!(party.alive_count(), 2);
    let mut rng = StepRng::new(0, 0);
    let outcome = party
        .apply_affliction_random(&mut rng, OtDeluxeAfflictionKind::Illness, 3, Some("flu"))
        .expect("affliction outcome");
    assert_eq!(outcome.disease_id.as_deref(), Some("flu"));
    assert!(party.members[outcome.member_index].is_sick());
    assert_eq!(
        party.members[outcome.member_index].illness_id.as_deref(),
        Some("flu")
    );
}

#[test]
fn otdeluxe_calendar_normalizes_and_handles_leap_years() {
    let mut invalid = OtDeluxeCalendar {
        month: 0,
        day_in_month: 0,
        year: 1849,
    };
    invalid.advance_days(1);
    assert_eq!(invalid.month, 1);
    assert_eq!(invalid.day_in_month, 2);

    let mut overflow = OtDeluxeCalendar {
        month: 2,
        day_in_month: 99,
        year: 1900,
    };
    overflow.advance_days(1);
    assert_eq!(overflow.month, 3);
    assert_eq!(overflow.day_in_month, 1);

    let mut leap = OtDeluxeCalendar {
        month: 2,
        day_in_month: 28,
        year: 2000,
    };
    leap.advance_days(1);
    assert_eq!(leap.month, 2);
    assert_eq!(leap.day_in_month, 29);
}

#[test]
fn test_loader_config_lookup_handles_missing_and_present_entries() {
    let mut loader = TestLoader::default();
    loader
        .configs
        .insert("camp".into(), "{\"enabled\": true}".into());

    let present: CampConfig = loader.load_config("camp").expect("config loads");
    assert!(present.enabled);

    let missing: CampConfig = loader.load_config("missing").expect("fallback config");
    assert!(!missing.enabled);
    assert_eq!(missing.rest.day, 0);
    assert_eq!(missing.forage.day, 0);
}

#[test]
fn disease_uniform_fallback_trace_records_weights() {
    let catalog = DiseaseCatalog {
        diseases: vec![
            DiseaseDef {
                id: "d1".into(),
                kind: DiseaseKind::Illness,
                display_key: "disease.d1".into(),
                weight: 0,
                duration_days: None,
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects::default(),
                fatality_model: None,
                tags: Vec::new(),
            },
            DiseaseDef {
                id: "d2".into(),
                kind: DiseaseKind::Illness,
                display_key: "disease.d2".into(),
                weight: 0,
                duration_days: None,
                onset_effects: DiseaseEffects::default(),
                daily_tick_effects: DiseaseEffects::default(),
                fatality_model: None,
                tags: Vec::new(),
            },
        ],
    };
    let mut rng = SmallRng::seed_from_u64(42);
    let (_, trace) = catalog.pick_by_kind_with_trace(DiseaseKind::Illness, &mut rng);
    let trace = trace.expect("trace");
    for candidate in &trace.candidates {
        assert_eq!(candidate.multipliers.len(), 1);
        assert_eq!(candidate.multipliers[0].label, "uniform_fallback");
        assert!((candidate.final_weight - 1.0).abs() <= f64::EPSILON);
    }
}

#[test]
fn pick_encounter_returns_ready_rotation_candidate() {
    let data = EncounterData::from_encounters(vec![encounter_def("alpha", &["Heartland"])]);
    let request = EncounterRequest {
        region: Region::Heartland,
        is_deep: false,
        malnutrition_level: 0,
        starving: false,
        data: &data,
        recent: &[],
        current_day: 10,
        policy: None,
        force_rotation: false,
    };
    let mut queue = VecDeque::from([String::from("alpha")]);
    let mut rng = ChaCha20Rng::seed_from_u64(1);
    let pick = pick_encounter(&request, &mut queue, &mut rng);
    assert!(pick.rotation_satisfied);
    assert!(pick.encounter.is_some());
}

#[test]
fn pick_encounter_returns_forced_rotation_candidate() {
    let data = EncounterData::from_encounters(vec![encounter_def("alpha", &["Heartland"])]);
    let recent = [RecentEncounter::new(
        String::from("alpha"),
        10,
        Region::Heartland,
    )];
    let request = EncounterRequest {
        region: Region::Heartland,
        is_deep: false,
        malnutrition_level: 0,
        starving: false,
        data: &data,
        recent: &recent,
        current_day: 10,
        policy: None,
        force_rotation: true,
    };
    let mut queue = VecDeque::from([String::from("alpha")]);
    let mut rng = ChaCha20Rng::seed_from_u64(2);
    let pick = pick_encounter(&request, &mut queue, &mut rng);
    assert!(pick.rotation_satisfied);
    assert!(pick.encounter.is_some());
}

#[test]
fn pick_encounter_weighted_selection_records_trace() {
    let data = EncounterData::from_encounters(vec![
        encounter_def("alpha", &["Heartland"]),
        encounter_def("beta", &["Heartland"]),
    ]);
    let recent = [
        RecentEncounter::new(String::from("alpha"), 10, Region::Heartland),
        RecentEncounter::new(String::from("beta"), 10, Region::Heartland),
    ];
    let request = EncounterRequest {
        region: Region::Heartland,
        is_deep: false,
        malnutrition_level: 0,
        starving: false,
        data: &data,
        recent: &recent,
        current_day: 10,
        policy: None,
        force_rotation: false,
    };
    let mut queue = VecDeque::new();
    let mut rng = ChaCha20Rng::seed_from_u64(3);
    let pick = pick_encounter(&request, &mut queue, &mut rng);
    assert!(pick.encounter.is_some());
    assert!(pick.decision_trace.is_some());
}

#[test]
fn pick_random_event_with_variant_emits_trace() {
    let catalog = OtDeluxeRandomEventCatalog {
        chance_per_day: 1.0,
        events: vec![OtDeluxeRandomEventDef {
            id: "resource_shortage".into(),
            weight: 1,
            variants: vec![OtDeluxeRandomEventVariant {
                id: "bad_water".into(),
                weight: 1,
            }],
        }],
    };
    let ctx = OtDeluxeRandomEventContext {
        season: Season::Spring,
        food_lbs: 200,
        oxen_total: 4,
        party_alive: 4,
        health_general: 100,
        spares_total: 1,
        weight_mult: 1.0,
        weight_cap: None,
    };
    let mut rng = StepRng::new(0, 0);
    let pick = pick_random_event_with_trace(&catalog, &ctx, &mut rng).expect("pick");
    assert!(pick.selection.variant_id.is_some());
    assert!(pick.variant_trace.is_some());
}

#[test]
fn select_weather_for_today_errors_when_region_missing_in_config() {
    let mut state = GameState {
        region: Region::RustBelt,
        ..GameState::default()
    };
    let mut cfg = WeatherConfig::default_config();
    cfg.weights.clear();
    let rng_bundle = RngBundle::from_user_seed(11);
    let err = select_weather_for_today(&mut state, &cfg, &rng_bundle).unwrap_err();
    assert!(err.contains("RustBelt"));
}

#[test]
fn select_weather_for_today_applies_neutral_buffer_on_heatwave_streak() {
    let mut state = GameState {
        region: Region::Heartland,
        ..GameState::default()
    };
    state.weather_state.heatwave_streak = 99;
    let mut cfg = WeatherConfig::default_config();
    cfg.weights
        .insert(Region::Heartland, HashMap::from([(Weather::HeatWave, 5)]));
    let rng_bundle = RngBundle::from_user_seed(12);
    let selected = select_weather_for_today(&mut state, &cfg, &rng_bundle).expect("weather");
    assert!(state.weather_state.neutral_buffer > 0);
    assert_ne!(selected, Weather::HeatWave);
}
