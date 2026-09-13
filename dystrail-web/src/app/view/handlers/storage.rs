use crate::app::phase::session_from_state;
use crate::app::state::AppState;
use crate::game::{GameState, GameStorage, WebGameStorage, encode_friendly};
use yew::prelude::*;

pub fn build_save(state: &AppState) -> Callback<()> {
    let state = state.clone();
    Callback::from(move |()| {
        let success = state
            .pending_turn
            .as_ref()
            .map(|p| p.session.state())
            .or_else(|| {
                state
                    .session
                    .as_ref()
                    .map(crate::game::JourneySession::state)
            })
            .is_some_and(|gs| WebGameStorage.save_game("default", gs).is_ok());
        state.save_status.set(crate::i18n::t(if success {
            "save.saved"
        } else {
            "ux.save_error"
        }));
    })
}

fn restore(state: &AppState, gs: GameState) {
    state.travel_running.set(false);
    state.map_automatic.set(false);
    state.map_return.set(None);
    state
        .journey_detail
        .set(crate::components::ui::travel_panel::tabs::Selection::default());
    state.pending_turn.set(None);
    state.town_open.set(false);
    let gs = gs.rehydrate((*state.data).clone());
    let next = crate::app::aftermath::next_phase(&gs);
    state
        .code
        .set(encode_friendly(gs.mode.is_deep(), gs.seed).into());
    state.run_seed.set(gs.seed);
    state.pending_state.set(Some(gs.clone()));
    let gs_logs = gs
        .logs
        .iter()
        .map(|s| crate::i18n::log_message(s))
        .collect();
    state
        .session
        .set(Some(session_from_state(gs, &state.endgame_config)));
    state.aftermath.set(None);
    *state.action_lock.borrow_mut() = false;
    state.logs.set(gs_logs);
    state.save_status.set(crate::i18n::t("save.loaded"));
    state.show_save.set(false);
    state.phase.set(next);
}

pub fn build_load(state: &AppState) -> Callback<()> {
    let state = state.clone();
    Callback::from(move |()| match WebGameStorage.load_game("default") {
        Ok(Some(gs)) => restore(&state, gs),
        _ => state.save_status.set(crate::i18n::t("save.error")),
    })
}

pub fn build_export_state(state: &AppState) -> Callback<()> {
    let state = state.clone();
    Callback::from(move |()| {
        let Some(sess) = state
            .pending_turn
            .as_ref()
            .map(|p| &p.session)
            .or(state.session.as_ref())
        else {
            return;
        };
        let Ok(text) = serde_json::to_string(sess.state()) else {
            return;
        };
        if let Some(win) = web_sys::window() {
            let promise = win.navigator().clipboard().write_text(&text);
            let status = state.save_status.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let success = wasm_bindgen_futures::JsFuture::from(promise).await.is_ok();
                status.set(crate::i18n::t(if success {
                    "save.exported"
                } else {
                    "save.error"
                }));
            });
        }
    })
}

pub fn build_download_state(state: &AppState) -> Callback<()> {
    let state = state.clone();
    Callback::from(move |()| {
        let game = state
            .pending_turn
            .as_ref()
            .map(|pending| pending.session.state())
            .or_else(|| {
                state
                    .session
                    .as_ref()
                    .map(crate::game::JourneySession::state)
            });
        let success = game.is_some_and(|game| {
            serde_json::to_string(game).ok().is_some_and(|text| {
                crate::components::ui::save_drawer::transfer::download(&text, game.day).is_ok()
            })
        });
        state.save_status.set(crate::i18n::t(if success {
            "save.downloaded"
        } else {
            "ux.save_error"
        }));
    })
}

pub fn build_import_state(state: &AppState) -> Callback<String> {
    let state = state.clone();
    Callback::from(
        move |text: String| match serde_json::from_str::<GameState>(&text) {
            Ok(gs) => restore(&state, gs),
            Err(_) => state.save_status.set(crate::i18n::t("save.error")),
        },
    )
}
