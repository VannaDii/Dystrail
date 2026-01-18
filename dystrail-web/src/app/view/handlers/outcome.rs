use crate::app::phase::{Phase, phase_for_state};
use crate::game::{DayOutcome, JourneySession};
use std::ops::Deref;
use yew::prelude::*;

pub fn commit_outcome(
    sess: JourneySession,
    outcome: &DayOutcome,
    logs: &UseStateHandle<Vec<String>>,
    phase: &UseStateHandle<Phase>,
    session_handle: &UseStateHandle<Option<JourneySession>>,
) {
    let mut lg = logs.deref().clone();
    if outcome.events.is_empty() {
        lg.push(crate::i18n::t(&outcome.log_key));
    } else {
        for event in &outcome.events {
            if let Some(key) = event.ui_key.as_deref() {
                lg.push(crate::i18n::t(key));
            }
        }
    }
    phase.set(phase_for_state(sess.state()));
    logs.set(lg);
    session_handle.set(Some(sess));
}
