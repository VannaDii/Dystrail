use std::collections::HashMap;
use std::rc::Rc;

use dystrail_game::GameState;
use dystrail_game::journey::RngBundle;
use dystrail_game::state::Region;
use dystrail_game::weather::WeatherState;
use dystrail_game::weather::{
    Weather, WeatherConfig, apply_weather_effects, process_daily_weather, select_weather_for_today,
};

#[test]
fn weather_selection_and_effects_cover_branches() {
    let mut cfg = WeatherConfig::default_config();
    cfg.limits.max_extreme_streak = 1;

    let rng = Rc::new(RngBundle::from_user_seed(1));
    let mut gs = GameState {
        region: Region::Heartland,
        weather_state: WeatherState {
            today: Weather::Storm,
            yesterday: Weather::Storm,
            extreme_streak: 1,
            ..WeatherState::default()
        },
        ..GameState::default()
    };
    gs.attach_rng_bundle(rng.clone());

    let picked = select_weather_for_today(&mut gs, &cfg, rng.as_ref()).unwrap();
    gs.weather_state.today = picked;
    apply_weather_effects(&mut gs, &cfg);
    process_daily_weather(&mut gs, &cfg, Some(rng.as_ref()));
    assert!(
        gs.weather_state.today != dystrail_game::weather::Weather::Storm
            || gs.weather_state.extreme_streak <= cfg.limits.max_extreme_streak
    );
}

#[test]
fn select_weather_is_deterministic_for_same_seed() {
    let cfg = deterministic_config();
    let rng1 = Rc::new(RngBundle::from_user_seed(42));
    let rng2 = Rc::new(RngBundle::from_user_seed(42));
    let mut gs_one = GameState {
        region: Region::Heartland,
        ..GameState::default()
    };
    let mut gs_two = GameState {
        region: Region::Heartland,
        ..GameState::default()
    };
    let weather_one = select_weather_for_today(&mut gs_one, &cfg, rng1.as_ref()).unwrap();
    let weather_two = select_weather_for_today(&mut gs_two, &cfg, rng2.as_ref()).unwrap();
    assert_eq!(
        weather_one, weather_two,
        "weather selection should be seed-stable"
    );
}

#[test]
fn weather_selection_consumes_weather_rng_only() {
    let cfg = deterministic_config();
    let rng = Rc::new(RngBundle::from_user_seed(5));
    let mut gs = GameState {
        region: Region::Heartland,
        ..GameState::default()
    };
    let _ = select_weather_for_today(&mut gs, &cfg, rng.as_ref()).unwrap();

    assert!(rng.weather().draws() > 0);
    assert_eq!(rng.health().draws(), 0);
}

#[test]
fn neutral_buffer_defaults_to_clear_when_no_neutral_weights() {
    let mut cfg = deterministic_config();
    cfg.weights.insert(
        Region::Heartland,
        HashMap::from([
            (Weather::Clear, 0),
            (Weather::Smoke, 0),
            (Weather::Storm, 10),
            (Weather::HeatWave, 5),
            (Weather::ColdSnap, 3),
        ]),
    );
    let rng = Rc::new(RngBundle::from_user_seed(7));
    let mut gs = GameState {
        region: Region::Heartland,
        weather_state: WeatherState {
            neutral_buffer: 2,
            ..WeatherState::default()
        },
        ..GameState::default()
    };
    let weather = select_weather_for_today(&mut gs, &cfg, rng.as_ref()).unwrap();
    assert_eq!(weather, Weather::Clear);
}

#[test]
fn neutral_buffer_prefers_smoke_when_weighted() {
    let mut cfg = deterministic_config();
    cfg.weights.insert(
        Region::Heartland,
        HashMap::from([
            (Weather::Clear, 0),
            (Weather::Smoke, 12),
            (Weather::Storm, 5),
            (Weather::HeatWave, 3),
            (Weather::ColdSnap, 2),
        ]),
    );
    let rng = Rc::new(RngBundle::from_user_seed(9));
    let mut gs = GameState {
        region: Region::Heartland,
        weather_state: WeatherState {
            neutral_buffer: 1,
            ..WeatherState::default()
        },
        ..GameState::default()
    };
    let weather = select_weather_for_today(&mut gs, &cfg, rng.as_ref()).unwrap();
    assert_eq!(weather, Weather::Smoke);
}

fn deterministic_config() -> WeatherConfig {
    let mut cfg = WeatherConfig::default_config();
    cfg.weights.insert(
        Region::Heartland,
        HashMap::from([
            (Weather::Clear, 5),
            (Weather::Storm, 7),
            (Weather::HeatWave, 3),
            (Weather::ColdSnap, 2),
            (Weather::Smoke, 4),
        ]),
    );
    cfg
}
