#![allow(clippy::field_reassign_with_default)]

use dystrail_game::GameState;
use dystrail_game::journey::RngBundle;
use dystrail_game::weather::{
    WeatherConfig, apply_weather_effects, process_daily_weather, select_weather_for_today,
};
use std::rc::Rc;

#[test]
fn weather_selection_and_effects_cover_branches() {
    let mut cfg = WeatherConfig::default_config();
    cfg.limits.max_extreme_streak = 1;

    let rng = Rc::new(RngBundle::from_user_seed(1));
    let mut gs = GameState::default();
    gs.region = dystrail_game::state::Region::Heartland;
    gs.attach_rng_bundle(rng.clone());
    gs.weather_state.today = dystrail_game::weather::Weather::Storm;
    gs.weather_state.yesterday = dystrail_game::weather::Weather::Storm;
    gs.weather_state.extreme_streak = 1;

    let picked = select_weather_for_today(&mut gs, &cfg, rng.as_ref()).unwrap();
    gs.weather_state.today = picked;
    apply_weather_effects(&mut gs, &cfg);
    process_daily_weather(&mut gs, &cfg, Some(rng.as_ref()));
    assert!(
        gs.weather_state.today != dystrail_game::weather::Weather::Storm
            || gs.weather_state.extreme_streak <= cfg.limits.max_extreme_streak
    );
}
