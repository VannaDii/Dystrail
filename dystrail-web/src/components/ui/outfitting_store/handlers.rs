use super::state::{OutfittingStoreProps, StoreScreen, StoreState};
use crate::a11y::set_status;
use crate::game::store::{
    Cart, Grants, StoreItem, calculate_cart_total, calculate_effective_price,
};
use crate::i18n;
use std::collections::BTreeMap;
use yew::prelude::*;

pub(super) fn handle_menu_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match &state.current_screen {
        StoreScreen::Home => match index {
            1..=4 => {
                if let Some(category) = state.store_data.categories.get((index - 1) as usize) {
                    let mut new_state = state.clone();
                    new_state.current_screen = StoreScreen::Category(category.id.clone());
                    new_state.focus_idx = 1;
                    store_state.set(new_state);

                    let category_name = i18n::t(&format!(
                        "store.categories.{category_id}",
                        category_id = category.id
                    ));
                    announce_to_screen_reader(&category_name);
                }
            }
            5 => {
                let mut new_state = state.clone();
                new_state.current_screen = StoreScreen::Cart;
                new_state.focus_idx = 1;
                store_state.set(new_state);

                announce_to_screen_reader(&i18n::t("store.menu.view_cart"));
            }
            0 => {
                let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
                if remaining_budget >= 0 {
                    handle_checkout(state, props);
                } else {
                    announce_to_screen_reader(&i18n::t("store.alerts.over_budget"));
                }
            }
            _ => {}
        },
        StoreScreen::Category(category_id) => {
            if index == 0 {
                handle_back_navigation(state, store_state);
            } else if let Some(item) = state
                .store_data
                .categories
                .iter()
                .find(|c| c.id == *category_id)
                .and_then(|category| category.items.get((index - 1) as usize))
            {
                let mut new_state = state.clone();
                new_state.current_screen = StoreScreen::QuantityPrompt(item.id.clone());
                new_state.focus_idx = 1;
                store_state.set(new_state);

                let item_name = i18n::t(&format!("store.items.{}.name", item.id));
                announce_to_screen_reader(&item_name);
            }
        }
        StoreScreen::QuantityPrompt(item_id) => {
            handle_quantity_selection(index, item_id, state, store_state, props);
        }
        StoreScreen::Cart => {
            handle_cart_selection(index, state, store_state, props);
        }
    }
}

pub(super) fn handle_back_navigation(state: &StoreState, store_state: &UseStateHandle<StoreState>) {
    let mut new_state = state.clone();
    match &state.current_screen {
        StoreScreen::Category(_) | StoreScreen::Cart | StoreScreen::QuantityPrompt(_) => {
            new_state.current_screen = StoreScreen::Home;
        }
        StoreScreen::Home => {
            return;
        }
    }
    new_state.focus_idx = 1;
    store_state.set(new_state);

    announce_to_screen_reader(&i18n::t("store.menu.back"));
}

pub(super) fn get_max_menu_index(state: &StoreState) -> u8 {
    match &state.current_screen {
        StoreScreen::Home => 5,
        StoreScreen::Category(category_id) => state
            .store_data
            .categories
            .iter()
            .find(|c| c.id == *category_id)
            .map_or(1, |category| {
                u8::try_from(category.items.len()).unwrap_or(u8::MAX)
            }),
        StoreScreen::QuantityPrompt(_) => 4,
        StoreScreen::Cart => {
            if state.cart.lines.is_empty() {
                1
            } else {
                u8::try_from(state.cart.lines.len()).unwrap_or(u8::MAX)
            }
        }
    }
}

pub(super) fn handle_quantity_selection(
    index: u8,
    item_id: &str,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    if let Some(item) = state.store_data.find_item(item_id) {
        let mut new_state = state.clone();
        match index {
            0 => {
                new_state.current_screen = StoreScreen::Category(item.category.clone());
                new_state.focus_idx = 1;
                store_state.set(new_state);
                return;
            }
            1 => {
                if can_add_item(
                    &new_state.cart,
                    item,
                    1,
                    props.game_state.budget_cents,
                    new_state.discount_pct,
                ) {
                    new_state.cart.add_item(item_id, 1);
                    announce_quantity_change(
                        item,
                        1,
                        true,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                } else {
                    announce_cannot_add(item);
                    return;
                }
            }
            2 => {
                if can_add_item(
                    &new_state.cart,
                    item,
                    5,
                    props.game_state.budget_cents,
                    new_state.discount_pct,
                ) {
                    new_state.cart.add_item(item_id, 5);
                    announce_quantity_change(
                        item,
                        5,
                        true,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                } else {
                    announce_cannot_add(item);
                    return;
                }
            }
            3 => {
                if new_state.cart.get_quantity(item_id) > 0 {
                    new_state.cart.remove_item(item_id, 1);
                    announce_quantity_change(
                        item,
                        1,
                        false,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                }
            }
            4 => {
                if new_state.cart.get_quantity(item_id) > 0 {
                    new_state.cart.remove_all_item(item_id);
                    announce_quantity_change(
                        item,
                        0,
                        false,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                }
            }
            _ => return,
        }

        new_state.cart.total_cents =
            calculate_cart_total(&new_state.cart, &state.store_data, state.discount_pct);
        store_state.set(new_state);
    }
}

pub(super) fn can_add_item(
    cart: &Cart,
    item: &StoreItem,
    qty_to_add: i32,
    budget_cents: i64,
    discount_pct: f64,
) -> bool {
    let current_qty = cart.get_quantity(&item.id);
    let new_qty = current_qty + qty_to_add;

    if new_qty > item.max_qty {
        return false;
    }

    if item.unique && new_qty > 1 {
        return false;
    }

    let effective_price = calculate_effective_price(item.price_cents, discount_pct);
    let additional_cost = effective_price * i64::from(qty_to_add);
    let new_total = cart.total_cents + additional_cost;

    new_total <= budget_cents
}

pub(super) fn announce_quantity_change(
    item: &StoreItem,
    qty: i32,
    added: bool,
    state: &StoreState,
    budget_cents: i64,
) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(if added {
        effective_price * i64::from(qty)
    } else {
        effective_price
    });
    let remaining = budget_cents - state.cart.total_cents;
    let remaining_str = format_currency(remaining);

    let message = if added {
        i18n::tr(
            "store.alerts.added",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("price", price_str.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.removed",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    };

    announce_to_screen_reader(&message);
}

pub(super) fn announce_cannot_add(item: &StoreItem) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let message = if item.unique {
        i18n::tr(
            "store.alerts.unique",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.max_qty",
            Some(&{
                let mut vars = BTreeMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    };

    announce_to_screen_reader(&message);
}

pub(super) fn handle_cart_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    if index == 0 {
        let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
        if remaining_budget >= 0 {
            handle_checkout(state, props);
        } else {
            announce_to_screen_reader(&i18n::t("store.alerts.over_budget"));
        }
    } else if let Some(cart_line) = state.cart.lines.get((index - 1) as usize) {
        let mut new_state = state.clone();
        new_state.current_screen = StoreScreen::QuantityPrompt(cart_line.item_id.clone());
        new_state.focus_idx = 1;
        store_state.set(new_state);
    }
}

pub(super) fn handle_checkout(state: &StoreState, props: &OutfittingStoreProps) {
    let mut total_grants = Grants::default();
    let mut all_tags = Vec::new();

    for line in &state.cart.lines {
        if let Some(item) = state.store_data.find_item(&line.item_id) {
            total_grants.supplies += item.grants.supplies * line.qty;
            total_grants.credibility += item.grants.credibility * line.qty;
            total_grants.spare_tire += item.grants.spare_tire * line.qty;
            total_grants.spare_battery += item.grants.spare_battery * line.qty;
            total_grants.spare_alt += item.grants.spare_alt * line.qty;
            total_grants.spare_pump += item.grants.spare_pump * line.qty;

            for tag in &item.tags {
                all_tags.push(tag.clone());
            }
        }
    }

    let mut new_game_state = props.game_state.clone();
    new_game_state.apply_store_purchase(state.cart.total_cents, &total_grants, &all_tags);

    props
        .on_continue
        .emit((new_game_state, total_grants, all_tags));
}

pub(super) fn format_currency(cents: i64) -> String {
    crate::i18n::fmt_currency(cents)
}

pub(super) fn announce_to_screen_reader(message: &str) {
    set_status(message);
}
