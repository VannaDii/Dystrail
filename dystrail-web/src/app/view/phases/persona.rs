use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::persona::PersonaPage;
use yew::prelude::*;

fn build_persona_selected(
    pending: UseStateHandle<Option<crate::game::state::GameState>>,
) -> Callback<crate::game::personas::Persona> {
    Callback::from(move |per: crate::game::personas::Persona| {
        let mut gs = (*pending).clone().unwrap_or_default();
        gs.apply_persona(&per);
        pending.set(Some(gs));
    })
}

pub fn render_persona(state: &AppState) -> Html {
    let on_selected = build_persona_selected(state.pending_state.clone());
    let on_continue = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::ModeSelect))
    };
    html! { <PersonaPage {on_selected} {on_continue} /> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[function_component(PersonaSelectHarness)]
    fn persona_select_harness() -> Html {
        let pending = use_state(|| None::<crate::game::state::GameState>);
        let invoked = use_mut_ref(|| false);
        let called = Rc::new(Cell::new(false));
        let called_ref = called.clone();
        let on_selected = build_persona_selected(pending);
        let wrapper = Callback::from(move |()| {
            called_ref.set(true);
            let persona = crate::game::personas::Persona {
                id: "test".to_string(),
                name: "Test".to_string(),
                desc: "Test persona".to_string(),
                score_mult: 1.0,
                start: crate::game::personas::PersonaStart::default(),
                mods: crate::game::personas::PersonaMods::default(),
            };
            on_selected.emit(persona);
        });
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            wrapper.emit(());
        }
        html! { <div data-called={called.get().to_string()} /> }
    }

    #[test]
    fn build_persona_selected_executes() {
        let html = block_on(LocalServerRenderer::<PersonaSelectHarness>::new().render());
        assert!(html.contains("data-called=\"true\""));
    }
}
