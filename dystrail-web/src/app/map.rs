//! Periodic route review between the completed journey animation and the next decision.
use super::{Phase, state::AppState};
use crate::game::{GameState, route::MapCheckpoint};
use yew::prelude::*;

#[must_use]
pub fn after_transit(gs: &GameState, next: Phase) -> Phase {
    if next == Phase::Travel
        && gs.day >= 6
        && gs.stats.sanity > 2
        && gs.stats.hp > 2
        && gs.stats.supplies > 2
        && gs.stats.pants < 80
        && gs.breakdown.is_none()
        && gs.continuity.route_services.map_reviewed.as_ref() != Some(&MapCheckpoint::current(gs))
    {
        Phase::Map
    } else {
        next
    }
}

pub fn render(state: &AppState) -> Html {
    let Some(gs) = state.session.as_ref().map(|s| s.state().clone()) else {
        return Html::default();
    };
    let on_continue = {
        let state = state.clone();
        Callback::from(move |_| {
            if let Some(mut session) = (*state.session).clone() {
                session.with_state_mut(|gs| {
                    gs.continuity.route_services.map_reviewed = Some(MapCheckpoint::current(gs));
                });
                state.session.set(Some(session));
            }
            let next = state
                .session
                .as_ref()
                .map_or(Phase::Travel, |s| super::aftermath::next_phase(s.state()));
            state.phase.set(next);
        })
    };
    html! {<crate::components::ui::route_map::RouteMap state={std::rc::Rc::new(gs)} {on_continue}/>}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_wait_for_a_safe_gap_and_never_hide_a_decision() {
        let mut gs = GameState {
            day: 6,
            ..GameState::default()
        };
        gs.stats.supplies = 10;
        assert!(after_transit(&gs, Phase::Travel) == Phase::Map);
        for pending in [
            Phase::Encounter,
            Phase::Town,
            Phase::CrewCare,
            Phase::Boss,
            Phase::Result,
        ] {
            assert!(after_transit(&gs, pending) == pending);
        }
        gs.stats.sanity = 2;
        assert!(after_transit(&gs, Phase::Travel) == Phase::Travel);
        gs.stats.sanity = 10;
        gs.continuity.route_services.map_reviewed = Some(MapCheckpoint::current(&gs));
        assert!(after_transit(&gs, Phase::Travel) == Phase::Travel);
    }
}
