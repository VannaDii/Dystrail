#[cfg(any(target_arch = "wasm32", test))]
use crate::app::state::AppState;
#[cfg(any(target_arch = "wasm32", test))]
use crate::game::load_result_config;
#[cfg(any(target_arch = "wasm32", test))]
use yew::prelude::*;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone)]
struct BootstrapHandles {
    data: UseStateHandle<crate::game::data::EncounterData>,
    pacing_config: UseStateHandle<crate::game::pacing::PacingConfig>,
    endgame_config: UseStateHandle<crate::game::endgame::EndgameTravelCfg>,
    weather_config: UseStateHandle<crate::game::weather::WeatherConfig>,
    camp_config: UseStateHandle<crate::game::CampConfig>,
    crossing_config: UseStateHandle<crate::game::CrossingConfig>,
    result_config: UseStateHandle<crate::game::ResultConfig>,
    preload_progress: UseStateHandle<u8>,
    boot_ready: UseStateHandle<bool>,
}

#[cfg(any(target_arch = "wasm32", test))]
fn handles_from_state(app_state: &AppState) -> BootstrapHandles {
    BootstrapHandles {
        data: app_state.data.clone(),
        pacing_config: app_state.pacing_config.clone(),
        endgame_config: app_state.endgame_config.clone(),
        weather_config: app_state.weather_config.clone(),
        camp_config: app_state.camp_config.clone(),
        crossing_config: app_state.crossing_config.clone(),
        result_config: app_state.result_config.clone(),
        preload_progress: app_state.preload_progress.clone(),
        boot_ready: app_state.boot_ready.clone(),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn bootstrap_load(handles: &BootstrapHandles) {
    let mut progress = 0_u8;
    let mut bump = |p: &UseStateHandle<u8>| {
        progress = progress.saturating_add(9);
        p.set(progress.min(99));
    };
    let loaded_data = crate::game::data::EncounterData::load_from_static();
    bump(&handles.preload_progress);
    let loaded_pacing = crate::game::pacing::PacingConfig::load_from_static();
    bump(&handles.preload_progress);
    let loaded_endgame = crate::game::endgame::EndgameTravelCfg::default_config();
    bump(&handles.preload_progress);
    let loaded_weather = crate::game::weather::WeatherConfig::load_from_static();
    bump(&handles.preload_progress);
    let loaded_camp = crate::game::CampConfig::load_from_static();
    bump(&handles.preload_progress);
    let loaded_crossings = serde_json::from_str::<crate::game::CrossingConfig>(include_str!(
        "../../static/assets/data/crossings.json"
    ))
    .unwrap_or_default();
    bump(&handles.preload_progress);
    let loaded_result = load_result_config().unwrap_or_default();
    bump(&handles.preload_progress);
    let _ = serde_json::from_str::<crate::game::store::Store>(include_str!(
        "../../static/assets/data/store.json"
    ));
    bump(&handles.preload_progress);
    let _ = crate::game::personas::PersonasList::from_json(include_str!(
        "../../static/assets/data/personas.json"
    ));
    bump(&handles.preload_progress);
    let _ = serde_json::from_str::<crate::game::vehicle::VehicleConfig>(include_str!(
        "../../static/assets/data/vehicle.json"
    ));
    bump(&handles.preload_progress);
    let _ = serde_json::from_str::<crate::game::boss::BossConfig>(include_str!(
        "../../static/assets/data/boss.json"
    ));
    bump(&handles.preload_progress);
    handles.data.set(loaded_data);
    handles.pacing_config.set(loaded_pacing);
    handles.endgame_config.set(loaded_endgame);
    handles.weather_config.set(loaded_weather);
    handles.camp_config.set(loaded_camp);
    handles.crossing_config.set(loaded_crossings);
    handles.result_config.set(loaded_result);
    handles.preload_progress.set(100);
    handles.boot_ready.set(true);
}

#[cfg(target_arch = "wasm32")]
#[hook]
pub fn use_bootstrap(app_state: &AppState) {
    let handles = handles_from_state(app_state);

    use_effect_with((), move |()| {
        wasm_bindgen_futures::spawn_local(async move {
            bootstrap_load(&handles);
        });
        || {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[function_component(BootstrapHarness)]
    fn bootstrap_harness() -> Html {
        let app_state = crate::app::state::use_app_state();
        let handles = handles_from_state(&app_state);
        let initialized = use_state(|| false);
        if !*initialized {
            initialized.set(true);
            bootstrap_load(&handles);
        }
        Html::default()
    }

    #[test]
    fn bootstrap_loads_assets_for_tests() {
        let _ = block_on(LocalServerRenderer::<BootstrapHarness>::new().render());
    }
}
