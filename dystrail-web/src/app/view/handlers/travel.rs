use crate::app::phase::{Phase, phase_for_state};
use crate::app::state::AppState;
use crate::game::state::{DietId, PaceId};
use yew::prelude::*;

pub fn build_travel(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    Callback::from(move |()| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        let outcome = sess.tick_day();

        let mut lg = (*logs).clone();
        if outcome.events.is_empty() {
            lg.push(crate::i18n::t(&outcome.log_key));
        } else {
            for event in &outcome.events {
                if let Some(key) = event.ui_key.as_deref() {
                    lg.push(crate::i18n::t(key));
                }
            }
        }
        let state_ref = sess.state();
        phase.set(phase_for_state(state_ref));

        logs.set(lg);
        session_handle.set(Some(sess));
    })
}

pub fn build_pace_change(state: &AppState) -> Callback<PaceId> {
    let session_handle = state.session.clone();
    Callback::from(move |new_pace: PaceId| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.pace = new_pace);
            session_handle.set(Some(sess));
        }
    })
}

pub fn build_diet_change(state: &AppState) -> Callback<DietId> {
    let session_handle = state.session.clone();
    Callback::from(move |new_diet: DietId| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.diet = new_diet);
            session_handle.set(Some(sess));
        }
    })
}

pub fn build_encounter_choice(state: &AppState) -> Callback<usize> {
    let session_handle = state.session.clone();
    let phase_handle = state.phase.clone();
    Callback::from(move |idx: usize| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.apply_choice(idx));
            phase_handle.set(Phase::Travel);
            session_handle.set(Some(sess));
        }
    })
}
