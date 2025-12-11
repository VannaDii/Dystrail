use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::outfitting::OutfittingPage;
use yew::prelude::*;

pub fn render_outfitting(state: &AppState) -> Html {
    let current_state = (*state.pending_state).clone().unwrap_or_default();
    let on_continue = {
        let pending_handle = state.pending_state.clone();
        let phase_handle = state.phase.clone();
        Callback::from(
            move |(new_state, _grants, _tags): (
                crate::game::GameState,
                crate::game::store::Grants,
                Vec<String>,
            )| {
                pending_handle.set(Some(new_state));
                phase_handle.set(Phase::Menu);
            },
        )
    };
    html! {
        <OutfittingPage game_state={current_state} {on_continue} />
    }
}
