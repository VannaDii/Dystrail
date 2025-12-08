use super::super::handlers::{
    announce_cannot_add, announce_quantity_change, can_add_item, format_currency,
};
use super::super::state::{StoreScreen, StoreState};
use crate::game::store::calculate_cart_total;
use crate::game::{
    GameState,
    store::{StoreItem, calculate_effective_price},
};
use crate::i18n;
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(Debug, Clone)]
struct QuantityOption {
    idx: u8,
    label: String,
    preview: String,
}

fn build_quantity_options(
    item: &StoreItem,
    state: &StoreState,
    game_state: &GameState,
    effective_price: i64,
    current_qty: i32,
) -> Vec<QuantityOption> {
    let mut options = Vec::new();

    let can_add_1 = can_add_item(
        &state.cart,
        item,
        1,
        game_state.budget_cents,
        state.discount_pct,
    );
    options.push(if can_add_1 {
        let budget_preview = game_state.budget_cents - state.cart.total_cents - effective_price;
        let preview_str = format!(
            "[{}{}]",
            i18n::t("store.budget"),
            format_currency(budget_preview)
        );
        QuantityOption {
            idx: 1,
            label: i18n::t("store.qty_prompt.add1"),
            preview: preview_str,
        }
    } else {
        QuantityOption {
            idx: 1,
            label: i18n::t("store.qty_prompt.add1"),
            preview: String::from("[Max/Budget]"),
        }
    });

    if !item.unique {
        let can_add_5 = can_add_item(
            &state.cart,
            item,
            5,
            game_state.budget_cents,
            state.discount_pct,
        );
        let add5 = if can_add_5 {
            let budget_preview =
                game_state.budget_cents - state.cart.total_cents - (effective_price * 5);
            let preview_str = format!(
                "[{}{}]",
                i18n::t("store.budget"),
                format_currency(budget_preview)
            );
            QuantityOption {
                idx: 2,
                label: i18n::t("store.qty_prompt.add5"),
                preview: preview_str,
            }
        } else {
            QuantityOption {
                idx: 2,
                label: i18n::t("store.qty_prompt.add5"),
                preview: String::from("[Max/Budget]"),
            }
        };
        options.push(add5);
    }

    if current_qty > 0 {
        options.push(QuantityOption {
            idx: 3,
            label: i18n::t("store.qty_prompt.rem1"),
            preview: String::new(),
        });
        options.push(QuantityOption {
            idx: 4,
            label: i18n::t("store.qty_prompt.rem_all"),
            preview: String::new(),
        });
    }

    options.push(QuantityOption {
        idx: 0,
        label: i18n::t("store.menu.back"),
        preview: String::new(),
    });

    options
}

fn quantity_select_callback(
    item: &StoreItem,
    budget_cents: i64,
    store_state: &UseStateHandle<StoreState>,
) -> Callback<u8> {
    let store_state = store_state.clone();
    let item_owned = item.clone();
    let item_category = item.category.clone();
    let item_id_owned = item.id.clone();
    let store_data = store_state.store_data.clone();
    Callback::from(move |index: u8| {
        let mut new_state = (*store_state).clone();
        match index {
            0 => {
                new_state.current_screen = StoreScreen::Category(item_category.clone());
                new_state.focus_idx = 1;
                store_state.set(new_state);
                return;
            }
            1 => {
                if can_add_item(
                    &new_state.cart,
                    &item_owned,
                    1,
                    budget_cents,
                    new_state.discount_pct,
                ) {
                    new_state.cart.add_item(&item_id_owned, 1);
                    announce_quantity_change(&item_owned, 1, true, &new_state, budget_cents);
                } else {
                    announce_cannot_add(&item_owned);
                    return;
                }
            }
            2 => {
                if can_add_item(
                    &new_state.cart,
                    &item_owned,
                    5,
                    budget_cents,
                    new_state.discount_pct,
                ) {
                    new_state.cart.add_item(&item_id_owned, 5);
                    announce_quantity_change(&item_owned, 5, true, &new_state, budget_cents);
                } else {
                    announce_cannot_add(&item_owned);
                    return;
                }
            }
            3 => {
                if new_state.cart.get_quantity(&item_id_owned) > 0 {
                    new_state.cart.remove_item(&item_id_owned, 1);
                    announce_quantity_change(&item_owned, 1, false, &new_state, budget_cents);
                }
            }
            4 => {
                if new_state.cart.get_quantity(&item_id_owned) > 0 {
                    new_state.cart.remove_all_item(&item_id_owned);
                    announce_quantity_change(&item_owned, 0, false, &new_state, budget_cents);
                }
            }
            _ => return,
        }

        new_state.cart.total_cents =
            calculate_cart_total(&new_state.cart, &store_data, new_state.discount_pct);
        store_state.set(new_state);
    })
}

fn render_quantity_options(
    options: &[QuantityOption],
    focus_idx: u8,
    on_select: &Callback<u8>,
) -> Html {
    html! {
        <ul role="menu" aria-label={i18n::t("store.qty_prompt.title")} >
            { for options.iter().enumerate().map(|(i, option)| {
                let focused = focus_idx == option.idx;
                let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
                html!{
                    <li role="menuitem"
                        tabindex={if focused { "0" } else { "-1" }}
                        data-key={option.idx.to_string()}
                        aria-posinset={posinset.to_string()}
                        aria-setsize={options.len().to_string()}
                        class="ot-menuitem"
                        onclick={{
                            let on_select = on_select.clone();
                            let idx = option.idx;
                            Callback::from(move |_| on_select.emit(idx))
                        }}>
                        <span class="num">{ format!("{})", option.idx) }</span>
                        <span class="label">
                            { option.label.clone() }
                            { if option.preview.is_empty() {
                                html! {}
                            } else {
                                html! { <span class="preview">{ format!(" {}", option.preview) }</span> }
                            }}
                        </span>
                    </li>
                }
            }) }
        </ul>
    }
}

pub fn render_quantity_screen(
    item_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<web_sys::KeyboardEvent>,
) -> Html {
    let Some(item) = state.store_data.find_item(item_id) else {
        return html! { <div>{ "Item not found" }</div> };
    };

    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(effective_price);

    let title = i18n::tr(
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
            <section role="region" aria-labelledby="qty-title" onkeydown={on_keydown}>
                <h1 id="qty-title">{ title }</h1>
                { if current_qty > 0 {
                    html! { <p class="current-qty">{ format!("Current in cart: {current_qty}") }</p> }
                } else {
                    html! {}
                }}
                <div ref={list_ref}>
                    { render_quantity_options(&options, state.focus_idx, &on_select) }
                </div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}
