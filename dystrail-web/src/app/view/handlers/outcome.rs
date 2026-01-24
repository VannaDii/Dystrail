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
    lg.extend(outcome_log_entries(outcome));
    phase.set(phase_for_state(sess.state()));
    logs.set(lg);
    session_handle.set(Some(sess));
}

fn outcome_log_entries(outcome: &DayOutcome) -> Vec<String> {
    if outcome.events.is_empty() {
        vec![crate::i18n::t(&outcome.log_key)]
    } else {
        outcome
            .events
            .iter()
            .filter_map(|event| event.ui_key.as_deref())
            .map(crate::i18n::t)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{commit_outcome, outcome_log_entries};
    use crate::app::phase::Phase;
    use crate::game::journey::event::{Event, EventId};
    use crate::game::journey::{DayEffects, DayInputs, DayOutcome, MechanicalPolicyId};
    use crate::game::state::{DayIntent, DietId, GameMode, PaceId, Region, Season};
    use crate::game::weather::Weather;
    use crate::game::{EndgameTravelCfg, JourneySession, StrategyId};
    use futures::executor::block_on;
    use std::rc::Rc;
    use yew::LocalServerRenderer;
    use yew::prelude::*;

    fn base_inputs() -> DayInputs {
        DayInputs {
            day: 1,
            intent: DayIntent::Continue,
            pace: PaceId::Steady,
            diet: DietId::Mixed,
            region: Region::Heartland,
            season: Season::Spring,
            mode: GameMode::Classic,
            mechanical_policy: MechanicalPolicyId::DystrailLegacy,
            weather: Weather::Clear,
        }
    }

    fn outcome_with_events(events: Vec<Event>, log_key: &str) -> DayOutcome {
        DayOutcome {
            ended: false,
            log_key: log_key.to_string(),
            breakdown_started: false,
            day_consumed: false,
            inputs: base_inputs(),
            effects: DayEffects::default(),
            record: None,
            events,
            decision_traces: Vec::new(),
        }
    }

    fn base_session() -> JourneySession {
        let state = crate::game::GameState::default();
        JourneySession::from_state(
            state,
            StrategyId::Balanced,
            &EndgameTravelCfg::default_config(),
        )
    }

    #[derive(Properties, Clone)]
    struct CommitOutcomeHarnessProps {
        outcome: Rc<DayOutcome>,
    }

    impl PartialEq for CommitOutcomeHarnessProps {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.outcome, &other.outcome)
        }
    }

    #[function_component(CommitOutcomeHarness)]
    fn commit_outcome_harness(props: &CommitOutcomeHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let invoked = use_state(|| false);
        let logs = use_state(Vec::<String>::new);
        let phase = use_state(|| Phase::Menu);
        let session_handle = use_state(|| None::<JourneySession>);

        if !*invoked {
            invoked.set(true);
            let sess = base_session();
            commit_outcome(sess, &props.outcome, &logs, &phase, &session_handle);
        }

        let log_text = outcome_log_entries(&props.outcome).join("|");
        html! { <div data-logs={log_text} /> }
    }

    #[test]
    fn commit_outcome_uses_log_key_when_no_events() {
        crate::i18n::set_lang("en");
        let expected = crate::i18n::t("save.saved");
        let outcome = Rc::new(outcome_with_events(Vec::new(), "save.saved"));
        let entries = outcome_log_entries(&outcome);
        assert!(entries.iter().any(|entry| entry == &expected));
        let _ = block_on(
            LocalServerRenderer::<CommitOutcomeHarness>::with_props(CommitOutcomeHarnessProps {
                outcome,
            })
            .render(),
        );
    }

    #[test]
    fn commit_outcome_uses_event_keys_when_present() {
        crate::i18n::set_lang("en");
        let expected = crate::i18n::t("save.loaded");
        let event = Event::legacy_log_key(EventId::new(1, 0), 1, "save.loaded");
        let outcome = Rc::new(outcome_with_events(vec![event], "save.saved"));
        let entries = outcome_log_entries(&outcome);
        assert!(entries.iter().any(|entry| entry == &expected));
        let _ = block_on(
            LocalServerRenderer::<CommitOutcomeHarness>::with_props(CommitOutcomeHarnessProps {
                outcome,
            })
            .render(),
        );
    }
}
