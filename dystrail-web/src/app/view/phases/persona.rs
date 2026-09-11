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
            // New-run goods are purchased in the editable starter cart.
            gs.stats.supplies = 0;
            gs.inventory = crate::game::state::Inventory::default();
            pending.set(Some(gs));
        })
    };
    let on_continue = {
        let phase = state.phase.clone();
        let pending = state.pending_state.clone();
        Callback::from(move |()| {
            let mut gs = (*pending).clone().unwrap_or_default();
            gs.party.initialize(
                gs.persona_id.as_deref().unwrap_or("journalist"),
                js_sys::Date::now().to_bits(),
            );
            pending.set(Some(gs));
            phase.set(Phase::Crew);
        })
    };
    html! { <PersonaPage {on_selected} {on_continue} initial_id={state.pending_state.as_ref().and_then(|gs|gs.persona_id.clone())} /> }
}
