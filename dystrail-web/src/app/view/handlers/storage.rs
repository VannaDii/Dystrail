use crate::app::phase::{phase_for_state, session_from_state};
use crate::app::state::AppState;
use crate::game::state::GameState;
use yew::prelude::*;

pub fn build_save(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let logs_handle = state.logs.clone();
    Callback::from(move |()| {
        if let Some(sess) = (*session_handle).clone() {
            sess.state().save();
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.saved"));
            logs_handle.set(l);
        }
    })
}

pub fn build_load(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let pending_handle = state.pending_state.clone();
    let data_handle = state.data.clone();
    let logs_handle = state.logs.clone();
    let phase_handle = state.phase.clone();
    let run_seed_handle = state.run_seed.clone();
    let endgame_cfg = (*state.endgame_config).clone();
    Callback::from(move |()| {
        if let Some(mut gs) = GameState::load() {
            gs = gs.rehydrate((*data_handle).clone());
            let sess = session_from_state(gs, &endgame_cfg);
            let next_phase = phase_for_state(sess.state());
            run_seed_handle.set(sess.state().seed);
            pending_handle.set(Some(sess.state().clone()));
            session_handle.set(Some(sess));
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.loaded"));
            logs_handle.set(l);
            phase_handle.set(next_phase);
        }
    })
}

pub fn build_export_state(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    Callback::from(move |()| {
        let Some(sess) = (*session_handle).clone() else {
            return;
        };
        let Ok(text) = serde_json::to_string(sess.state()) else {
            return;
        };
        if let Some(win) = web_sys::window() {
            let nav = win.navigator();
            let cb = nav.clipboard();
            let _ = cb.write_text(&text);
        }
    })
}

pub fn build_import_state(state: &AppState) -> Callback<String> {
    let session_handle = state.session.clone();
    let pending_handle = state.pending_state.clone();
    let data_handle = state.data.clone();
    let logs_handle = state.logs.clone();
    let run_seed_handle = state.run_seed.clone();
    let phase_handle = state.phase.clone();
    let endgame_cfg = (*state.endgame_config).clone();
    Callback::from(move |txt: String| {
        if let Ok(mut gs) = serde_json::from_str::<GameState>(&txt) {
            gs = gs.rehydrate((*data_handle).clone());
            let sess = session_from_state(gs, &endgame_cfg);
            let next_phase = phase_for_state(sess.state());
            run_seed_handle.set(sess.state().seed);
            pending_handle.set(Some(sess.state().clone()));
            session_handle.set(Some(sess));
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.loaded"));
            logs_handle.set(l);
            phase_handle.set(next_phase);
        } else {
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.error"));
            logs_handle.set(l);
        }
    })
}
