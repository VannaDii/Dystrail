use super::{
    handlers::handle_cart_selection,
    state::{OutfittingStoreProps, StoreState},
};
use crate::i18n;
use yew::prelude::*;

pub fn render(
    state: &UseStateHandle<StoreState>,
    p: &OutfittingStoreProps,
    review: bool,
    on_review: Callback<MouseEvent>,
) -> Html {
    let total = state.cart.total_cents;
    let remaining = p.game_state.budget_cents - total;
    let added: i32 = state
        .cart
        .lines
        .iter()
        .filter_map(|line| {
            state
                .store_data
                .find_item(&line.item_id)
                .map(|item| item.grants.supplies * line.qty)
        })
        .sum();
    let stocked = p.game_state.stats.supplies + added;
    let checkout = {
        let state = state.clone();
        let p = p.clone();
        Callback::from(move |_| handle_cart_selection(0, &state, &state, &p))
    };
    let clear = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut next = (*state).clone();
            next.cart.clear();
            state.set(next);
        })
    };
    html! {<footer class="loadout-panel" aria-label={i18n::t("store.cart.title")}>
        if stocked>20 {<p class="capacity-warning" role="status">{i18n::t("journey.capacity")}</p>}
        if remaining<0 {<p class="capacity-warning" role="alert">{i18n::t("store.alerts.over_budget")}</p>}
        <dl class="store-cart-summary">
            <div><dt>{i18n::t("play.packed_supplies")}</dt><dd>{format!("{stocked} / 20")}</dd></div>
            <div><dt>{i18n::t("play.spent")}</dt><dd class="cart-total">{i18n::fmt_currency(total)}</dd></div>
            <div><dt>{i18n::t("shopping.cash_after")}</dt><dd class="cart-cash">{i18n::fmt_currency(remaining)}</dd></div>
        </dl>
        <div class="store-checkout-actions">
            if review {<button class="retro-btn-secondary" onclick={on_review.clone()}>{i18n::t("play.edit")}</button>}
            else {<button class="clear-loadout" onclick={clear} disabled={state.cart.lines.is_empty()}>{i18n::t(if p.resupply {"shopping.clear"} else {"trail.clear_cart"})}</button>}
            if review || p.resupply {<button class="retro-btn-primary" onclick={checkout} disabled={remaining<0 || stocked>20 || (p.resupply && state.cart.lines.is_empty())}>{i18n::t(if p.resupply{"play2.buy_return"}else{"play.depart"})}</button>}
            else {<button class="retro-btn-primary" onclick={on_review}>{i18n::t("play.review")}</button>}
        </div>
        <div id="store-status" role="status" class="sr-only"></div>
    </footer>}
}
