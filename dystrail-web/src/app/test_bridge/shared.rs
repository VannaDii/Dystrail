use crate::app::phase::session_from_state;
use crate::app::state::AppState;
#[cfg(test)]
use crate::game::JourneySession;
use crate::game::data::EncounterData;
use crate::game::state::{GameMode, GameState};
use yew::AttrValue;

#[cfg(test)]
pub(super) fn apply_session_update(
    session: &mut Option<JourneySession>,
    pending_state: &mut Option<GameState>,
    updater: impl FnOnce(&mut GameState),
) -> bool {
    if let Some(session) = session.as_mut() {
        session.with_state_mut(|gs| updater(gs));
        *pending_state = Some(session.state().clone());
        return true;
    }
    false
}

#[cfg(test)]
pub(super) fn update_session_state(state: &AppState, updater: impl FnOnce(&mut GameState)) {
    let mut session = (*state.session).clone();
    let mut pending_state = (*state.pending_state).clone();
    if apply_session_update(&mut session, &mut pending_state, updater) {
        state.pending_state.set(pending_state);
        state.session.set(session);
    }
}

pub(super) fn seed_session_with_data(
    state: &AppState,
    seed: u64,
    data: EncounterData,
    updater: impl FnOnce(&mut GameState),
) {
    let mut gs = GameState::default().with_seed(seed, GameMode::Classic, data);
    updater(&mut gs);
    let endgame_cfg = (*state.endgame_config).clone();
    let session = session_from_state(gs.clone(), &endgame_cfg);
    state.pending_state.set(Some(gs));
    state.session.set(Some(session));
    state.run_seed.set(seed);
    let code = AttrValue::from(crate::game::encode_friendly(false, seed));
    state.code.set(code);
    state.preload_progress.set(100);
    state.boot_ready.set(true);
}

#[cfg(test)]
mod tests {
    use super::{apply_session_update, seed_session_with_data, update_session_state};
    use crate::app::phase::{Phase, session_from_state};
    use crate::app::state::AppState;
    use crate::game::boss::BossConfig;
    use crate::game::data::EncounterData;
    use crate::game::endgame::EndgameTravelCfg;
    use crate::game::pacing::PacingConfig;
    use crate::game::state::{GameMode, GameState};
    use crate::game::weather::WeatherConfig;
    use crate::game::{CampConfig, CrossingConfig, ResultConfig};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;
    use yew::prelude::*;

    #[test]
    fn seed_session_with_data_updates_app_state() {
        #[function_component(SeedSessionHarness)]
        fn seed_session_harness() -> Html {
            let invoked = use_mut_ref(|| false);
            let state = AppState {
                phase: use_state(|| Phase::Menu),
                code: use_state(|| AttrValue::from("")),
                data: use_state(EncounterData::empty),
                pacing_config: use_state(PacingConfig::default_config),
                endgame_config: use_state(EndgameTravelCfg::default_config),
                weather_config: use_state(WeatherConfig::default_config),
                camp_config: use_state(CampConfig::default_config),
                crossing_config: use_state(CrossingConfig::default),
                boss_config: use_state(BossConfig::load_from_static),
                result_config: use_state(ResultConfig::default),
                preload_progress: use_state(|| 0_u8),
                boot_ready: use_state(|| false),
                high_contrast: use_state(|| false),
                pending_state: use_state(|| None::<GameState>),
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
                seed_session_with_data(&state, 4242, EncounterData::load_from_static(), |gs| {
                    gs.day = 3;
                });
            }

            let called = invoked.borrow().to_string();
            html! { <div data-called={called} /> }
        }

        let html = block_on(LocalServerRenderer::<SeedSessionHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }

    #[test]
    fn update_session_state_snapshots_pending_state() {
        let data = EncounterData::load_from_static();
        let gs = GameState::default().with_seed(7, GameMode::Classic, data);
        let endgame_cfg = EndgameTravelCfg::default_config();
        let session = session_from_state(gs, &endgame_cfg);
        let mut session = Some(session);
        let mut pending_state = None;
        let updated = apply_session_update(&mut session, &mut pending_state, |gs| {
            gs.day = gs.day.saturating_add(1);
        });
        assert!(updated);
        assert!(pending_state.is_some());
    }

    #[test]
    fn update_session_state_noops_without_session() {
        let mut session = None;
        let mut pending_state = None;
        let updated = apply_session_update(&mut session, &mut pending_state, |gs| {
            gs.day = gs.day.saturating_add(1);
        });
        assert!(!updated);
        assert!(pending_state.is_none());
    }

    #[test]
    fn update_session_state_updates_handles() {
        #[function_component(UpdateSessionHarness)]
        fn update_session_harness() -> Html {
            let invoked = use_mut_ref(|| false);
            let state = AppState {
                phase: use_state(|| Phase::Travel),
                code: use_state(|| AttrValue::from("CL-ORANGE42")),
                data: use_state(EncounterData::empty),
                pacing_config: use_state(PacingConfig::default_config),
                endgame_config: use_state(EndgameTravelCfg::default_config),
                weather_config: use_state(WeatherConfig::default_config),
                camp_config: use_state(CampConfig::default_config),
                crossing_config: use_state(CrossingConfig::default),
                boss_config: use_state(BossConfig::load_from_static),
                result_config: use_state(ResultConfig::default),
                preload_progress: use_state(|| 0_u8),
                boot_ready: use_state(|| false),
                high_contrast: use_state(|| false),
                pending_state: use_state(|| None::<GameState>),
                session: use_state(|| {
                    let data = EncounterData::load_from_static();
                    let gs = GameState::default().with_seed(7, GameMode::Classic, data);
                    let endgame_cfg = EndgameTravelCfg::default_config();
                    Some(session_from_state(gs, &endgame_cfg))
                }),
                logs: use_state(Vec::<String>::new),
                run_seed: use_state(|| 7_u64),
                show_save: use_state(|| false),
                save_focus_target: use_state(|| AttrValue::from("save-open-btn")),
                show_settings: use_state(|| false),
                current_language: use_state(|| String::from("en")),
            };

            if !*invoked.borrow() {
                *invoked.borrow_mut() = true;
                update_session_state(&state, |gs| {
                    gs.day = gs.day.saturating_add(1);
                });
            }

            Html::default()
        }

        let _ = block_on(LocalServerRenderer::<UpdateSessionHarness>::new().render());
    }
}
