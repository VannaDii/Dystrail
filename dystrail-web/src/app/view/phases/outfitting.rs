use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::outfitting::OutfittingPage;
use yew::prelude::*;

fn build_outfitting_continue(
    pending_handle: UseStateHandle<Option<crate::game::GameState>>,
    phase_handle: UseStateHandle<Phase>,
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
            pending_handle.set(Some(new_state));
            phase_handle.set(Phase::Menu);
        },
    )
}

pub fn render_outfitting(state: &AppState) -> Html {
    let current_state = (*state.pending_state).clone().unwrap_or_default();
    let on_continue = build_outfitting_continue(state.pending_state.clone(), state.phase.clone());
    html! {
        <OutfittingPage game_state={current_state} {on_continue} />
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
        let pending_handle = use_state(|| None::<crate::game::GameState>);
        let phase_handle = use_state(|| Phase::Outfitting);
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let on_continue = build_outfitting_continue(pending_handle, phase_handle);
        let wrapper = Callback::from(move |()| {
            called_ref.set(true);
            on_continue.emit((
                crate::game::GameState::default(),
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
