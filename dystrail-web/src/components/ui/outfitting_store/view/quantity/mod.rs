mod options;

use super::super::state::StoreState;
use options::{build_quantity_options, quantity_select_callback, render_quantity_options};
use std::collections::BTreeMap;
use yew::prelude::*;

pub fn render_quantity_screen(
    item_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &crate::game::GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
) -> Html {
    let Some(item) = state.store_data.find_item(item_id) else {
        return html! { <div>{ "Item not found" }</div> };
    };

    let item_name = crate::i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price =
        crate::game::store::calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = crate::i18n::fmt_currency(effective_price);

    let title = crate::i18n::tr(
        "store.qty_prompt.title",
        Some(&{
            let mut vars = BTreeMap::new();
            vars.insert("item", item_name.as_str());
            vars.insert("price", price_str.as_str());
            vars
        }),
    );

    let current_qty = state.cart.get_quantity(item_id);
    let options = build_quantity_options(item, state, game_state, effective_price, current_qty);
    let on_select = quantity_select_callback(item, game_state.budget_cents, state);

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="qty-title" onkeydown={on_keydown} tabindex="0" data-testid="outfitting-store">
                <h1 id="qty-title">{ title }</h1>
                { if current_qty > 0 {
                    html! { <p class="current-qty">{ format!("Current in cart: {current_qty}") }</p> }
                } else { html! {} }}
                <div ref={list_ref}>{ render_quantity_options(&options, state.focus_idx, &on_select) }</div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::state::{StoreScreen, load_store_data};
    use super::*;
    use crate::game::GameState;
    use crate::game::store::Cart;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[function_component(MissingItemHarness)]
    fn missing_item_harness() -> Html {
        let store_data = load_store_data().expect("store data should load");
        let state = StoreState {
            store_data,
            cart: Cart::new(),
            current_screen: StoreScreen::Home,
            focus_idx: 1,
            discount_pct: 0.0,
        };
        let state = use_state(|| state);
        let list_ref = NodeRef::default();
        let on_keydown: Callback<web_sys::KeyboardEvent> = Callback::noop();
        render_quantity_screen(
            "missing",
            &state,
            &GameState::default(),
            &list_ref,
            &on_keydown,
        )
    }

    #[test]
    fn render_quantity_screen_reports_missing_item() {
        let html = block_on(LocalServerRenderer::<MissingItemHarness>::new().render());
        assert!(html.contains("Item not found"));
    }

    #[function_component(QuantityRenderHarness)]
    fn quantity_render_harness() -> Html {
        let store_data = load_store_data().expect("store data should load");
        let item_id = store_data
            .categories
            .iter()
            .find_map(|cat| cat.items.first().map(|item| item.id.clone()))
            .or_else(|| store_data.items.first().map(|item| item.id.clone()))
            .expect("store data should include at least one item");
        let state = StoreState {
            store_data,
            cart: Cart::new(),
            current_screen: StoreScreen::QuantityPrompt(item_id.clone()),
            focus_idx: 1,
            discount_pct: 0.0,
        };
        let state = use_state(|| state);
        let list_ref = NodeRef::default();
        let on_keydown: Callback<web_sys::KeyboardEvent> = Callback::noop();
        render_quantity_screen(
            &item_id,
            &state,
            &GameState::default(),
            &list_ref,
            &on_keydown,
        )
    }

    #[test]
    fn render_quantity_screen_renders_options() {
        let html = block_on(LocalServerRenderer::<QuantityRenderHarness>::new().render());
        assert!(html.contains("outfitting-store"));
    }
}
