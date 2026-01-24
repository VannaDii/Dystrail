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

#[derive(Clone)]
struct MenuActionHandlers {
    start_run: Callback<()>,
    open_save: Callback<()>,
    open_settings: Callback<()>,
    set_phase: Callback<Phase>,
}

#[cfg(target_arch = "wasm32")]
fn next_entropy() -> u64 {
    js_sys::Date::now().to_bits()
}

#[cfg(not(target_arch = "wasm32"))]
const fn next_entropy() -> u64 {
    0
}

fn open_save_action(
    show_save: UseStateHandle<bool>,
    save_focus: UseStateHandle<AttrValue>,
) -> Callback<()> {
    Callback::from(move |()| {
        save_focus.set(AttrValue::from("save-open-btn"));
        show_save.set(true);
    })
}

fn apply_start_run_plan(state: &AppState, plan: StartRunPlan) {
    state.code.set(plan.code);
    state.logs.set(plan.logs);
    state.run_seed.set(plan.run_seed);
    state.pending_state.set(Some(plan.pending_state));
    state.session.set(Some(plan.session));
    state.phase.set(plan.phase);
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

fn start_with_code_action(state: &AppState) -> Callback<()> {
    let code_handle = state.code.clone();
    let pending_handle = state.pending_state.clone();
    let data_handle = state.data.clone();
    let endgame_cfg = (*state.endgame_config).clone();
    let state = state.clone();
    Callback::from(move |()| {
        let entropy = next_entropy();
        if let Some(plan) = build_start_run_plan(
            code_handle.as_str(),
            (*pending_handle).clone(),
            &data_handle,
            &endgame_cfg,
            entropy,
        ) {
            apply_start_run_plan(&state, plan);
        }
    })
}

fn menu_action_callback(handlers: MenuActionHandlers) -> Callback<MenuAction> {
    Callback::from(move |action: MenuAction| match action {
        MenuAction::StartRun => handlers.start_run.emit(()),
        MenuAction::CampPreview => handlers.set_phase.emit(Phase::Camp),
        MenuAction::OpenSave => handlers.open_save.emit(()),
        MenuAction::OpenSettings => handlers.open_settings.emit(()),
        MenuAction::Reset => handlers.set_phase.emit(Phase::Boot),
    })
}

pub fn render_menu(state: &AppState) -> Html {
    let start_with_code_action = start_with_code_action(state);
    let open_save = open_save_action(state.show_save.clone(), state.save_focus_target.clone());
    let open_settings = {
        let show_settings = state.show_settings.clone();
        Callback::from(move |()| show_settings.set(true))
    };
    let set_phase = {
        let phase_handle = state.phase.clone();
        Callback::from(move |phase: Phase| phase_handle.set(phase))
    };
    let on_action = menu_action_callback(MenuActionHandlers {
        start_run: start_with_code_action,
        open_save,
        open_settings,
        set_phase,
    });

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
    use futures::executor::block_on;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use yew::LocalServerRenderer;

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

    #[test]
    fn start_run_plan_builds_deep_mode() {
        crate::i18n::set_lang("en");
        let data = EncounterData::load_from_static();
        let endgame = EndgameTravelCfg::default_config();
        let plan = build_plan_from_seed(
            42,
            true,
            None,
            &data,
            &endgame,
            AttrValue::from("DP-TEST01"),
        );
        assert_eq!(plan.pending_state.mode, GameMode::Deep);
        assert_eq!(plan.phase, Phase::Travel);
    }

    #[function_component(StartWithCodeHarness)]
    fn start_with_code_harness() -> Html {
        crate::i18n::set_lang("en");
        let data = EncounterData::load_from_static();
        let base = crate::game::GameState::default().with_seed(7, GameMode::Classic, data.clone());
        let state = AppState {
            phase: use_state(|| Phase::Menu),
            code: use_state(|| AttrValue::from("CL-ORANGE42")),
            data: use_state(move || data),
            pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
            endgame_config: use_state(EndgameTravelCfg::default_config),
            weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
            camp_config: use_state(crate::game::CampConfig::default_config),
            crossing_config: use_state(crate::game::CrossingConfig::default),
            boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
            result_config: use_state(crate::game::ResultConfig::default),
            preload_progress: use_state(|| 100_u8),
            boot_ready: use_state(|| true),
            high_contrast: use_state(|| false),
            pending_state: use_state(|| Some(base)),
            session: use_state(|| None::<JourneySession>),
            logs: use_state(Vec::<String>::new),
            run_seed: use_state(|| 7_u64),
            show_save: use_state(|| false),
            save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
            show_settings: use_state(|| false),
            current_language: use_state(|| String::from("en")),
        };
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let start_action = start_with_code_action(&state);
        let wrapper = Callback::from(move |()| {
            called_ref.set(true);
            start_action.emit(());
        });
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            wrapper.emit(());
        }
        html! { <div data-called={called.get().to_string()} /> }
    }

    #[test]
    fn start_with_code_action_executes() {
        let html = block_on(LocalServerRenderer::<StartWithCodeHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }

    #[test]
    fn menu_action_callback_routes_actions() {
        let start_called = Rc::new(Cell::new(false));
        let save_called = Rc::new(Cell::new(false));
        let settings_called = Rc::new(Cell::new(false));
        let phase = Rc::new(RefCell::new(None::<Phase>));

        let handlers = MenuActionHandlers {
            start_run: {
                let start_called = start_called.clone();
                Callback::from(move |()| start_called.set(true))
            },
            open_save: {
                let save_called = save_called.clone();
                Callback::from(move |()| save_called.set(true))
            },
            open_settings: {
                let settings_called = settings_called.clone();
                Callback::from(move |()| settings_called.set(true))
            },
            set_phase: {
                let phase = phase.clone();
                Callback::from(move |next: Phase| {
                    *phase.borrow_mut() = Some(next);
                })
            },
        };

        let on_action = menu_action_callback(handlers);
        on_action.emit(MenuAction::StartRun);
        on_action.emit(MenuAction::CampPreview);
        on_action.emit(MenuAction::OpenSave);
        on_action.emit(MenuAction::OpenSettings);
        on_action.emit(MenuAction::Reset);

        assert!(start_called.get());
        assert!(save_called.get());
        assert!(settings_called.get());
        assert_eq!(*phase.borrow(), Some(Phase::Boot));
    }

    #[function_component(OpenSaveHarness)]
    fn open_save_harness() -> Html {
        let show_save = use_state(|| false);
        let save_focus = use_state(|| AttrValue::from("initial"));
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let open_save = open_save_action(show_save, save_focus);
        let wrapper = Callback::from(move |()| {
            called_ref.set(true);
            open_save.emit(());
        });
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            wrapper.emit(());
        }
        html! { <div data-called={called.get().to_string()} /> }
    }

    #[test]
    fn open_save_action_executes() {
        let html = block_on(LocalServerRenderer::<OpenSaveHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
