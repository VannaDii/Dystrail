use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::{CrossingChoice, MechanicalPolicyId, can_afford_bribe, can_use_permit};
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
        } else if state_ref.pending_crossing.is_some() {
            phase.set(Phase::Crossing);
        } else if state_ref.current_encounter.is_some() {
            phase.set(Phase::Encounter);
        } else if boss_gate {
            phase.set(Phase::Boss);
        } else {
            phase.set(Phase::Travel);
        }

        logs.set(lg);
        session_handle.set(Some(sess));
    })
}
