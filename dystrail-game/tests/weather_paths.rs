#![allow(clippy::field_reassign_with_default)]

use dystrail_game::GameState;
use dystrail_game::weather::{
    WeatherConfig, apply_weather_effects, process_daily_weather, select_weather_for_today,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[test]
fn weather_selection_and_effects_cover_branches() {
    let mut cfg = WeatherConfig::default_config();
    cfg.limits.max_extreme_streak = 1;

    let mut gs = GameState::default();
    gs.region = dystrail_game::state::Region::Heartland;
    gs.rng = Some(ChaCha20Rng::from_seed([1_u8; 32]));
    gs.weather_state.today = dystrail_game::weather::Weather::Storm;
    gs.weather_state.yesterday = dystrail_game::weather::Weather::Storm;
    gs.weather_state.extreme_streak = 1;

    let picked = select_weather_for_today(&mut gs, &cfg).unwrap();
    gs.weather_state.today = picked;
    apply_weather_effects(&mut gs, &cfg);
    process_daily_weather(&mut gs, &cfg);
    assert!(
        gs.weather_state.today != dystrail_game::weather::Weather::Storm
            || gs.weather_state.extreme_streak <= cfg.limits.max_extreme_streak
    );
}
