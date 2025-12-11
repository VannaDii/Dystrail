use super::layout::{
    focus_effect, keydown_handler, menu_items, render_menu, resolve_selection, status_message,
};
use super::logic::VehicleAction;
use crate::game::GameState;
use crate::i18n;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct VehicleStatusProps {
    pub game_state: Option<Rc<GameState>>,
    pub on_back: Callback<()>,
    pub on_repair_action: Callback<VehicleAction>,
}

impl PartialEq for VehicleStatusProps {
    fn eq(&self, other: &Self) -> bool {
        let self_breakdown = self
            .game_state
            .as_ref()
            .and_then(|gs| gs.breakdown.as_ref().map(|b| (b.part, b.day_started)));
        let other_breakdown = other
            .game_state
            .as_ref()
            .and_then(|gs| gs.breakdown.as_ref().map(|b| (b.part, b.day_started)));

        let self_spares = self.game_state.as_ref().map(|gs| {
            (
                gs.inventory.spares.tire,
                gs.inventory.spares.battery,
                gs.inventory.spares.alt,
                gs.inventory.spares.pump,
            )
        });
        let other_spares = other.game_state.as_ref().map(|gs| {
            (
                gs.inventory.spares.tire,
                gs.inventory.spares.battery,
                gs.inventory.spares.alt,
                gs.inventory.spares.pump,
            )
        });

        self_breakdown == other_breakdown && self_spares == other_spares
    }
}

#[function_component(VehicleStatus)]
pub fn vehicle_status(p: &VehicleStatusProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();

    let gs = p.game_state.as_ref();
    let breakdown = gs.and_then(|state| state.breakdown.as_ref());
    let spare_counts = gs.map(|state| {
        let spares = &state.inventory.spares;
        (spares.tire, spares.battery, spares.alt, spares.pump)
    });

    let activate = resolve_selection(
        p.on_back.clone(),
        p.on_repair_action.clone(),
        breakdown.map(|b| b.part),
        spare_counts,
    );

    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), focus_idx.clone());

    let items = menu_items(breakdown.map(|b| b.part), spare_counts);
    let setsize = u8::try_from(items.len()).unwrap_or(u8::MAX);
    let status_msg = status_message(breakdown.map(|b| b.part));

    html! {
      <section role="region" aria-labelledby="vehicle-title" onkeydown={on_keydown} class="ot-menu">
        <h2 id="vehicle-title">{ i18n::t("vehicle.title") }</h2>

        <div class="status-display" role="status" aria-live="polite">
            <p>{ status_msg }</p>
        </div>

        <ul role="menu" aria-label={i18n::t("vehicle.title")} id="vehicle-menu" ref={list_ref}>
          { render_menu(&items, *focus_idx, setsize, &activate) }
        </ul>

        <p class="muted">{ i18n::t("vehicle.help") }</p>
      </section>
    }
}
