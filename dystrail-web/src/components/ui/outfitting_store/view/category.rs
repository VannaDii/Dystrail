use super::super::handlers::format_currency;
use super::super::state::{StoreScreen, StoreState, set_screen};
use super::item_card::render_store_item_card;
use crate::i18n;
use yew::prelude::*;

pub fn render_category_screen(
    category_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &crate::game::GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
) -> Html {
    let Some(category) = state
        .store_data
        .categories
        .iter()
        .find(|c| c.id == *category_id)
    else {
        return html! { <div>{ "Category not found" }</div> };
    };

    let budget_str = format_currency(game_state.budget_cents - state.cart.total_cents);
    let category_name = i18n::t(&format!("store.categories.{category_id}"));
    let title = format!(
        "{category_name} — {budget_label}: {budget_str}",
        budget_label = i18n::t("store.budget")
    );

    let items = category.items.clone();
    let on_nav = {
        let state = state.clone();
        Callback::from(move |_| set_screen(&state, StoreScreen::Home))
    };

    let cart_cta = Callback::from({
        let state = state.clone();
        move |_| set_screen(&state, StoreScreen::Cart)
    });

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="category-title" onkeydown={on_keydown} class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="category-title">{ title }</h1>
                    </div>
                    <div class="store-budget">
                        <span class="label">{ i18n::t("store.budget") }</span>
                        <span class="value">{ budget_str }</span>
                    </div>
                </header>
                <div class="store-cart-summary" role="status" aria-live="polite">
                    <span>{ i18n::t("store.cart.title") }</span>
                    <span class="value">{ format_currency(state.cart.total_cents) }</span>
                </div>
                <div class="store-item-grid" ref={list_ref}>
                    { for items.iter().enumerate().map(|(i, item)| render_store_item_card(
                        u8::try_from(i + 1).unwrap_or_default(),
                        item,
                        state,
                        game_state
                    )) }
                </div>
                <div class="store-footer-row">
                    <button class="retro-btn-secondary" onclick={on_nav}>{ i18n::t("store.menu.back") }</button>
                    <button class="retro-btn-primary" onclick={cart_cta}>{ i18n::t("store.menu.view_cart") }</button>
                </div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}
