use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

pub fn build_lang_change(state: &AppState) -> Callback<String> {
    let current_language = state.current_language.clone();
    Callback::from(move |code: String| {
        crate::i18n::set_lang(&code);
        current_language.set(code);
    })
}

pub fn build_toggle_hc(state: &AppState) -> Callback<bool> {
    let high_contrast = state.high_contrast.clone();
    Callback::from(move |next: bool| {
        crate::a11y::set_high_contrast(next);
        high_contrast.set(next);
    })
}

pub fn build_settings_hc_change(state: &AppState) -> Callback<bool> {
    let high_contrast = state.high_contrast.clone();
    Callback::from(move |next: bool| {
        high_contrast.set(next);
    })
}

pub fn build_go_home(state: &AppState, navigator: Option<Navigator>) -> Callback<()> {
    let phase = state.phase.clone();
    let session = state.session.clone();
    let pending = state.pending_state.clone();
    let logs = state.logs.clone();
    let run_seed = state.run_seed.clone();
    let code = state.code.clone();
    let show_save = state.show_save.clone();
    let show_settings = state.show_settings.clone();
    Callback::from(move |()| {
        let _ = navigator.as_ref().map(|nav| nav.push(&Route::Menu));
        session.set(None);
        pending.set(None);
        logs.set(Vec::new());
        run_seed.set(0);
        code.set(AttrValue::from(""));
        show_save.set(false);
        show_settings.set(false);
        phase.set(Phase::Menu);
    })
}

pub fn build_begin_boot(state: &AppState) -> Callback<()> {
    let phase = state.phase.clone();
    let ready = state.boot_ready.clone();
    let session = state.session.clone();
    let pending = state.pending_state.clone();
    let logs = state.logs.clone();
    let run_seed = state.run_seed.clone();
    let code = state.code.clone();
    let show_save = state.show_save.clone();
    let show_settings = state.show_settings.clone();
    Callback::from(move |()| {
        // Only advance when explicitly called (user presses key)
        if *ready {
            session.set(None);
            pending.set(None);
            logs.set(Vec::new());
            run_seed.set(0);
            code.set(AttrValue::from(""));
            show_save.set(false);
            show_settings.set(false);
            phase.set(Phase::Menu);
        }
    })
}
