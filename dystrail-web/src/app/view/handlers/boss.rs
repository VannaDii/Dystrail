use crate::app::phase::Phase;
use crate::app::state::AppState;
use yew::prelude::*;

pub fn build_boss(state: &AppState) -> Callback<()> {
    let app = state.clone();
    let session_handle = state.session.clone();
    let lock = state.action_lock.clone();
    let phase_handle = state.phase.clone();
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
                    message: crate::i18n::t(if gs.boss.outcome.victory {
                        "journey.vote_won"
                    } else {
                        "journey.vote_lost"
                    }),
                    scene: crate::components::ui::journey_scene::SceneStage::Boss,
                    before: before.stats.clone(),
                    after: gs.stats.clone(),
                    details: crate::app::receipt::resource_details(&before, gs),
                    next: Phase::Result,
                };
                crate::app::history::record(&before, gs, &mut report, 120);
                crate::app::history::publish(&app, report, false);
            });
            phase_handle.set(Phase::Result);
            session_handle.set(Some(sess));
        }
    })
}
