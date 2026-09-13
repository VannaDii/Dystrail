//! Periodic route review between the completed journey animation and the next decision.
use super::{Phase, flow::TravelSpeed, state::AppState};
use crate::game::{GameState, route::MapCheckpoint};
use yew::prelude::*;

#[must_use]
pub fn after_transit(gs: &GameState, next: Phase, speed: TravelSpeed) -> Phase {
    if speed != TravelSpeed::Fast
        && next == Phase::Travel
        && gs.day >= 6
        && safe_for_preview(gs)
        && gs.continuity.route_services.map_reviewed.as_ref() != Some(&MapCheckpoint::current(gs))
    {
        Phase::Map
    } else {
        next
    }
}

#[must_use]
pub fn transition(gs: &GameState, next: Phase, automatic_open: bool, speed: TravelSpeed) -> Phase {
    if automatic_open {
        Phase::Map
    } else {
        after_transit(gs, next, speed)
    }
}

const fn safe_for_preview(gs: &GameState) -> bool {
    gs.stats.sanity > 2 && gs.stats.hp > 2 && gs.stats.supplies > 2 && gs.breakdown.is_none()
}

pub fn render(state: &AppState) -> Html {
    let Some(gs) = state.session.as_ref().map(|s| s.state().clone()) else {
        return Html::default();
    };
    let on_continue = {
        let state = state.clone();
        Callback::from(move |()| {
            if !*state.map_automatic {
                let previous = (*state.map_return).unwrap_or_else(|| {
                    state
                        .session
                        .as_ref()
                        .map_or(Phase::Travel, |s| super::aftermath::next_phase(s.state()))
                });
                state.travel_running.set(false);
                state.map_return.set(None);
                state.phase.set(previous);
                return;
            }
            if let Some(mut session) = (*state.session).clone() {
                session.with_state_mut(|gs| {
                    gs.continuity.route_services.map_reviewed = Some(MapCheckpoint::current(gs));
                });
                state.session.set(Some(session));
            }
            if let Some(pending) = state.pending_turn.as_ref() {
                let mut session = pending.session.clone();
                session.with_state_mut(|gs| {
                    gs.continuity.route_services.map_reviewed = Some(MapCheckpoint::current(gs));
                });
                state
                    .pending_turn
                    .set(Some(std::rc::Rc::new(super::turn::PendingTurn {
                        session,
                        report: pending.report.clone(),
                        logs: pending.logs.clone(),
                    })));
            }
            let next = state
                .session
                .as_ref()
                .map_or(Phase::Travel, |s| super::aftermath::next_phase(s.state()));
            state.phase.set(next);
            state.travel_running.set(next == Phase::Travel);
            state.map_automatic.set(false);
            state.map_return.set(None);
        })
    };
    let automatic = *state.map_automatic;
    let running = automatic && *state.travel_running && !*state.show_save && !*state.show_abandon;
    let on_pause = {
        let running = state.travel_running.clone();
        Callback::from(move |_| running.set(false))
    };
    html! {<crate::components::ui::route_map::RouteMap state={std::rc::Rc::new(gs)} {automatic} {running} on_continue={{let next=on_continue.clone();Callback::from(move |_|next.emit(()))}} {on_pause}>
        if running {<super::map_countdown::MapCountdown on_complete={on_continue} />}
    </crate::components::ui::route_map::RouteMap>}
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
        assert!(after_transit(&gs, Phase::Travel, TravelSpeed::Fast) == Phase::Travel);
        assert!(after_transit(&gs, Phase::Travel, TravelSpeed::Normal) == Phase::Map);
        for pending in [
            Phase::Encounter,
            Phase::Town,
            Phase::CrewCare,
            Phase::AllyLoss,
            Phase::Boss,
            Phase::Result,
        ] {
            assert!(after_transit(&gs, pending, TravelSpeed::Normal) == pending);
            assert!(transition(&gs, pending, true, TravelSpeed::Normal) == Phase::Map);
        }
        gs.stats.sanity = 2;
        assert!(after_transit(&gs, Phase::Travel, TravelSpeed::Normal) == Phase::Travel);
        assert!(transition(&gs, Phase::Travel, true, TravelSpeed::Normal) == Phase::Map);
        gs.stats.sanity = 10;
        gs.continuity.route_services.map_reviewed = Some(MapCheckpoint::current(&gs));
        assert!(after_transit(&gs, Phase::Travel, TravelSpeed::Normal) == Phase::Travel);
    }
}
