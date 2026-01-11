use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::MechanicalPolicyId;
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
        let boss_gate = state_ref.mechanical_policy == MechanicalPolicyId::DystrailLegacy
            && state_ref.boss.readiness.ready
            && !state_ref.boss.outcome.attempted;
        if outcome.ended || state_ref.stats.pants >= 100 {
            phase.set(Phase::Result);
        } else if state_ref.current_encounter.is_some() {
            phase.set(Phase::Encounter);
        } else if boss_gate {
            phase.set(Phase::Boss);
        }

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
    let logs_handle = state.logs.clone();
    Callback::from(move |idx: usize| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.apply_choice(idx));
            let mut lg = (*logs_handle).clone();
            lg.push(format!("Chose option {}", idx + 1));
            phase_handle.set(Phase::Travel);
            logs_handle.set(lg);
            session_handle.set(Some(sess));
        }
    })
}
