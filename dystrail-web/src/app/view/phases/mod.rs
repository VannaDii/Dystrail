mod boss;
mod camp;
mod crossing;
mod encounter;
mod menu;
mod outfitting;
mod persona;
mod result;
mod route_prompt;
mod seed_footer;
mod store;
mod travel;

use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::not_found::NotFound;
use crate::router::Route;
use yew::prelude::*;

pub use boss::render_boss;
pub use camp::render_camp;
pub use crossing::render_crossing;
pub use encounter::render_encounter;
pub use menu::render_menu;
pub use outfitting::render_outfitting;
pub use persona::render_persona;
pub use result::render_result;
pub use route_prompt::render_route_prompt;
pub use seed_footer::render_seed_footer;
pub use store::render_store;
pub use travel::render_travel;

pub fn render_main_view(state: &AppState, handlers: &AppHandlers, route: Option<&Route>) -> Html {
    let not_found = matches!(route, None | Some(Route::NotFound));
    if not_found {
        return html! { <NotFound on_go_home={handlers.go_home.clone()} /> };
    }

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
        Phase::Persona => render_persona(state),
        Phase::Outfitting => render_outfitting(state),
        Phase::Menu => render_menu(state),
        Phase::Travel => render_travel(state, handlers),
        Phase::Store => render_store(state, handlers),
        Phase::Crossing => render_crossing(state, handlers),
        Phase::RoutePrompt => render_route_prompt(state, handlers),
        Phase::Camp => render_camp(state),
        Phase::Encounter => render_encounter(state, handlers),
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
            route: Some(Route::Home),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("boot"));

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
            phase: Phase::Outfitting,
            route: Some(Route::Outfitting),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("outfitting-store"));

        let html = render_phase(PhaseHarnessProps {
            phase: Phase::Menu,
            route: Some(Route::Home),
            session: None,
            pending_state: None,
            data: EncounterData::empty(),
        });
        assert!(html.contains("menu"));
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
}
