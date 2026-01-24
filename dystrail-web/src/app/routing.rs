#[cfg(any(target_arch = "wasm32", test))]
use crate::app::phase::Phase;
#[cfg(any(target_arch = "wasm32", test))]
use crate::router::Route;
#[cfg(target_arch = "wasm32")]
use yew::prelude::*;
#[cfg(target_arch = "wasm32")]
use yew_router::prelude::Navigator;

#[cfg(any(target_arch = "wasm32", test))]
fn next_route_for_phase(phase: Phase, current_route: Option<&Route>) -> Option<Route> {
    let new_route = Route::from_phase(&phase);
    if Some(&new_route) == current_route {
        None
    } else {
        Some(new_route)
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn next_phase_for_route(current_phase: Phase, route: Option<Route>) -> Option<Phase> {
    if current_phase == Phase::Boot {
        return None;
    }

    route
        .and_then(|route| route.to_phase())
        .filter(|new_phase| *new_phase != current_phase)
}

#[cfg(target_arch = "wasm32")]
#[hook]
pub fn use_sync_route_with_phase(
    phase: &UseStateHandle<Phase>,
    navigator: Option<Navigator>,
    active_route: Option<Route>,
) {
    let phase = phase.clone();
    use_effect_with((phase, active_route), move |(phase, current_route)| {
        if let (Some(nav), Some(new_route)) = (
            navigator.as_ref(),
            next_route_for_phase(**phase, current_route.as_ref()),
        ) {
            nav.push(&new_route);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[hook]
pub fn use_sync_phase_with_route(phase: &UseStateHandle<Phase>, route: Option<Route>) {
    let phase = phase.clone();
    use_effect_with(route, move |route| {
        if let Some(new_phase) = next_phase_for_route(*phase, route.clone()) {
            phase.set(new_phase);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_route_for_phase_skips_when_unchanged() {
        let route = Route::from_phase(&Phase::Travel);
        assert!(next_route_for_phase(Phase::Travel, Some(&route)).is_none());
        assert_eq!(
            next_route_for_phase(Phase::Travel, None),
            Some(Route::Travel)
        );
    }

    #[test]
    fn next_phase_for_route_respects_boot_and_diffs() {
        assert!(next_phase_for_route(Phase::Boot, Some(Route::Travel)).is_none());
        assert!(next_phase_for_route(Phase::Menu, Some(Route::Home)).is_none());
        assert_eq!(
            next_phase_for_route(Phase::Menu, Some(Route::Travel)),
            Some(Phase::Travel)
        );
    }
}
