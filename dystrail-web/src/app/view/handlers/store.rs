use crate::app::state::AppState;
use crate::app::view::handlers::outcome::commit_outcome;
use crate::game::{MechanicalPolicyId, OtDeluxeStoreLineItem};
use yew::prelude::*;

pub fn build_store_purchase(state: &AppState) -> Callback<Vec<OtDeluxeStoreLineItem>> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    Callback::from(move |lines: Vec<OtDeluxeStoreLineItem>| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        if sess.state().mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        let applied = sess.with_state_mut(|gs| gs.set_otdeluxe_store_purchase(lines));
        if !applied {
            return;
        }
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}

pub fn build_store_leave(state: &AppState) -> Callback<()> {
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
        let applied = sess.with_state_mut(|gs| gs.set_otdeluxe_store_purchase(Vec::new()));
        if !applied {
            return;
        }
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}
