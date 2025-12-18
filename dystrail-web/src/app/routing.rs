use crate::app::phase::Phase;
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

#[hook]
pub fn use_sync_route_with_phase(
    phase: &UseStateHandle<Phase>,
    navigator: Option<Navigator>,
    active_route: Option<Route>,
) {
    let phase = phase.clone();
    use_effect_with((phase, active_route), move |(phase, current_route)| {
        if let Some(nav) = navigator.as_ref() {
            let new_route = Route::from_phase(phase);
            if Some(&new_route) != current_route.as_ref() {
                nav.push(&new_route);
            }
        }
    });
}

#[hook]
pub fn use_sync_phase_with_route(phase: &UseStateHandle<Phase>, route: Option<Route>) {
    let phase = phase.clone();
    use_effect_with(route, move |route| {
        // Don't change phase during Boot - only route changes should trigger phase changes
        if *phase == Phase::Boot {
            return;
        }

        if let Some(route) = route
            && let Some(new_phase) = route.to_phase()
            && new_phase != *phase
        {
            phase.set(new_phase);
        }
    });
}
