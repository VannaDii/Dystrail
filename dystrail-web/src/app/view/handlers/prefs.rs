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
    Callback::from(move |()| {
        if let Some(nav) = navigator.as_ref() {
            nav.push(&Route::Home);
        }
        phase.set(Phase::Menu);
    })
}

pub fn build_begin_boot(state: &AppState) -> Callback<()> {
    let phase = state.phase.clone();
    let ready = state.boot_ready.clone();
    Callback::from(move |()| {
        // Only advance when explicitly called (user presses key)
        if *ready {
            phase.set(Phase::Persona);
        }
    })
}
