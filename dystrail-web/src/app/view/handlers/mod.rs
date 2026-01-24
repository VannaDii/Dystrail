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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::phase::Phase;
    use crate::game::data::{Choice, Effects, Encounter, EncounterData};
    use crate::game::otdeluxe_state::OtDeluxeRiverState;
    use crate::game::{
        CrossingKind, EndgameTravelCfg, GameMode, JourneySession, MechanicalPolicyId,
        OtDeluxeRiver, OtDeluxeRiverBed, OtDeluxeRouteDecision, PendingCrossing, StrategyId,
    };
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    fn encounter_stub() -> Encounter {
        Encounter {
            id: String::from("enc"),
            name: String::from("Encounter"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: String::from("Continue"),
                effects: Effects::default(),
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    fn base_data() -> EncounterData {
        EncounterData::load_from_static()
    }

    fn base_state(data: &EncounterData) -> crate::game::GameState {
        crate::game::GameState::default().with_seed(42, GameMode::Classic, data.clone())
    }

    fn build_session(state: crate::game::GameState) -> JourneySession {
        JourneySession::from_state(
            state,
            StrategyId::Balanced,
            &EndgameTravelCfg::default_config(),
        )
    }

    #[hook]
    fn use_app_state(
        session: Option<JourneySession>,
        pending: Option<crate::game::GameState>,
        data: EncounterData,
        boot_ready: bool,
    ) -> AppState {
        AppState {
            phase: use_state(|| Phase::Menu),
            code: use_state(|| AttrValue::from("CL-ORANGE42")),
            data: use_state(move || data),
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 100),
            boot_ready: use_state(move || boot_ready),
            high_contrast: use_state(|| false),
            pending_state: use_state(move || pending),
            session: use_state(move || session),
            logs: use_state(Vec::<String>::new),
            run_seed: use_state(|| 4242_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        }
    }

    #[function_component(DystrailHandlersHarness)]
    fn dystrail_handlers_harness() -> Html {
        crate::i18n::set_lang("en");
        let invoked = use_state(|| false);
        let data = base_data();
        let mut state = base_state(&data);
        state.current_encounter = Some(encounter_stub());
        state.pending_crossing = Some(PendingCrossing {
            kind: CrossingKind::Checkpoint,
            computed_miles_today: 0.0,
        });
        state.inventory.tags.insert(String::from("permit"));
        state.budget_cents = 50_000;
        let session = build_session(state);
        let app_state = use_app_state(Some(session), None, data, true);
        let handlers = AppHandlers::new(&app_state, None);

        if !*invoked {
            invoked.set(true);
            handlers.begin_boot.emit(());
            handlers.go_home.emit(());
            handlers.lang_change.emit(String::from("es"));
            handlers.toggle_hc.emit(true);
            handlers.settings_hc_change.emit(false);
            handlers
                .pace_change
                .emit(crate::game::state::PaceId::Steady);
            handlers.diet_change.emit(crate::game::state::DietId::Mixed);
            handlers.encounter_choice.emit(0);
            handlers.crossing_choice.emit(1);
            handlers.crossing_choice.emit(2);
            handlers.crossing_choice.emit(3);
            handlers.crossing_choice.emit(0);
            handlers.boss.emit(());
            handlers.save.emit(());
            handlers.load.emit(());
            handlers.export_state.emit(());
            handlers
                .import_state
                .emit(serde_json::to_string(&crate::game::GameState::default()).unwrap());
            handlers.import_state.emit(String::from("invalid"));
            handlers.travel.emit(());
        }
        Html::default()
    }

    #[function_component(OtDeluxeHandlersHarness)]
    fn otdeluxe_handlers_harness() -> Html {
        crate::i18n::set_lang("en");
        let invoked = use_state(|| false);
        let data = base_data();
        let mut state = base_state(&data);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.store.pending_node = Some(3);
        state.ot_deluxe.crossing.choice_pending = true;
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Snake);
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 120.0,
            depth_ft: 4.0,
            swiftness: 0.5,
            bed: OtDeluxeRiverBed::Muddy,
        });
        state.ot_deluxe.inventory.cash_cents = 1_000;
        state.ot_deluxe.inventory.clothes_sets = 6;
        state.ot_deluxe.route.pending_prompt =
            Some(crate::game::OtDeluxeRoutePrompt::SubletteCutoff);
        let session = build_session(state);
        let app_state = use_app_state(Some(session), None, data, true);
        let handlers = AppHandlers::new(&app_state, None);

        if !*invoked {
            invoked.set(true);
            handlers.trade.emit(());
            handlers.hunt.emit(());
            handlers.store_purchase.emit(Vec::new());
            handlers.store_leave.emit(());
            handlers.otdeluxe_crossing_choice.emit(1);
            handlers.otdeluxe_crossing_choice.emit(2);
            handlers.otdeluxe_crossing_choice.emit(3);
            handlers.otdeluxe_crossing_choice.emit(4);
            handlers.otdeluxe_crossing_choice.emit(0);
            handlers
                .route_prompt_choice
                .emit(OtDeluxeRouteDecision::StayOnTrail);
        }
        Html::default()
    }

    #[function_component(EmptyHandlersHarness)]
    fn empty_handlers_harness() -> Html {
        let invoked = use_state(|| false);
        let app_state = use_app_state(None, None, EncounterData::empty(), false);
        let handlers = AppHandlers::new(&app_state, None);
        if !*invoked {
            invoked.set(true);
            handlers.travel.emit(());
            handlers.trade.emit(());
            handlers.hunt.emit(());
            handlers.crossing_choice.emit(9);
            handlers.otdeluxe_crossing_choice.emit(9);
            handlers.store_purchase.emit(Vec::new());
            handlers.store_leave.emit(());
        }
        Html::default()
    }

    #[test]
    fn handlers_cover_dystrail_paths() {
        let _ = block_on(LocalServerRenderer::<DystrailHandlersHarness>::new().render());
    }

    #[test]
    fn handlers_cover_otdeluxe_paths() {
        let _ = block_on(LocalServerRenderer::<OtDeluxeHandlersHarness>::new().render());
    }

    #[test]
    fn handlers_cover_empty_paths() {
        let _ = block_on(LocalServerRenderer::<EmptyHandlersHarness>::new().render());
    }
}
