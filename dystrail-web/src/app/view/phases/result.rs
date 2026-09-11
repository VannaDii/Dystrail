use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::game::encode_friendly;
use crate::game::state::GameState;
use crate::pages::result::ResultPage;
use yew::prelude::*;

pub fn render_result(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let result_state = sess.state().clone();
        let result_config_data = (*state.result_config).clone();
        let boss_won = result_state.boss.outcome.victory;

        let session_for_replay = state.session.clone();
        let pending_state_for_replay = state.pending_state.clone();
        let seed_for_replay = *state.run_seed;
        let replay_phase = state.phase.clone();
        let replay_code = state.code.clone();
        let replay_deep = result_state.mode.is_deep();
        let on_replay_seed = Callback::from(move |()| {
            let new_game = GameState {
                seed: seed_for_replay,
                ..GameState::default()
            };
            pending_state_for_replay.set(Some(new_game));
            session_for_replay.set(None);
            replay_code.set(encode_friendly(replay_deep, seed_for_replay).into());
            replay_phase.set(Phase::Persona);
        });

        let session_for_new_run = state.session.clone();
        let pending_state_for_new_run = state.pending_state.clone();
        let new_phase = state.phase.clone();
        let new_code = state.code.clone();
        let deep = result_state.mode.is_deep();
        let on_new_run = Callback::from(move |()| {
            pending_state_for_new_run.set(Some(GameState::default()));
            session_for_new_run.set(None);
            new_code.set(
                crate::game::seed::generate_code_from_entropy(deep, js_sys::Date::now().to_bits())
                    .into(),
            );
            new_phase.set(Phase::Boot);
        });

        let session_for_title = state.session.clone();
        let pending_state_for_title = state.pending_state.clone();
        let phase_for_title = state.phase.clone();
        let on_title = Callback::from(move |()| {
            pending_state_for_title.set(None);
            session_for_title.set(None);
            phase_for_title.set(Phase::Boot);
        });

        let on_export = crate::app::view::handlers::build_export_state(state);

        html! {
            <ResultPage
                state={result_state}
                result_config={result_config_data}
                boss_won={boss_won}
                on_replay_seed={on_replay_seed}
                on_new_run={on_new_run}
                on_title={on_title}
                on_export={on_export}
            />
        }
    })
}
