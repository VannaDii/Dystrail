use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::pages::persona::PersonaPage;
use yew::prelude::*;

pub fn render_persona(state: &AppState) -> Html {
    let on_selected = {
        let pending = state.pending_state.clone();
        Callback::from(move |per: crate::game::personas::Persona| {
            let mut gs = (*pending).clone().unwrap_or_default();
            gs.apply_persona(&per);
            pending.set(Some(gs));
        })
    };
    let on_continue = {
        let phase = state.phase.clone();
        Callback::from(move |()| phase.set(Phase::Outfitting))
    };
    html! { <PersonaPage {on_selected} {on_continue} /> }
}
