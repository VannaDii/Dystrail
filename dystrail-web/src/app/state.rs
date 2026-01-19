use crate::app::phase::Phase;
use crate::game::boss::BossConfig;
use crate::game::data::EncounterData;
use crate::game::endgame::EndgameTravelCfg;
use crate::game::pacing::PacingConfig;
use crate::game::state::GameState;
use crate::game::weather::WeatherConfig;
use crate::game::{CampConfig, CrossingConfig};
use crate::game::{JourneySession, ResultConfig};
use yew::prelude::*;

#[derive(Clone)]
pub struct AppState {
    pub phase: UseStateHandle<Phase>,
    pub code: UseStateHandle<AttrValue>,
    pub data: UseStateHandle<EncounterData>,
    pub pacing_config: UseStateHandle<PacingConfig>,
    pub endgame_config: UseStateHandle<EndgameTravelCfg>,
    pub weather_config: UseStateHandle<WeatherConfig>,
    pub camp_config: UseStateHandle<CampConfig>,
    pub crossing_config: UseStateHandle<CrossingConfig>,
    pub boss_config: UseStateHandle<BossConfig>,
    pub result_config: UseStateHandle<ResultConfig>,
    pub preload_progress: UseStateHandle<u8>,
    pub boot_ready: UseStateHandle<bool>,
    pub high_contrast: UseStateHandle<bool>,
    pub pending_state: UseStateHandle<Option<GameState>>,
    pub session: UseStateHandle<Option<JourneySession>>,
    pub logs: UseStateHandle<Vec<String>>,
    pub run_seed: UseStateHandle<u64>,
    pub show_save: UseStateHandle<bool>,
    pub save_focus_target: UseStateHandle<AttrValue>,
    pub show_settings: UseStateHandle<bool>,
    pub current_language: UseStateHandle<String>,
}

#[hook]
pub fn use_app_state() -> AppState {
    AppState {
        phase: use_state(|| Phase::Boot),
        code: use_state(|| AttrValue::from("CL-ORANGE42")),
        data: use_state(EncounterData::empty),
        pacing_config: use_state(PacingConfig::default_config),
        endgame_config: use_state(EndgameTravelCfg::default_config),
        weather_config: use_state(WeatherConfig::default_config),
        camp_config: use_state(CampConfig::default_config),
        crossing_config: use_state(CrossingConfig::default),
        boss_config: use_state(BossConfig::load_from_static),
        result_config: use_state(ResultConfig::default),
        preload_progress: use_state(|| 0_u8),
        boot_ready: use_state(|| false),
        high_contrast: use_state(crate::a11y::high_contrast_enabled),
        pending_state: use_state(|| None::<GameState>),
        session: use_state(|| None::<JourneySession>),
        logs: use_state(Vec::<String>::new),
        run_seed: use_state(|| 0_u64),
        show_save: use_state(|| false),
        save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
        show_settings: use_state(|| false),
        current_language: use_state(crate::i18n::current_lang),
    }
}

impl AppState {
    #[must_use]
    pub fn data_ready(&self) -> bool {
        !self.data.encounters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[function_component(DataReadyEmptyHarness)]
    fn data_ready_empty_harness() -> Html {
        let state = use_app_state();
        let label = if state.data_ready() {
            "ready"
        } else {
            "not-ready"
        };
        html! { <span>{ label }</span> }
    }

    #[function_component(DataReadyLoadedHarness)]
    fn data_ready_loaded_harness() -> Html {
        let data = use_state(crate::game::data::EncounterData::load_from_static);
        let ready = !data.encounters.is_empty();
        html! { <span>{ if ready { "ready" } else { "not-ready" } }</span> }
    }

    #[test]
    fn data_ready_tracks_encounter_data() {
        let html = block_on(LocalServerRenderer::<DataReadyEmptyHarness>::new().render());
        assert!(html.contains("not-ready"));
        let html = block_on(LocalServerRenderer::<DataReadyLoadedHarness>::new().render());
        assert!(html.contains("ready"));
    }
}
