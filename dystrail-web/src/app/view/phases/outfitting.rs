use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::outfitting::OutfittingPage;
use yew::prelude::*;

fn build_outfitting_continue(
    pending_handle: UseStateHandle<Option<crate::game::GameState>>,
    session_handle: UseStateHandle<Option<crate::game::JourneySession>>,
    logs_handle: UseStateHandle<Vec<String>>,
    run_seed_handle: UseStateHandle<u64>,
    phase_handle: UseStateHandle<Phase>,
    endgame_cfg: crate::game::EndgameTravelCfg,
) -> Callback<(
    crate::game::GameState,
    crate::game::store::Grants,
    Vec<String>,
)> {
    Callback::from(
        move |(new_state, _grants, _tags): (
            crate::game::GameState,
            crate::game::store::Grants,
            Vec<String>,
        )| {
            let mode_label = if new_state.mode.is_deep() {
                crate::i18n::t("mode.deep")
            } else {
                crate::i18n::t("mode.classic")
            };
            let mut vars = std::collections::BTreeMap::new();
            vars.insert("mode", mode_label.as_str());
            let logs = vec![crate::i18n::tr("log.run_begins", Some(&vars))];

            let seed = new_state.seed;
            let session = crate::app::phase::session_from_state(new_state.clone(), &endgame_cfg);

            logs_handle.set(logs);
            run_seed_handle.set(seed);
            pending_handle.set(Some(new_state));
            session_handle.set(Some(session));
            phase_handle.set(Phase::Travel);
        },
    )
}

pub fn render_outfitting(state: &AppState) -> Html {
    let current_state = (*state.pending_state).clone().unwrap_or_default();
    let on_continue = build_outfitting_continue(
        state.pending_state.clone(),
        state.session.clone(),
        state.logs.clone(),
        state.run_seed.clone(),
        state.phase.clone(),
        (*state.endgame_config).clone(),
    );
    let on_back = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::ModeSelect))
    };
    html! {
        <OutfittingPage game_state={current_state} {on_continue} {on_back} />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[function_component(OutfittingContinueHarness)]
    fn outfitting_continue_harness() -> Html {
        crate::i18n::set_lang("en");
        let pending_handle = use_state(|| None::<crate::game::GameState>);
        let session_handle = use_state(|| None::<crate::game::JourneySession>);
        let logs_handle = use_state(Vec::<String>::new);
        let run_seed_handle = use_state(|| 0_u64);
        let phase_handle = use_state(|| Phase::Outfitting);
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let on_continue = build_outfitting_continue(
            pending_handle,
            session_handle,
            logs_handle,
            run_seed_handle,
            phase_handle,
            crate::game::EndgameTravelCfg::default_config(),
        );
        let wrapper = Callback::from(move |()| {
            called_ref.set(true);
            let classic_state = crate::game::GameState::default();
            let data = crate::game::data::EncounterData::empty();
            let deep_state = crate::game::GameState::default().with_seed(
                11,
                crate::game::state::GameMode::Deep,
                data,
            );
            on_continue.emit((
                classic_state,
                crate::game::store::Grants::default(),
                Vec::new(),
            ));
            on_continue.emit((
                deep_state,
                crate::game::store::Grants::default(),
                Vec::new(),
            ));
        });
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            wrapper.emit(());
        }
        html! { <div data-called={called.get().to_string()} /> }
    }

    #[test]
    fn build_outfitting_continue_executes() {
        let html = block_on(LocalServerRenderer::<OutfittingContinueHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
