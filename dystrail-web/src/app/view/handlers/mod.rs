mod boss;
mod crossing;
mod outcome;
mod prefs;
mod route_prompt;
mod storage;
mod store;
mod travel;

use crate::app::state::AppState;
use crate::game::state::{DietId, PaceId};
use yew::prelude::*;
use yew_router::prelude::Navigator;

pub use boss::build_boss;
pub use crossing::{build_crossing_choice, build_otdeluxe_crossing_choice};
pub use prefs::{
    build_begin_boot, build_go_home, build_lang_change, build_settings_hc_change, build_toggle_hc,
};
pub use route_prompt::build_route_prompt_choice;
pub use storage::{build_export_state, build_import_state, build_load, build_save};
pub use store::{build_store_leave, build_store_purchase};
pub use travel::{
    build_diet_change, build_encounter_choice, build_hunt, build_pace_change, build_trade,
    build_travel,
};

#[derive(Clone)]
pub struct AppHandlers {
    pub travel: Callback<()>,
    pub trade: Callback<()>,
    pub hunt: Callback<()>,
    pub store_purchase: Callback<Vec<crate::game::OtDeluxeStoreLineItem>>,
    pub store_leave: Callback<()>,
    pub pace_change: Callback<PaceId>,
    pub diet_change: Callback<DietId>,
    pub encounter_choice: Callback<usize>,
    pub crossing_choice: Callback<u8>,
    pub otdeluxe_crossing_choice: Callback<u8>,
    pub route_prompt_choice: Callback<crate::game::OtDeluxeRouteDecision>,
    pub boss: Callback<()>,
    pub save: Callback<()>,
    pub load: Callback<()>,
    pub export_state: Callback<()>,
    pub import_state: Callback<String>,
    pub lang_change: Callback<String>,
    pub toggle_hc: Callback<bool>,
    pub settings_hc_change: Callback<bool>,
    pub go_home: Callback<()>,
    pub begin_boot: Callback<()>,
}

impl AppHandlers {
    #[must_use]
    pub fn new(state: &AppState, navigator: Option<Navigator>) -> Self {
        Self {
            travel: build_travel(state),
            trade: build_trade(state),
            hunt: build_hunt(state),
            store_purchase: build_store_purchase(state),
            store_leave: build_store_leave(state),
            pace_change: build_pace_change(state),
            diet_change: build_diet_change(state),
            encounter_choice: build_encounter_choice(state),
            crossing_choice: build_crossing_choice(state),
            otdeluxe_crossing_choice: build_otdeluxe_crossing_choice(state),
            route_prompt_choice: build_route_prompt_choice(state),
            boss: build_boss(state),
            save: build_save(state),
            load: build_load(state),
            export_state: build_export_state(state),
            import_state: build_import_state(state),
            lang_change: build_lang_change(state),
            toggle_hc: build_toggle_hc(state),
            settings_hc_change: build_settings_hc_change(state),
            go_home: build_go_home(state, navigator),
            begin_boot: build_begin_boot(state),
        }
    }
}
