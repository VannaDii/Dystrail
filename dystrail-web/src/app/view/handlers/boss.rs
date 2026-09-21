use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::boss::{HearingOutcome, HearingPhase};
use yew::prelude::*;

#[derive(Clone, Copy)]
pub enum HearingCommand {
    Advance(HearingPhase),
    Skip(HearingPhase),
}

pub fn build_hearing(state: &AppState) -> Callback<HearingCommand> {
    let app = state.clone();
    Callback::from(move |command| {
        if *app.action_lock.borrow() {
            return;
        }
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        let (expected, skip) = match command {
            HearingCommand::Advance(phase) => (phase, false),
            HearingCommand::Skip(phase) => (phase, true),
        };
        if session.state().boss.presentation != expected {
            return;
        }
        let next = if let Some(report) = &session.state().boss.hearing {
            if skip {
                HearingPhase::Verdict
            } else {
                report.next_phase(expected)
            }
        } else if expected == HearingPhase::Arrival {
            HearingPhase::Preparation
        } else {
            return;
        };
        if next == expected {
            return;
        }
        *app.action_lock.borrow_mut() = true;
        session.with_state_mut(|gs| gs.boss.presentation = next);
        app.session.set(Some(session));
        if next == HearingPhase::Complete {
            app.phase.set(Phase::Result);
        }
    })
}

pub fn build_boss(state: &AppState) -> Callback<()> {
    let app = state.clone();
    let session_handle = state.session.clone();
    let lock = state.action_lock.clone();
    let boss_config_handle = state.boss_config.clone();
    Callback::from(move |()| {
        if *lock.borrow() {
            return;
        }
        if let Some(mut sess) = (*session_handle).clone() {
            if !sess.state().boss.readiness.ready || sess.state().boss.outcome.attempted {
                return;
            }
            *lock.borrow_mut() = true;
            let before = sess.state().clone();
            let cfg = (*boss_config_handle).clone();
            let _ = sess.with_state_mut(|gs| crate::game::boss::run_boss_minigame(gs, &cfg));
            sess.with_state_mut(|gs| {
                let mut report = crate::app::aftermath::Aftermath {
                    title: crate::i18n::t("boss.title"),
                    message: crate::i18n::t(
                        if gs
                            .boss
                            .hearing
                            .as_ref()
                            .is_some_and(|r| r.outcome == HearingOutcome::Exhausted)
                        {
                            "hearing.exhausted_body"
                        } else if gs.boss.outcome.victory {
                            "journey.vote_won"
                        } else {
                            "journey.vote_lost"
                        },
                    ),
                    scene: crate::components::ui::journey_scene::SceneStage::Boss,
                    before: before.stats.clone(),
                    after: gs.stats.clone(),
                    resources: Vec::new(),
                    details: crate::app::receipt::resource_details(&before, gs),
                    next: Phase::Result,
                };
                crate::app::history::record(&before, gs, &mut report, 120);
            });
            app.travel_running.set(false);
            app.aftermath.set(None);
            session_handle.set(Some(sess));
        }
    })
}
