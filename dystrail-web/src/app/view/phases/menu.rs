use crate::app::phase::{Phase, session_from_state};
use crate::app::state::AppState;
use crate::game::seed::{decode_to_seed, generate_code_from_entropy};
use crate::game::state::{GameMode, GameState};
use crate::game::{EncounterData, EndgameTravelCfg, JourneySession};
use crate::pages::menu::{MenuAction, MenuPage};
use yew::prelude::*;

struct StartRunPlan {
    code: AttrValue,
    session: JourneySession,
    pending_state: GameState,
    run_seed: u64,
    logs: Vec<String>,
    phase: Phase,
}

fn build_plan_from_seed(
    seed: u64,
    is_deep: bool,
    pending_state: Option<GameState>,
    data: &EncounterData,
    endgame_cfg: &EndgameTravelCfg,
    code: AttrValue,
) -> StartRunPlan {
    let mode = if is_deep {
        GameMode::Deep
    } else {
        GameMode::Classic
    };
    let base = pending_state.unwrap_or_default();
    let gs = base.with_seed(seed, mode, data.clone());
    let sess = session_from_state(gs, endgame_cfg);
    let mode_label = if is_deep {
        crate::i18n::t("mode.deep")
    } else {
        crate::i18n::t("mode.classic")
    };
    let mut m = std::collections::BTreeMap::new();
    m.insert("mode", mode_label.as_str());
    let logs = vec![crate::i18n::tr("log.run_begins", Some(&m))];
    StartRunPlan {
        code,
        pending_state: sess.state().clone(),
        session: sess,
        run_seed: seed,
        logs,
        phase: Phase::Travel,
    }
}

fn build_start_run_plan(
    code: &str,
    pending_state: Option<GameState>,
    data: &EncounterData,
    endgame_cfg: &EndgameTravelCfg,
    entropy: u64,
) -> Option<StartRunPlan> {
    if let Some((is_deep, seed)) = decode_to_seed(code) {
        return Some(build_plan_from_seed(
            seed,
            is_deep,
            pending_state,
            data,
            endgame_cfg,
            AttrValue::from(code),
        ));
    }

    let new_code = generate_code_from_entropy(false, entropy);
    decode_to_seed(&new_code).map(|(is_deep, seed)| {
        build_plan_from_seed(
            seed,
            is_deep,
            pending_state,
            data,
            endgame_cfg,
            AttrValue::from(new_code),
        )
    })
}

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
            let entropy = js_sys::Date::now().to_bits();
            if let Some(plan) = build_start_run_plan(
                code_handle.as_str(),
                (*pending_handle).clone(),
                &data_handle,
                &endgame_cfg,
                entropy,
            ) {
                code_handle.set(plan.code.clone());
                logs_handle.set(plan.logs);
                run_seed_handle.set(plan.run_seed);
                pending_handle.set(Some(plan.pending_state));
                session_handle.set(Some(plan.session));
                phase_handle.set(plan.phase);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_run_plan_uses_valid_code() {
        crate::i18n::set_lang("en");
        let data = EncounterData::load_from_static();
        let endgame = EndgameTravelCfg::default_config();
        let code = "CL-ORANGE42";
        let plan =
            build_start_run_plan(code, None, &data, &endgame, 0).expect("plan should be built");
        assert_eq!(plan.code.as_str(), code);
        assert_eq!(plan.phase, Phase::Travel);
        assert_eq!(plan.pending_state.seed, plan.run_seed);
        assert!(!plan.logs.is_empty());
    }

    #[test]
    fn start_run_plan_generates_code_when_invalid() {
        crate::i18n::set_lang("en");
        let data = EncounterData::load_from_static();
        let endgame = EndgameTravelCfg::default_config();
        let entropy = 123_456_u64;
        let plan = build_start_run_plan("invalid", None, &data, &endgame, entropy)
            .expect("plan should be built");
        let expected_code = generate_code_from_entropy(false, entropy);
        assert_eq!(plan.code.as_str(), expected_code);
        assert_eq!(plan.pending_state.seed, plan.run_seed);
        assert_eq!(plan.pending_state.mode, GameMode::Classic);
    }
}
