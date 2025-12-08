use super::super::handlers::{announce_quantity_change, can_add_item, format_currency};
use super::super::state::StoreState;
use crate::game::{
    GameState,
    store::{StoreItem, calculate_cart_total, calculate_effective_price},
};
use crate::i18n;
use yew::prelude::*;

pub fn render_store_item_card(
    idx: u8,
    item: &StoreItem,
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
) -> Html {
    let name = i18n::t(&format!("store.items.{}.name", item.id));
    let desc = i18n::t(&format!("store.items.{}.desc", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(effective_price);
    let qty_in_cart = state.cart.get_quantity(&item.id);
    let can_add = can_add_item(
        &state.cart,
        item,
        1,
        game_state.budget_cents,
        state.discount_pct,
    );
    let initials = name
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());

    let on_add = {
        let state = state.clone();
        let item_clone = item.clone();
        let budget = game_state.budget_cents;
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            if can_add_item(
                &new_state.cart,
                &item_clone,
                1,
                budget,
                new_state.discount_pct,
            ) {
                new_state.cart.add_item(&item_clone.id, 1);
                announce_quantity_change(&item_clone, 1, true, &new_state, budget);
                new_state.cart.total_cents = calculate_cart_total(
                    &new_state.cart,
                    &new_state.store_data,
                    new_state.discount_pct,
                );
                new_state.focus_idx = idx;
                state.set(new_state);
            }
        })
    };

    let on_remove = {
        let state = state.clone();
        let item_clone = item.clone();
        let budget = game_state.budget_cents;
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            if new_state.cart.get_quantity(&item_clone.id) > 0 {
                new_state.cart.remove_item(&item_clone.id, 1);
                announce_quantity_change(&item_clone, 1, false, &new_state, budget);
                new_state.cart.total_cents = calculate_cart_total(
                    &new_state.cart,
                    &new_state.store_data,
                    new_state.discount_pct,
                );
                new_state.focus_idx = idx;
                state.set(new_state);
            }
        })
    };

    html! {
        <article
            role="group"
            aria-labelledby={format!("store-item-{idx}")}
            class="store-card"
            data-key={idx.to_string()}
            title={desc.clone()}>
            <div class="store-card-icon" aria-hidden="true">
                <span>{ initials }</span>
            </div>
            <div class="store-card-body">
                <div class="store-card-head">
                    <h2 id={format!("store-item-{idx}")}>{ name }</h2>
                    <span class="store-price">{ price_str }</span>
                </div>
                <p class="muted">{ desc }</p>
                <div class="store-qty-row">
                    <button class="store-qty-btn" onclick={on_remove} aria-label={i18n::t("store.qty_prompt.rem1")} disabled={qty_in_cart == 0}>{"–"}</button>
                    <span class="store-qty" aria-live="polite">{ qty_in_cart }</span>
                    <button class="store-qty-btn" onclick={on_add} aria-label={i18n::t("store.qty_prompt.add1")} disabled={!can_add}>{"+"}</button>
                </div>
            </div>
        </article>
    }
}
