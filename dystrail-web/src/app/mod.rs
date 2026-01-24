#[cfg(target_arch = "wasm32")]
use crate::router::Route;
#[cfg(target_arch = "wasm32")]
use yew::prelude::*;
#[cfg(target_arch = "wasm32")]
use yew_router::prelude::*;

pub mod bootstrap;
pub mod phase;
pub mod routing;
pub mod state;
pub mod view;

pub use phase::Phase;

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
pub fn app() -> Html {
    let router_base = crate::paths::router_base().map(AttrValue::from);
    html! {
        <BrowserRouter basename={router_base}>
            <AppInner />
        </BrowserRouter>
    }
}

#[cfg(target_arch = "wasm32")]
#[function_component(AppInner)]
pub fn app_inner() -> Html {
    let app_state = state::use_app_state();
    bootstrap::use_bootstrap(&app_state);

    let navigator = use_navigator();
    let route = use_route::<Route>();

    routing::use_sync_route_with_phase(&app_state.phase, navigator.clone(), route.clone());
    routing::use_sync_phase_with_route(&app_state.phase, route.clone());

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
            Phase::Outfitting,
            Phase::Menu,
            Phase::Travel,
            Phase::Store,
            Phase::Crossing,
            Phase::RoutePrompt,
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
