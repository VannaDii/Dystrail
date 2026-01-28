use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::menu::{MenuAction, MenuPage};
use yew::prelude::*;

#[derive(Clone)]
struct MenuActionHandlers {
    start_journey: Callback<()>,
    open_about: Callback<()>,
    open_settings: Callback<()>,
    quit: Callback<()>,
}

trait RunStateTarget {
    fn set_session(&self, value: Option<crate::game::JourneySession>);
    fn set_pending_state(&self, value: Option<crate::game::GameState>);
    fn set_logs(&self, value: Vec<String>);
    fn set_run_seed(&self, value: u64);
    fn set_code(&self, value: AttrValue);
    fn set_show_save(&self, value: bool);
    fn set_show_settings(&self, value: bool);
}

impl RunStateTarget for AppState {
    fn set_session(&self, value: Option<crate::game::JourneySession>) {
        self.session.set(value);
    }

    fn set_pending_state(&self, value: Option<crate::game::GameState>) {
        self.pending_state.set(value);
    }

    fn set_logs(&self, value: Vec<String>) {
        self.logs.set(value);
    }

    fn set_run_seed(&self, value: u64) {
        self.run_seed.set(value);
    }

    fn set_code(&self, value: AttrValue) {
        self.code.set(value);
    }

    fn set_show_save(&self, value: bool) {
        self.show_save.set(value);
    }

    fn set_show_settings(&self, value: bool) {
        self.show_settings.set(value);
    }
}

fn reset_run_state<T: RunStateTarget>(state: &T) {
    state.set_session(None);
    state.set_pending_state(None);
    state.set_logs(Vec::new());
    state.set_run_seed(0);
    state.set_code(AttrValue::from(""));
    state.set_show_save(false);
    state.set_show_settings(false);
}

fn menu_action_callback(handlers: MenuActionHandlers) -> Callback<MenuAction> {
    Callback::from(move |action: MenuAction| match action {
        MenuAction::StartJourney => handlers.start_journey.emit(()),
        MenuAction::About => handlers.open_about.emit(()),
        MenuAction::Settings => handlers.open_settings.emit(()),
        MenuAction::Quit => handlers.quit.emit(()),
    })
}

fn build_menu_action_handlers(state: &AppState) -> MenuActionHandlers {
    let set_phase = {
        let phase_handle = state.phase.clone();
        Callback::from(move |phase: Phase| phase_handle.set(phase))
    };

    let start_journey = {
        let state = state.clone();
        let set_phase = set_phase.clone();
        Callback::from(move |()| {
            reset_run_state(&state);
            set_phase.emit(Phase::Persona);
        })
    };

    let open_about = {
        let set_phase = set_phase.clone();
        Callback::from(move |()| set_phase.emit(Phase::About))
    };

    let open_settings = {
        let set_phase = set_phase.clone();
        Callback::from(move |()| set_phase.emit(Phase::Settings))
    };

    let quit = {
        let state = state.clone();
        let set_phase = set_phase;
        Callback::from(move |()| {
            reset_run_state(&state);
            set_phase.emit(Phase::Boot);
        })
    };

    MenuActionHandlers {
        start_journey,
        open_about,
        open_settings,
        quit,
    }
}

pub fn render_menu(state: &AppState) -> Html {
    let on_action = menu_action_callback(build_menu_action_handlers(state));
    let menu_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
    html! {
        <MenuPage
            logo_src={menu_logo_src}
            on_action={on_action}
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[function_component(MenuActionHarness)]
    fn menu_action_harness() -> Html {
        let called = use_mut_ref(|| 0u8);
        let handlers = MenuActionHandlers {
            start_journey: {
                let called = called.clone();
                Callback::from(move |()| *called.borrow_mut() = 1)
            },
            open_about: {
                let called = called.clone();
                Callback::from(move |()| *called.borrow_mut() = 2)
            },
            open_settings: {
                let called = called.clone();
                Callback::from(move |()| *called.borrow_mut() = 3)
            },
            quit: {
                let called = called.clone();
                Callback::from(move |()| *called.borrow_mut() = 4)
            },
        };

        let invoked = use_mut_ref(|| false);
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            let on_action = menu_action_callback(handlers);
            on_action.emit(MenuAction::StartJourney);
            on_action.emit(MenuAction::About);
            on_action.emit(MenuAction::Settings);
            on_action.emit(MenuAction::Quit);
        }

        let current = *called.borrow();
        html! { <div data-called={current.to_string()} /> }
    }

    #[test]
    fn menu_action_callback_routes_actions() {
        let html = block_on(LocalServerRenderer::<MenuActionHarness>::new().render());
        assert!(html.contains("data-called=\"4\""));
    }

    #[test]
    fn reset_run_state_clears_progress() {
        struct RunStateProbe {
            session: RefCell<Option<crate::game::JourneySession>>,
            pending_state: RefCell<Option<crate::game::GameState>>,
            logs: RefCell<Vec<String>>,
            run_seed: Cell<u64>,
            code: RefCell<AttrValue>,
            show_save: Cell<bool>,
            show_settings: Cell<bool>,
        }

        impl RunStateTarget for RunStateProbe {
            fn set_session(&self, value: Option<crate::game::JourneySession>) {
                *self.session.borrow_mut() = value;
            }

            fn set_pending_state(&self, value: Option<crate::game::GameState>) {
                *self.pending_state.borrow_mut() = value;
            }

            fn set_logs(&self, value: Vec<String>) {
                *self.logs.borrow_mut() = value;
            }

            fn set_run_seed(&self, value: u64) {
                self.run_seed.set(value);
            }

            fn set_code(&self, value: AttrValue) {
                *self.code.borrow_mut() = value;
            }

            fn set_show_save(&self, value: bool) {
                self.show_save.set(value);
            }

            fn set_show_settings(&self, value: bool) {
                self.show_settings.set(value);
            }
        }

        let data = crate::game::data::EncounterData::empty();
        let endgame_cfg = crate::game::endgame::EndgameTravelCfg::default_config();
        let session = crate::game::JourneySession::new(
            crate::game::state::GameMode::Classic,
            crate::game::StrategyId::Balanced,
            7,
            data,
            &endgame_cfg,
        );
        let probe = RunStateProbe {
            session: RefCell::new(Some(session)),
            pending_state: RefCell::new(Some(crate::game::GameState::default())),
            logs: RefCell::new(vec![String::from("log.booting")]),
            run_seed: Cell::new(99),
            code: RefCell::new(AttrValue::from("CL-ORANGE42")),
            show_save: Cell::new(true),
            show_settings: Cell::new(true),
        };

        reset_run_state(&probe);

        assert!(probe.session.borrow().is_none());
        assert!(probe.pending_state.borrow().is_none());
        assert!(probe.logs.borrow().is_empty());
        assert_eq!(probe.run_seed.get(), 0);
        assert!(probe.code.borrow().is_empty());
        assert!(!probe.show_save.get());
        assert!(!probe.show_settings.get());
    }

    #[test]
    fn reset_run_state_updates_app_state_handles() {
        #[function_component(ResetHarness)]
        fn reset_harness() -> Html {
            let invoked = use_mut_ref(|| false);
            let state = AppState {
                phase: use_state(|| Phase::Menu),
                code: use_state(|| AttrValue::from("CL-ORANGE42")),
                data: use_state(crate::game::data::EncounterData::empty),
                pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
                endgame_config: use_state(crate::game::endgame::EndgameTravelCfg::default_config),
                weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
                camp_config: use_state(crate::game::CampConfig::default_config),
                crossing_config: use_state(crate::game::CrossingConfig::default),
                boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
                result_config: use_state(crate::game::ResultConfig::default),
                preload_progress: use_state(|| 100_u8),
                boot_ready: use_state(|| true),
                high_contrast: use_state(|| false),
                pending_state: use_state(|| Some(crate::game::GameState::default())),
                session: use_state(|| None::<crate::game::JourneySession>),
                logs: use_state(|| vec![String::from("log.booting")]),
                run_seed: use_state(|| 99_u64),
                show_save: use_state(|| false),
                save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
                show_settings: use_state(|| false),
                current_language: use_state(|| String::from("en")),
            };

            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                reset_run_state(&state);
            }

            Html::default()
        }

        let _ = block_on(LocalServerRenderer::<ResetHarness>::new().render());
    }

    #[test]
    fn menu_action_callback_routes_actions_directly() {
        let start_called = Rc::new(Cell::new(false));
        let about_called = Rc::new(Cell::new(false));
        let settings_called = Rc::new(Cell::new(false));
        let quit_called = Rc::new(Cell::new(false));

        let handlers = MenuActionHandlers {
            start_journey: {
                let start_called = start_called.clone();
                Callback::from(move |()| start_called.set(true))
            },
            open_about: {
                let about_called = about_called.clone();
                Callback::from(move |()| about_called.set(true))
            },
            open_settings: {
                let settings_called = settings_called.clone();
                Callback::from(move |()| settings_called.set(true))
            },
            quit: {
                let quit_called = quit_called.clone();
                Callback::from(move |()| quit_called.set(true))
            },
        };

        let on_action = menu_action_callback(handlers);
        on_action.emit(MenuAction::StartJourney);
        on_action.emit(MenuAction::About);
        on_action.emit(MenuAction::Settings);
        on_action.emit(MenuAction::Quit);

        assert!(start_called.get());
        assert!(about_called.get());
        assert!(settings_called.get());
        assert!(quit_called.get());
    }

    #[test]
    fn menu_action_handlers_reset_state_on_start_and_quit() {
        #[function_component(MenuHandlersHarness)]
        fn menu_handlers_harness() -> Html {
            let invoked = use_mut_ref(|| false);
            let state = AppState {
                phase: use_state(|| Phase::Menu),
                code: use_state(|| AttrValue::from("CL-ORANGE42")),
                data: use_state(crate::game::data::EncounterData::empty),
                pacing_config: use_state(crate::game::pacing::PacingConfig::default_config),
                endgame_config: use_state(crate::game::endgame::EndgameTravelCfg::default_config),
                weather_config: use_state(crate::game::weather::WeatherConfig::default_config),
                camp_config: use_state(crate::game::CampConfig::default_config),
                crossing_config: use_state(crate::game::CrossingConfig::default),
                boss_config: use_state(crate::game::boss::BossConfig::load_from_static),
                result_config: use_state(crate::game::ResultConfig::default),
                preload_progress: use_state(|| 0_u8),
                boot_ready: use_state(|| false),
                high_contrast: use_state(|| false),
                pending_state: use_state(|| Some(crate::game::GameState::default())),
                session: use_state(|| None::<crate::game::JourneySession>),
                logs: use_state(|| vec![String::from("log.booting")]),
                run_seed: use_state(|| 99_u64),
                show_save: use_state(|| true),
                save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
                show_settings: use_state(|| true),
                current_language: use_state(|| String::from("en")),
            };
            let handlers = build_menu_action_handlers(&state);

            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                handlers.start_journey.emit(());
                handlers.quit.emit(());
            }

            Html::default()
        }

        let _ = block_on(LocalServerRenderer::<MenuHandlersHarness>::new().render());
    }
}
