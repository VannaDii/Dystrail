use super::super::handlers::{format_currency, handle_back_navigation, handle_cart_selection};
use super::super::state::{OutfittingStoreProps, StoreState};
use crate::i18n;
use yew::prelude::*;

pub fn render_cart_screen(
    state: &UseStateHandle<StoreState>,
    game_state: &crate::game::GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
    props: &OutfittingStoreProps,
) -> Html {
    let total_str = format_currency(state.cart.total_cents);
    let remaining = game_state.budget_cents - state.cart.total_cents;
    let remaining_str = format_currency(remaining);

    let cart_total_line = i18n::tr(
        "store.cart.total",
        Some(&{
            let mut vars = std::collections::BTreeMap::new();
            vars.insert("sum", total_str.as_str());
            vars.insert("left", remaining_str.as_str());
            vars
        }),
    );

    let can_checkout = remaining >= 0;
    let mut cart_lines = Vec::new();

    let on_checkout = {
        let st = state.clone();
        let props = props.clone();
        Callback::from(move |_| {
            handle_cart_selection(0, &st, &st, &props);
        })
    };

    let on_back = {
        let st = state.clone();
        Callback::from(move |_| handle_back_navigation(&st, &st))
    };

    for (i, line) in state.cart.lines.iter().enumerate() {
        if let Some(item) = state.store_data.find_item(&line.item_id) {
            let idx = u8::try_from(i + 1).unwrap_or(u8::MAX);
            let item_name = i18n::t(&format!("store.items.{}.name", item.id));
            let effective_price =
                crate::game::store::calculate_effective_price(item.price_cents, state.discount_pct);
            let line_total = effective_price * i64::from(line.qty);
            let line_total_str = format_currency(line_total);

            let qty_str = line.qty.to_string();
            let label = i18n::tr(
                "store.cart.line",
                Some(&{
                    let mut vars = std::collections::BTreeMap::new();
                    vars.insert("item", item_name.as_str());
                    vars.insert("qty", qty_str.as_str());
                    vars.insert("line_total", line_total_str.as_str());
                    vars
                }),
            );

            cart_lines.push((idx, label));
        }
    }

    cart_lines.push((0u8, i18n::t("store.cart.checkout")));

    html! {
        <section class="panel store-cart-panel" role="region" aria-labelledby="cart-title" onkeydown={on_keydown}>
            <header class="section-header">
                <h1 id="cart-title">{ i18n::t("store.cart.title") }</h1>
                <div class="store-cart-summary" aria-live="polite">
                    <span class="label">{ i18n::t("store.budget") }</span>
                    <span class="value">{ remaining_str }</span>
                </div>
            </header>
            { if state.cart.lines.is_empty() {
                html! { <p class="empty-cart">{ "NONE" }</p> }
            } else {
                html! {
                    <div class="cart-body">
                        <ul role="menu" aria-label={i18n::t("store.cart.title")} ref={list_ref} class="store-cart-list">
                            { for cart_lines.iter().enumerate().map(|(i, (idx, label))| {
                                let focused = state.focus_idx == *idx;
                                let disabled = *idx == 0 && !can_checkout;
                                let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);

                                html!{
                                    <li role="menuitem"
                                        tabindex={if focused && !disabled { "0" } else { "-1" }}
                                        data-key={idx.to_string()}
                                        aria-posinset={posinset.to_string()}
                                        aria-setsize={cart_lines.len().to_string()}
                                        aria-disabled={disabled.to_string()}
                                        class={classes!("ot-menuitem", "store-cart-line", disabled.then_some("disabled"))}>
                                        <span class="num">{ format!("{})", idx) }</span>
                                        <span class="label">{ label.clone() }</span>
                                    </li>
                                }
                            }) }
                        </ul>
                        <p class="cart-total" aria-live="polite">{ cart_total_line }</p>
                        { if can_checkout {
                            html! {}
                        } else {
                            html! { <p class="error" role="alert">{ i18n::t("store.alerts.over_budget") }</p> }
                        }}
                    </div>
                }
            }}
            <footer class="panel-footer">
                <button class="retro-btn-secondary" onclick={on_back}>{ i18n::t("store.menu.back") }</button>
                <button class="retro-btn-primary" onclick={on_checkout} disabled={!can_checkout}>
                    { i18n::t("store.cart.checkout") }
                </button>
            </footer>
            <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
        </section>
    }
}
