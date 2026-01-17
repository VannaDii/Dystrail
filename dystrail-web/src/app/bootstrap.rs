use crate::app::state::AppState;
use crate::game::load_result_config;
use yew::prelude::*;

#[hook]
pub fn use_bootstrap(app_state: &AppState) {
    let data = app_state.data.clone();
    let pacing_config = app_state.pacing_config.clone();
    let endgame_config = app_state.endgame_config.clone();
    let weather_config = app_state.weather_config.clone();
    let preload_progress = app_state.preload_progress.clone();
    let boot_ready = app_state.boot_ready.clone();
    let camp_config = app_state.camp_config.clone();
    let crossing_config = app_state.crossing_config.clone();
    let result_config = app_state.result_config.clone();

    use_effect_with((), move |()| {
        #[cfg(not(test))]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let mut progress = 0_u8;
                let mut bump = |p: &UseStateHandle<u8>| {
                    progress = progress.saturating_add(9);
                    p.set(progress.min(99));
                };
                let loaded_data = crate::game::data::EncounterData::load_from_static();
                bump(&preload_progress);
                let loaded_pacing = crate::game::pacing::PacingConfig::load_from_static();
                bump(&preload_progress);
                let loaded_endgame = crate::game::endgame::EndgameTravelCfg::default_config();
                bump(&preload_progress);
                let loaded_weather = crate::game::weather::WeatherConfig::load_from_static();
                bump(&preload_progress);
                let loaded_camp = crate::game::CampConfig::load_from_static();
                bump(&preload_progress);
                let loaded_crossings = serde_json::from_str::<crate::game::CrossingConfig>(
                    include_str!("../../static/assets/data/crossings.json"),
                )
                .unwrap_or_default();
                bump(&preload_progress);
                let loaded_result = load_result_config().unwrap_or_default();
                bump(&preload_progress);
                let _ = serde_json::from_str::<crate::game::store::Store>(include_str!(
                    "../../static/assets/data/store.json"
                ));
                bump(&preload_progress);
                let _ = crate::game::personas::PersonasList::from_json(include_str!(
                    "../../static/assets/data/personas.json"
                ));
                bump(&preload_progress);
                let _ = serde_json::from_str::<crate::game::vehicle::VehicleConfig>(include_str!(
                    "../../static/assets/data/vehicle.json"
                ));
                bump(&preload_progress);
                let _ = serde_json::from_str::<crate::game::boss::BossConfig>(include_str!(
                    "../../static/assets/data/boss.json"
                ));
                bump(&preload_progress);
                data.set(loaded_data);
                pacing_config.set(loaded_pacing);
                endgame_config.set(loaded_endgame);
                weather_config.set(loaded_weather);
                camp_config.set(loaded_camp);
                crossing_config.set(loaded_crossings);
                result_config.set(loaded_result);
                preload_progress.set(100);
                boot_ready.set(true);
            });
        }
        #[cfg(test)]
        {
            let loaded_data = crate::game::data::EncounterData::load_from_static();
            let loaded_pacing = crate::game::pacing::PacingConfig::load_from_static();
            let loaded_endgame = crate::game::endgame::EndgameTravelCfg::default_config();
            let loaded_weather = crate::game::weather::WeatherConfig::load_from_static();
            let loaded_camp = crate::game::CampConfig::load_from_static();
            let loaded_crossings = serde_json::from_str::<crate::game::CrossingConfig>(
                include_str!("../../static/assets/data/crossings.json"),
            )
            .unwrap_or_default();
            let loaded_result = load_result_config().unwrap_or_default();
            data.set(loaded_data);
            pacing_config.set(loaded_pacing);
            endgame_config.set(loaded_endgame);
            weather_config.set(loaded_weather);
            camp_config.set(loaded_camp);
            crossing_config.set(loaded_crossings);
            result_config.set(loaded_result);
            preload_progress.set(100);
            boot_ready.set(true);
        }
        || {}
    });
}
