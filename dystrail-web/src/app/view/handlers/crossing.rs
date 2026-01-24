use super::outcome::commit_outcome;
use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::{
    CrossingChoice, MechanicalPolicyId, OtDeluxe90sPolicy, OtDeluxeCrossingMethod,
    can_afford_bribe, can_use_permit, otdeluxe_crossing_options,
};
use yew::prelude::*;

pub fn build_crossing_choice(state: &AppState) -> Callback<u8> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    let crossing_config = state.crossing_config.clone();

    Callback::from(move |idx: u8| {
        if idx == 0 {
            phase.set(Phase::Travel);
            return;
        }

        let choice = match idx {
            1 => CrossingChoice::Detour,
            2 => CrossingChoice::Bribe,
            3 => CrossingChoice::Permit,
            _ => return,
        };

        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        let cfg = (*crossing_config).clone();
        let kind = match sess.state().pending_crossing {
            Some(pending) => pending.kind,
            None => return,
        };
        let allowed = match choice {
            CrossingChoice::Detour => true,
            CrossingChoice::Bribe => can_afford_bribe(sess.state(), &cfg, kind),
            CrossingChoice::Permit => can_use_permit(sess.state(), &kind),
        };
        if !allowed {
            return;
        }

        sess.with_state_mut(|gs| gs.set_crossing_choice(choice));
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}

pub fn build_otdeluxe_crossing_choice(state: &AppState) -> Callback<u8> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();

    Callback::from(move |idx: u8| {
        if idx == 0 {
            phase.set(Phase::Travel);
            return;
        }

        let method = match idx {
            1 => OtDeluxeCrossingMethod::Ford,
            2 => OtDeluxeCrossingMethod::CaulkFloat,
            3 => OtDeluxeCrossingMethod::Ferry,
            4 => OtDeluxeCrossingMethod::Guide,
            _ => return,
        };

        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        if sess.state().mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return;
        }
        let Some(river_kind) = sess.state().ot_deluxe.crossing.river_kind else {
            return;
        };
        let Some(river_state) = sess.state().ot_deluxe.crossing.river.as_ref() else {
            return;
        };
        let policy = OtDeluxe90sPolicy::default();
        let options = otdeluxe_crossing_options(
            &policy.crossings,
            river_kind,
            river_state,
            &sess.state().ot_deluxe.inventory,
        );
        if !options.is_allowed(method) {
            return;
        }

        sess.with_state_mut(|gs| gs.set_otdeluxe_crossing_choice(method));
        let outcome = sess.tick_day();
        commit_outcome(sess, &outcome, &logs, &phase, &session_handle);
    })
}
