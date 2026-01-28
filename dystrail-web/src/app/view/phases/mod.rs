mod about;
mod boss;
mod camp;
mod crossing;
mod encounter;
mod inventory;
mod map;
mod menu;
mod mode_select;
mod outfitting;
mod pace_diet;
mod persona;
mod result;
mod route_prompt;
mod seed_footer;
mod settings;
mod store;
mod travel;

use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::not_found::NotFound;
use crate::router::Route;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub use about::render_about;
pub use boss::render_boss;
pub use camp::render_camp;
pub use crossing::render_crossing;
pub use encounter::render_encounter;
pub use inventory::render_inventory;
pub use map::render_map;
pub use menu::render_menu;
pub use mode_select::render_mode_select;
pub use outfitting::render_outfitting;
pub use pace_diet::render_pace_diet;
pub use persona::render_persona;
pub use result::render_result;
pub use route_prompt::render_route_prompt;
pub use seed_footer::render_seed_footer;
pub use settings::render_settings;
pub use store::render_store;
pub use travel::render_travel;

#[cfg(any(test, target_arch = "wasm32"))]
fn apply_escape_to_travel(key: &str, set_phase: &Callback<Phase>, prevent_default: impl FnOnce()) {
    if key == "Escape" {
        set_phase.emit(Phase::Travel);
        prevent_default();
    }
}

pub fn render_main_view(state: &AppState, handlers: &AppHandlers, route: Option<&Route>) -> Html {
    let not_found = matches!(route, None | Some(Route::NotFound));
    if not_found {
        return html! { <NotFound on_go_home={handlers.go_home.clone()} /> };
    }

    let escape_to_travel = {
        let phase_handle = state.phase.clone();
        let set_phase = Callback::from(move |phase: Phase| phase_handle.set(phase));
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: KeyboardEvent| {
                apply_escape_to_travel(&e.key(), &set_phase, || e.prevent_default());
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = set_phase;
            Callback::from(|_e: KeyboardEvent| {})
        }
    };

    match *state.phase {
        Phase::Boot => {
            let boot_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
            html! {
                <crate::pages::boot::BootPage
                    logo_src={boot_logo_src}
                    ready={*state.boot_ready}
                    preload_progress={*state.preload_progress}
                    on_begin={handlers.begin_boot.clone()}
                />
            }
        }
        Phase::Menu => render_menu(state),
        Phase::About => render_about(state),
        Phase::Settings => render_settings(state, handlers),
        Phase::Persona => render_persona(state),
        Phase::ModeSelect => render_mode_select(state),
        Phase::Outfitting => render_outfitting(state),
        Phase::Travel => render_travel(state, handlers),
        Phase::Inventory => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_inventory(state) }</div> }
        }
        Phase::PaceDiet => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_pace_diet(state, handlers) }</div> }
        }
        Phase::Map => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_map(state) }</div> }
        }
        Phase::Store => render_store(state, handlers),
        Phase::Crossing => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_crossing(state, handlers) }</div> }
        }
        Phase::RoutePrompt => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_route_prompt(state, handlers) }</div> }
        }
        Phase::Camp => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_camp(state) }</div> }
        }
        Phase::Encounter => {
            html! { <div onkeydown={escape_to_travel.clone()}>{ render_encounter(state, handlers) }</div> }
        }
        Phase::Boss => render_boss(state, handlers),
        Phase::Result => render_result(state),
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
        OtDeluxeRiver, OtDeluxeRiverBed, PendingCrossing, StrategyId,
    };
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[derive(Properties, Clone)]
    struct PhaseHarnessProps {
        phase: Phase,
        route: Option<Route>,
        session: Option<JourneySession>,
        pending_state: Option<crate::game::GameState>,
        data: EncounterData,
    }

    impl PartialEq for PhaseHarnessProps {
        fn eq(&self, other: &Self) -> bool {
            self.phase == other.phase && self.route == other.route
        }
    }

    #[function_component(PhaseHarness)]
    fn phase_harness(props: &PhaseHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let data_handle = {
            let data = props.data.clone();
            use_state(move || data)
        };
        let app_state = AppState {
            phase: use_state(|| props.phase),
            code: use_state(|| AttrValue::from("CL-ORANGE42")),
            data: data_handle,
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 42_u8),
            boot_ready: use_state(|| true),
            high_contrast: use_state(|| false),
            pending_state: use_state(|| props.pending_state.clone()),
            session: use_state(|| props.session.clone()),
            logs: use_state(|| vec![String::from("log.booting")]),
            run_seed: use_state(|| 4242_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        };
        let handlers = AppHandlers::new(&app_state, None);
        render_main_view(&app_state, &handlers, props.route.as_ref())
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

    fn render_phase(props: PhaseHarnessProps) -> String {
        block_on(LocalServerRenderer::<PhaseHarness>::with_props(props).render())
    }

    #[test]
    fn render_main_view_handles_boot_and_not_found() {
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Boot,
            route: Some(Route::Boot),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("boot-screen"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Menu,
            route: Some(Route::NotFound),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("not-found"));
    }

    #[test]
    fn render_main_view_handles_persona_outfitting_menu() {
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Persona,
            route: Some(Route::Persona),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("persona-select"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::ModeSelect,
            route: Some(Route::ModeSelect),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("mode-select"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Outfitting,
            route: Some(Route::Outfitting),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("outfitting-store"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Menu,
            route: Some(Route::Menu),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("menu-screen"));
    }

    #[test]
    fn render_main_view_handles_travel_and_camp() {
        let data = base_data();
        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Travel,
            route: Some(Route::Travel),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("travel"));

        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Inventory,
            route: Some(Route::Inventory),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("inventory-screen"));

        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::PaceDiet,
            route: Some(Route::PaceDiet),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("pace-diet-screen"));

        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Map,
            route: Some(Route::Map),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("map-screen"));

        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Camp,
            route: Some(Route::Camp),
            session: Some(session),
            pending_state: None,
            data,
        });
        assert!(html.contains("camp"));
    }

    #[test]
    fn render_main_view_handles_encounter_and_boss() {
        let data = base_data();
        let mut state = base_state(&data);
        state.current_encounter = Some(encounter_stub());
        let session = build_session(state);
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Encounter,
            route: Some(Route::Encounter),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("encounter"));

        let mut state = base_state(&data);
        state.boss.readiness.ready = true;
        let session = build_session(state);
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Boss,
            route: Some(Route::Boss),
            session: Some(session),
            pending_state: None,
            data,
        });
        assert!(html.contains("boss"));
    }

    #[test]
    fn render_main_view_handles_crossing_and_store() {
        let data = base_data();
        let mut state = base_state(&data);
        state.pending_crossing = Some(PendingCrossing {
            kind: CrossingKind::Checkpoint,
            computed_miles_today: 0.0,
        });
        let session = build_session(state);
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Crossing,
            route: Some(Route::Crossing),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("crossing"));

        let mut state = base_state(&data);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.store.pending_node = Some(3);
        let session = build_session(state);
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Store,
            route: Some(Route::Store),
            session: Some(session),
            pending_state: None,
            data,
        });
        assert!(html.contains("store"));
    }

    #[test]
    fn render_main_view_handles_route_prompt_and_result() {
        let data = base_data();
        let mut state = base_state(&data);
        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.route.pending_prompt =
            Some(crate::game::OtDeluxeRoutePrompt::SubletteCutoff);
        state.ot_deluxe.crossing.river_kind = Some(OtDeluxeRiver::Kansas);
        state.ot_deluxe.crossing.river = Some(OtDeluxeRiverState {
            width_ft: 120.0,
            depth_ft: 4.0,
            swiftness: 0.5,
            bed: OtDeluxeRiverBed::Muddy,
        });
        let session = build_session(state);
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::RoutePrompt,
            route: Some(Route::RoutePrompt),
            session: Some(session),
            pending_state: None,
            data: data.clone(),
        });
        assert!(html.contains("route"));

        let session = build_session(base_state(&data));
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Result,
            route: Some(Route::Result),
            session: Some(session),
            pending_state: None,
            data,
        });
        assert!(html.contains("result"));
    }

    #[test]
    fn render_main_view_handles_meta_screens() {
        let html = render_phase(PhaseHarnessProps {
            phase: Phase::About,
            route: Some(Route::About),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("about-screen"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Settings,
            route: Some(Route::Settings),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("settings-screen"));
    }

    #[test]
    fn apply_escape_to_travel_updates_phase() {
        let phase_seen = std::rc::Rc::new(std::cell::Cell::new(None));
        let phase_seen_ref = phase_seen.clone();
        let set_phase = Callback::from(move |phase: Phase| phase_seen_ref.set(Some(phase)));
        let prevented = std::rc::Rc::new(std::cell::Cell::new(false));
        let prevented_ref = prevented.clone();
        apply_escape_to_travel("Escape", &set_phase, || prevented_ref.set(true));
        assert_eq!(phase_seen.get(), Some(Phase::Travel));
        assert!(prevented.get());
    }
}
