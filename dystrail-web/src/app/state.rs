use crate::app::phase::Phase;
use crate::game::CampConfig;
use crate::game::boss::BossConfig;
use crate::game::data::EncounterData;
use crate::game::endgame::EndgameTravelCfg;
use crate::game::pacing::PacingConfig;
use crate::game::state::GameState;
use crate::game::weather::WeatherConfig;
use crate::game::{JourneySession, ResultConfig};
use yew::prelude::*;

#[derive(Clone)]
pub struct AppState {
    pub show_abandon: UseStateHandle<bool>,
    pub travel_running: UseStateHandle<bool>,
    pub map_automatic: UseStateHandle<bool>,
    pub map_return: UseStateHandle<Option<Phase>>,
    pub journey_detail: UseStateHandle<crate::components::ui::travel_panel::tabs::Selection>,
    pub travel_speed: UseStateHandle<super::flow::TravelSpeed>,
    pub recovery_ready: UseStateHandle<bool>,
    pub town_open: UseStateHandle<bool>,
    pub pending_turn: UseStateHandle<Option<std::rc::Rc<crate::app::turn::PendingTurn>>>,
    pub aftermath: UseStateHandle<Option<crate::app::aftermath::Aftermath>>,
    pub action_lock: std::rc::Rc<std::cell::RefCell<bool>>,
    pub phase: UseStateHandle<Phase>,
    pub code: UseStateHandle<AttrValue>,
    pub data: UseStateHandle<EncounterData>,
    pub pacing_config: UseStateHandle<PacingConfig>,
    pub endgame_config: UseStateHandle<EndgameTravelCfg>,
    pub weather_config: UseStateHandle<WeatherConfig>,
    pub camp_config: UseStateHandle<CampConfig>,
    pub boss_config: UseStateHandle<BossConfig>,
    pub result_config: UseStateHandle<ResultConfig>,
    pub preload_progress: UseStateHandle<u8>,
    pub boot_ready: UseStateHandle<bool>,
    pub high_contrast: UseStateHandle<bool>,
    pub help_enabled: UseStateHandle<bool>,
    pub pending_state: UseStateHandle<Option<GameState>>,
    pub session: UseStateHandle<Option<JourneySession>>,
    pub logs: UseStateHandle<Vec<String>>,
    pub run_seed: UseStateHandle<u64>,
    pub save_status: UseStateHandle<String>,
    pub show_save: UseStateHandle<bool>,
    pub save_focus_target: UseStateHandle<AttrValue>,
    pub show_settings: UseStateHandle<bool>,
    pub current_language: UseStateHandle<String>,
}

#[hook]
pub fn use_app_state() -> AppState {
    AppState {
        show_abandon: use_state(|| false),
        travel_running: use_state(|| false),
        map_automatic: use_state(|| false),
        map_return: use_state(|| None),
        journey_detail: use_state(crate::components::ui::travel_panel::tabs::Selection::default),
        travel_speed: use_state(super::flow::TravelSpeed::default),
        recovery_ready: use_state(|| false),
        town_open: use_state(|| false),
        pending_turn: use_state(|| None),
        aftermath: use_state(|| None),
        action_lock: use_mut_ref(|| false),
        phase: use_state(|| Phase::Boot),
        code: use_state(|| AttrValue::from("CL-ORANGE42")),
        data: use_state(EncounterData::empty),
        pacing_config: use_state(PacingConfig::default_config),
        endgame_config: use_state(EndgameTravelCfg::default_config),
        weather_config: use_state(WeatherConfig::default_config),
        camp_config: use_state(CampConfig::default_config),
        boss_config: use_state(BossConfig::load_from_static),
        result_config: use_state(ResultConfig::default),
        preload_progress: use_state(|| 0_u8),
        boot_ready: use_state(|| false),
        high_contrast: use_state(crate::a11y::high_contrast_enabled),
        help_enabled: use_state(super::help::enabled),
        pending_state: use_state(|| None::<GameState>),
        session: use_state(|| None::<JourneySession>),
        logs: use_state(Vec::<String>::new),
        run_seed: use_state(|| 0_u64),
        save_status: use_state(String::new),
        show_save: use_state(|| false),
        save_focus_target: use_state(|| AttrValue::from("game-menu-button")),
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
