use super::super::{
    handlers::{announce_quantity_change, maximum_quantity},
    state::StoreState,
};
use crate::{
    game::store::{StoreItem, calculate_cart_total},
    i18n,
};
use yew::prelude::*;

pub fn render(item: &StoreItem, state: &UseStateHandle<StoreState>, budget: i64) -> Html {
    let qty = state.cart.get_quantity(&item.id);
    let max = maximum_quantity(&state.cart, item, budget, state.discount_pct);
    let change = {
        let state = state.clone();
        let item = item.clone();
        Callback::from(move |requested: i32| {
            let mut next = (*state).clone();
            let current = next.cart.get_quantity(&item.id);
            let quantity = requested.clamp(
                0,
                maximum_quantity(&next.cart, &item, budget, next.discount_pct),
            );
            let difference = quantity - current;
            if difference == 0 {
                return;
            }
            if difference > 0 {
                next.cart.add_item(&item.id, difference);
            } else {
                next.cart.remove_item(&item.id, -difference);
            }
            next.cart.total_cents =
                calculate_cart_total(&next.cart, &next.store_data, next.discount_pct);
            announce_quantity_change(&item, difference.abs(), difference > 0, &next, budget);
            state.set(next);
        })
    };
    let on_input = {
        let change = change.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlInputElement>() {
                let quantity = input
                    .value()
                    .parse::<i32>()
                    .unwrap_or_default()
                    .clamp(0, max);
                input.set_value(&quantity.to_string());
                change.emit(quantity);
            }
        })
    };
    let on_remove = {
        let change = change.clone();
        Callback::from(move |_| change.emit(qty - 1))
    };
    let on_add = Callback::from(move |_| change.emit(qty + 1));
    html! {<div class="store-qty-row">
        <button class="store-qty-btn" onclick={on_remove} aria-label={i18n::t("store.qty_prompt.rem1")} disabled={qty == 0}>{"−"}</button>
        <input class="store-qty" type="number" inputmode="numeric" min="0" max={max.to_string()} step="1" value={qty.to_string()} oninput={on_input}
            aria-labelledby={format!("store-item-{} store-quantity-label",item.id)} />
        <button class="store-qty-btn" onclick={on_add} aria-label={i18n::t("store.qty_prompt.add1")} disabled={qty >= max}>{"+"}</button>
    </div>}
}
