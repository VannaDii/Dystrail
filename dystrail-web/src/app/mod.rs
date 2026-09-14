use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::*;

pub mod abandon;
pub mod activities;
pub mod aftermath;
pub mod ally_loss;
pub mod bootstrap;
pub mod crew_care;
pub mod flow;
pub mod help;
pub mod history;
pub mod journey_panel;
pub mod map;
mod map_countdown;
pub mod phase;
pub mod receipt;
pub mod recovery;
pub mod repair;
pub mod routing;
pub mod services;
pub mod state;
pub mod town;
pub mod town_facts;
mod town_profile;
pub mod trading;
pub mod turn;
pub mod view;

pub use phase::Phase;

#[function_component(App)]
pub fn app() -> Html {
    let router_base = crate::paths::router_base().map(AttrValue::from);
    html! {
        <BrowserRouter basename={router_base}>
            <AppInner />
        </BrowserRouter>
    }
}

#[function_component(AppInner)]
pub fn app_inner() -> Html {
    let app_state = state::use_app_state();
    bootstrap::use_bootstrap(&app_state);
    recovery::use_recovery(&app_state);
    flow::use_travel_flow(&app_state);
    crate::components::ui::dismiss::use_number_shortcuts();

    let navigator = use_navigator();
    let route = use_route::<Route>();

    routing::use_sync_route_with_phase(&app_state.phase, navigator.clone(), route.clone());

    {
        let lock = app_state.action_lock.clone();
        use_effect_with(
            (
                *app_state.phase,
                app_state.aftermath.is_some(),
                app_state.pending_turn.is_some(),
            ),
            move |(_, feedback, transit)| {
                if !feedback && !transit {
                    *lock.borrow_mut() = false;
                }
            },
        );
    }
    view::render_app(&app_state, route.as_ref(), navigator)
}

#[cfg(test)]
mod tests {
    use super::Phase;
    use super::phase::is_seed_code_valid;

    #[test]
    fn seed_code_validation_handles_expected_formats() {
        assert!(is_seed_code_valid("CL-ORANGE42"));
        assert!(is_seed_code_valid("DP-SIGNAL99"));
        assert!(!is_seed_code_valid("CL-ORANGE4"));
        assert!(!is_seed_code_valid("INVALID"));
        assert!(!is_seed_code_valid("XY-TOOLATE00"));
    }

    #[test]
    fn route_phase_mappings_cover_all_states() {
        use crate::router::Route;

        let phases = [
            Phase::Boot,
            Phase::Persona,
            Phase::Crew,
            Phase::Outfitting,
            Phase::Menu,
            Phase::Travel,
            Phase::Town,
            Phase::CrewCare,
            Phase::AllyLoss,
            Phase::Map,
            Phase::Camp,
            Phase::Encounter,
            Phase::Boss,
            Phase::Result,
        ];

        for phase in phases {
            let route = Route::from_phase(&phase);
            let round_trip = route.to_phase();
            match (phase, round_trip) {
                (Phase::Boot | Phase::Menu, None) => {}
                (_, Some(mapped)) => assert!(mapped == phase),
                (_, None) => panic!("Route should map to a phase"),
            }
        }
    }
}

pub mod weather_status;
pub mod workshop;
pub mod workshop_events;
#[cfg(test)]
mod workshop_tests;
