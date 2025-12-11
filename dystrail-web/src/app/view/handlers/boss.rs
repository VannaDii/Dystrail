use crate::app::phase::Phase;
use crate::app::state::AppState;
use yew::prelude::*;

pub fn build_boss(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let phase_handle = state.phase.clone();
    let boss_config_handle = state.boss_config.clone();
    Callback::from(move |()| {
        if let Some(mut sess) = (*session_handle).clone() {
            let cfg = (*boss_config_handle).clone();
            let _ = sess.with_state_mut(|gs| crate::game::boss::run_boss_minigame(gs, &cfg));
            phase_handle.set(Phase::Result);
            session_handle.set(Some(sess));
        }
    })
}
