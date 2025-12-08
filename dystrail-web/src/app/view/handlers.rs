use crate::app::phase::{Phase, session_from_state};
use crate::app::state::AppState;
use crate::game::state::{DietId, GameState, PaceId, Region};
use crate::router::Route;
use yew::prelude::*;
use yew_router::prelude::Navigator;

#[derive(Clone)]
pub struct AppHandlers {
    pub travel: Callback<()>,
    pub pace_change: Callback<PaceId>,
    pub diet_change: Callback<DietId>,
    pub encounter_choice: Callback<usize>,
    pub boss: Callback<()>,
    pub save: Callback<()>,
    pub load: Callback<()>,
    pub export_state: Callback<()>,
    pub import_state: Callback<String>,
    pub lang_change: Callback<String>,
    pub toggle_hc: Callback<bool>,
    pub settings_hc_change: Callback<bool>,
    pub go_home: Callback<()>,
    pub begin_boot: Callback<()>,
}

impl AppHandlers {
    #[must_use]
    pub fn new(state: &AppState, navigator: Option<Navigator>) -> Self {
        Self {
            travel: build_travel(state),
            pace_change: build_pace_change(state),
            diet_change: build_diet_change(state),
            encounter_choice: build_encounter_choice(state),
            boss: build_boss(state),
            save: build_save(state),
            load: build_load(state),
            export_state: build_export_state(state),
            import_state: build_import_state(state),
            lang_change: build_lang_change(state),
            toggle_hc: build_toggle_hc(state),
            settings_hc_change: build_settings_hc_change(state),
            go_home: build_go_home(state, navigator),
            begin_boot: build_begin_boot(state),
        }
    }
}

fn build_travel(state: &AppState) -> Callback<()> {
    let session_handle = state.session.clone();
    let logs = state.logs.clone();
    let phase = state.phase.clone();
    let pacing_cfg = (*state.pacing_config).clone();
    Callback::from(move |()| {
        let Some(mut sess) = (*session_handle).clone() else {
            return;
        };
        sess.with_state_mut(|gs| gs.apply_pace_and_diet(&pacing_cfg));
        let outcome = sess.tick_day();

        let mut lg = (*logs).clone();
        lg.push(crate::i18n::t(&outcome.log_key));
        let state_ref = sess.state();
        if outcome.ended || state_ref.stats.pants >= 100 {
            phase.set(Phase::Result);
        } else if state_ref.current_encounter.is_some() {
            phase.set(Phase::Encounter);
        } else if matches!(state_ref.region, Region::Beltway) && state_ref.day > 12 {
            phase.set(Phase::Boss);
        }

        logs.set(lg);
        session_handle.set(Some(sess));
    })
}

fn build_pace_change(state: &AppState) -> Callback<PaceId> {
    let session_handle = state.session.clone();
    Callback::from(move |new_pace: PaceId| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.pace = new_pace);
            session_handle.set(Some(sess));
        }
    })
}

fn build_diet_change(state: &AppState) -> Callback<DietId> {
    let session_handle = state.session.clone();
    Callback::from(move |new_diet: DietId| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.diet = new_diet);
            session_handle.set(Some(sess));
        }
    })
}

fn build_encounter_choice(state: &AppState) -> Callback<usize> {
    let session_handle = state.session.clone();
    let phase_handle = state.phase.clone();
    let logs_handle = state.logs.clone();
    Callback::from(move |idx: usize| {
        if let Some(mut sess) = (*session_handle).clone() {
            sess.with_state_mut(|gs| gs.apply_choice(idx));
            let mut lg = (*logs_handle).clone();
            lg.push(format!("Chose option {}", idx + 1));
            phase_handle.set(Phase::Travel);
            logs_handle.set(lg);
            session_handle.set(Some(sess));
        }
    })
}

fn build_boss(state: &AppState) -> Callback<()> {
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

fn build_save(state: &AppState) -> Callback<()> {
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

fn build_load(state: &AppState) -> Callback<()> {
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
            run_seed_handle.set(sess.state().seed);
            pending_handle.set(Some(sess.state().clone()));
            session_handle.set(Some(sess));
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.loaded"));
            logs_handle.set(l);
            phase_handle.set(Phase::Travel);
        }
    })
}

fn build_export_state(state: &AppState) -> Callback<()> {
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

fn build_import_state(state: &AppState) -> Callback<String> {
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
            run_seed_handle.set(sess.state().seed);
            pending_handle.set(Some(sess.state().clone()));
            session_handle.set(Some(sess));
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.loaded"));
            logs_handle.set(l);
            phase_handle.set(Phase::Travel);
        } else {
            let mut l = (*logs_handle).clone();
            l.push(crate::i18n::t("save.error"));
            logs_handle.set(l);
        }
    })
}

fn build_lang_change(state: &AppState) -> Callback<String> {
    let current_language = state.current_language.clone();
    Callback::from(move |code: String| {
        crate::i18n::set_lang(&code);
        current_language.set(code);
    })
}

fn build_toggle_hc(state: &AppState) -> Callback<bool> {
    let high_contrast = state.high_contrast.clone();
    Callback::from(move |next: bool| {
        crate::a11y::set_high_contrast(next);
        high_contrast.set(next);
    })
}

fn build_settings_hc_change(state: &AppState) -> Callback<bool> {
    let high_contrast = state.high_contrast.clone();
    Callback::from(move |next: bool| {
        high_contrast.set(next);
    })
}

fn build_go_home(state: &AppState, navigator: Option<Navigator>) -> Callback<()> {
    let phase = state.phase.clone();
    Callback::from(move |()| {
        if let Some(nav) = navigator.as_ref() {
            nav.push(&Route::Home);
        }
        phase.set(Phase::Menu);
    })
}

fn build_begin_boot(state: &AppState) -> Callback<()> {
    let phase = state.phase.clone();
    let ready = state.boot_ready.clone();
    Callback::from(move |()| {
        if *ready {
            phase.set(Phase::Persona);
        }
    })
}
