use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::encode_friendly;
#[cfg(any(test, target_arch = "wasm32"))]
use crate::game::state::GameState;
use crate::pages::result::ResultPage;
use yew::prelude::*;

pub fn render_result(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let result_state = sess.state().clone();
        let result_config_data = (*state.result_config).clone();
        let boss_won = result_state.boss.outcome.victory;

        let seed_for_replay = *state.run_seed;
        let replay_state = state.clone();
        let on_replay_seed = {
            #[cfg(target_arch = "wasm32")]
            {
                Callback::from(move |()| {
                    apply_replay_seed(seed_for_replay, &replay_state);
                })
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (seed_for_replay, replay_state);
                Callback::from(|()| {})
            }
        };

        let session_for_new_run = state.session.clone();
        let pending_state_for_new_run = state.pending_state.clone();
        let phase_for_new_run = state.phase.clone();
        let on_new_run = Callback::from(move |()| { pending_state_for_new_run.set(None); session_for_new_run.set(None); phase_for_new_run.set(Phase::Menu); });

        let session_for_title = state.session.clone();
        let pending_state_for_title = state.pending_state.clone();
        let phase_for_title = state.phase.clone();
        let on_title = Callback::from(move |()| { pending_state_for_title.set(None); session_for_title.set(None); phase_for_title.set(Phase::Menu); });

        let on_export = {
            let seed = *state.run_seed;
            let is_deep = result_state.mode.is_deep();
            Callback::from(move |()| { let code_str = encode_friendly(is_deep, seed); if let Some(win) = web_sys::window() { let nav = win.navigator(); let cb = nav.clipboard(); let _ = cb.write_text(&code_str); } })
        };

        html! { <ResultPage state={result_state} result_config={result_config_data} boss_won={boss_won} on_replay_seed={on_replay_seed} on_new_run={on_new_run} on_title={on_title} on_export={on_export} /> }
    })
}

#[cfg(any(test, target_arch = "wasm32"))]
trait ReplaySeedTarget {
    fn set_pending_state(&self, value: Option<GameState>);
    fn set_session(&self, value: Option<crate::game::JourneySession>);
    fn set_phase(&self, value: Phase);
}

#[cfg(any(test, target_arch = "wasm32"))]
impl ReplaySeedTarget for AppState {
    fn set_pending_state(&self, value: Option<GameState>) {
        self.pending_state.set(value);
    }

    fn set_session(&self, value: Option<crate::game::JourneySession>) {
        self.session.set(value);
    }

    fn set_phase(&self, value: Phase) {
        self.phase.set(value);
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn apply_replay_seed<T: ReplaySeedTarget>(seed: u64, target: &T) {
    let new_game = GameState {
        seed,
        ..GameState::default()
    };
    target.set_pending_state(Some(new_game));
    target.set_session(None);
    target.set_phase(Phase::Menu);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    struct ReplayProbe {
        pending_state: RefCell<Option<GameState>>,
        session: RefCell<Option<crate::game::JourneySession>>,
        phase: Cell<Phase>,
    }

    impl ReplaySeedTarget for ReplayProbe {
        fn set_pending_state(&self, value: Option<GameState>) {
            *self.pending_state.borrow_mut() = value;
        }

        fn set_session(&self, value: Option<crate::game::JourneySession>) {
            *self.session.borrow_mut() = value;
        }

        fn set_phase(&self, value: Phase) {
            self.phase.set(value);
        }
    }

    #[test]
    fn apply_replay_seed_resets_state() {
        let data = crate::game::data::EncounterData::empty();
        let endgame_cfg = crate::game::endgame::EndgameTravelCfg::default_config();
        let session = crate::game::JourneySession::new(
            crate::game::state::GameMode::Classic,
            crate::game::StrategyId::Balanced,
            7,
            data,
            &endgame_cfg,
        );
        let probe = ReplayProbe {
            pending_state: RefCell::new(None),
            session: RefCell::new(Some(session)),
            phase: Cell::new(Phase::Result),
        };
        apply_replay_seed(42, &probe);
        assert_eq!(
            probe
                .pending_state
                .borrow()
                .as_ref()
                .map(|state| state.seed),
            Some(42)
        );
        assert!(probe.session.borrow().is_none());
        assert_eq!(probe.phase.get(), Phase::Menu);
    }

    #[test]
    fn apply_replay_seed_updates_app_state_handles() {
        #[function_component(ReplaySeedHarness)]
        fn replay_seed_harness() -> Html {
            let invoked = use_mut_ref(|| false);
            let state = AppState {
                phase: use_state(|| Phase::Result),
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
                pending_state: use_state(|| None::<crate::game::GameState>),
                session: use_state(|| None::<crate::game::JourneySession>),
                logs: use_state(Vec::<String>::new),
                run_seed: use_state(|| 0_u64),
                show_save: use_state(|| false),
                save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
                show_settings: use_state(|| false),
                current_language: use_state(|| String::from("en")),
            };

            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                apply_replay_seed(7, &state);
            }

            Html::default()
        }

        use futures::executor::block_on;
        use yew::LocalServerRenderer;
        let _ = block_on(LocalServerRenderer::<ReplaySeedHarness>::new().render());
    }
}
