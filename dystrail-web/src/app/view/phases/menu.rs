use crate::app::phase::{Phase, session_from_state};
use crate::app::state::AppState;
use crate::game::seed::{decode_to_seed, generate_code_from_entropy};
use crate::game::state::GameMode;
use crate::pages::menu::{MenuAction, MenuPage};
use yew::prelude::*;

pub fn render_menu(state: &AppState) -> Html {
    let start_with_code_action = {
        let code_handle = state.code.clone();
        let pending_handle = state.pending_state.clone();
        let phase_handle = state.phase.clone();
        let logs_handle = state.logs.clone();
        let data_handle = state.data.clone();
        let run_seed_handle = state.run_seed.clone();
        let session_handle = state.session.clone();
        let endgame_cfg = (*state.endgame_config).clone();
        move || {
            if let Some((is_deep, seed)) = decode_to_seed(&code_handle) {
                let mode = if is_deep {
                    GameMode::Deep
                } else {
                    GameMode::Classic
                };
                let base = (*pending_handle).clone().unwrap_or_default();
                let gs = base.with_seed(seed, mode, (*data_handle).clone());
                let sess = session_from_state(gs, &endgame_cfg);
                let mode_label = if is_deep {
                    crate::i18n::t("mode.deep")
                } else {
                    crate::i18n::t("mode.classic")
                };
                let mut m = std::collections::BTreeMap::new();
                m.insert("mode", mode_label.as_str());
                logs_handle.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                run_seed_handle.set(seed);
                pending_handle.set(Some(sess.state().clone()));
                session_handle.set(Some(sess));
                phase_handle.set(Phase::Travel);
            } else {
                let entropy = js_sys::Date::now().to_bits();
                let new_code = generate_code_from_entropy(false, entropy);
                code_handle.set(new_code.clone().into());
                if let Some((_, seed)) = decode_to_seed(&new_code) {
                    let base = (*pending_handle).clone().unwrap_or_default();
                    let gs = base.with_seed(seed, GameMode::Classic, (*data_handle).clone());
                    let sess = session_from_state(gs, &endgame_cfg);
                    let mode_label = crate::i18n::t("mode.classic");
                    let mut m = std::collections::BTreeMap::new();
                    m.insert("mode", mode_label.as_str());
                    logs_handle.set(vec![crate::i18n::tr("log.run_begins", Some(&m))]);
                    run_seed_handle.set(seed);
                    pending_handle.set(Some(sess.state().clone()));
                    session_handle.set(Some(sess));
                    phase_handle.set(Phase::Travel);
                }
            }
        }
    };

    let on_action = {
        let phase_handle = state.phase.clone();
        let show_save_handle = state.show_save.clone();
        let show_settings_handle = state.show_settings.clone();
        let save_focus = state.save_focus_target.clone();
        Callback::from(move |action: MenuAction| match action {
            MenuAction::StartRun => start_with_code_action(),
            MenuAction::CampPreview => phase_handle.set(Phase::Camp),
            MenuAction::OpenSave => {
                save_focus.set(AttrValue::from("save-open-btn"));
                show_save_handle.set(true);
            }
            MenuAction::OpenSettings => show_settings_handle.set(true),
            MenuAction::Reset => phase_handle.set(Phase::Boot),
        })
    };

    let menu_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
    html! {
        <MenuPage
            code={(*state.code).clone()}
            logo_src={menu_logo_src}
            {on_action}
        />
    }
}
