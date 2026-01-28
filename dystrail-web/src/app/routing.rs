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
    let new_phase = route.and_then(|route| route.to_phase())?;
    if new_phase == current_phase {
        return None;
    }

    is_route_transition_allowed(current_phase, new_phase).then_some(new_phase)
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhaseDomain {
    Meta,
    Setup,
    Journey,
    Terminal,
}

#[cfg(any(target_arch = "wasm32", test))]
const fn phase_domain(phase: Phase) -> PhaseDomain {
    match phase {
        Phase::Boot | Phase::Menu | Phase::About | Phase::Settings => PhaseDomain::Meta,
        Phase::Persona | Phase::ModeSelect | Phase::Outfitting => PhaseDomain::Setup,
        Phase::Travel
        | Phase::Inventory
        | Phase::PaceDiet
        | Phase::Map
        | Phase::Store
        | Phase::Crossing
        | Phase::RoutePrompt
        | Phase::Camp
        | Phase::Encounter
        | Phase::Boss => PhaseDomain::Journey,
        Phase::Result => PhaseDomain::Terminal,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const fn is_route_transition_allowed(current: Phase, next: Phase) -> bool {
    match phase_domain(current) {
        PhaseDomain::Meta => matches!(
            next,
            Phase::Boot | Phase::Menu | Phase::About | Phase::Settings | Phase::Persona
        ),
        PhaseDomain::Setup => {
            matches!(next, Phase::Persona | Phase::ModeSelect | Phase::Outfitting)
        }
        PhaseDomain::Journey => {
            matches!(
                phase_domain(next),
                PhaseDomain::Journey | PhaseDomain::Terminal
            )
        }
        PhaseDomain::Terminal => matches!(next, Phase::Menu),
    }
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
        assert!(next_phase_for_route(Phase::Menu, Some(Route::Menu)).is_none());
        assert!(next_phase_for_route(Phase::Menu, Some(Route::Travel)).is_none());
        assert_eq!(
            next_phase_for_route(Phase::Menu, Some(Route::Persona)),
            Some(Phase::Persona)
        );
        assert!(next_phase_for_route(Phase::Travel, Some(Route::Menu)).is_none());
    }

    #[test]
    fn phase_domain_covers_setup_and_terminal() {
        assert_eq!(phase_domain(Phase::Outfitting), PhaseDomain::Setup);
        assert_eq!(phase_domain(Phase::Result), PhaseDomain::Terminal);
    }

    #[test]
    fn route_transition_rules_cover_setup_and_terminal() {
        assert!(is_route_transition_allowed(
            Phase::Persona,
            Phase::Outfitting
        ));
        assert!(!is_route_transition_allowed(Phase::Persona, Phase::Travel));
        assert!(is_route_transition_allowed(Phase::Travel, Phase::Result));
        assert!(is_route_transition_allowed(Phase::Result, Phase::Menu));
        assert!(!is_route_transition_allowed(Phase::Result, Phase::Travel));
    }
}
