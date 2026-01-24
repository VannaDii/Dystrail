use super::outcome::commit_outcome;
use crate::app::state::AppState;
use crate::game::{MechanicalPolicyId, OtDeluxeRouteDecision};
use yew::prelude::*;

pub fn build_route_prompt_choice(state: &AppState) -> Callback<OtDeluxeRouteDecision> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();

    Callback::from(move |decision: OtDeluxeRouteDecision| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        if sess.state().mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        if sess.state().ot_deluxe.route.pending_prompt.is_none() {
            return;
        }

        sess.with_state_mut(|gs| gs.set_route_prompt_choice(decision));
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}
