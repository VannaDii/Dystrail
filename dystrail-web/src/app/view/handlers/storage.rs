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
    build_load_with(state, GameState::load)
}

fn build_load_with<F>(state: &AppState, load_fn: F) -> Callback<()>
where
    F: Fn() -> Option<GameState> + 'static,
{
    let session_handle = state.session.clone();
    let pending_handle = state.pending_state.clone();
    let data_handle = state.data.clone();
    let logs_handle = state.logs.clone();
    let phase_handle = state.phase.clone();
    let run_seed_handle = state.run_seed.clone();
    let endgame_cfg = (*state.endgame_config).clone();
    Callback::from(move |()| {
        if let Some(mut gs) = load_fn() {
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
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(win) = web_sys::window() {
                let nav = win.navigator();
                let cb = nav.clipboard();
                let _ = cb.write_text(&text);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = text;
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

#[cfg(test)]
mod tests {
    use super::build_load_with;
    use crate::app::phase::Phase;
    use crate::app::state::AppState;
    use crate::game::data::EncounterData;
    use crate::game::state::GameMode;
    use crate::game::{EndgameTravelCfg, JourneySession};
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;
    use yew::prelude::*;

    #[function_component(LoadHarness)]
    fn load_harness() -> Html {
        crate::i18n::set_lang("en");
        let invoked = use_state(|| false);
        let data = EncounterData::load_from_static();
        let data_for_state = data.clone();
        let state = AppState {
            phase: use_state(|| Phase::Menu),
            code: use_state(|| AttrValue::from("CL-TEST01")),
            data: use_state(move || data_for_state),
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 100),
            boot_ready: use_state(|| true),
            high_contrast: use_state(|| false),
            pending_state: use_state(|| None),
            session: use_state(|| None::<JourneySession>),
            logs: use_state(Vec::<String>::new),
            run_seed: use_state(|| 0_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        };

        let load_called = Rc::new(Cell::new(false));
        let load_called_ref = load_called.clone();
        let load_cb = build_load_with(&state, move || {
            load_called_ref.set(true);
            Some(crate::game::GameState::default().with_seed(7, GameMode::Classic, data.clone()))
        });

        if !*invoked {
            invoked.set(true);
            load_cb.emit(());
        }

        let called = load_called.get();
        html! {
            <div data-called={called.to_string()} />
        }
    }

    #[test]
    fn build_load_sets_state_from_loader() {
        let html = block_on(LocalServerRenderer::<LoadHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
