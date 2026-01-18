use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::outcome::commit_outcome;
use crate::game::MechanicalPolicyId;
use crate::game::state::{DayIntent, DietId, PaceId};
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
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}

pub fn build_trade(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    Callback::from(move |()| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        if sess.state().mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        sess.with_state_mut(|gs| gs.intent.pending = DayIntent::Trade);
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}

pub fn build_hunt(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    Callback::from(move |()| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        if sess.state().mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        sess.with_state_mut(|gs| gs.intent.pending = DayIntent::Hunt);
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
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
