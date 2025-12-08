use super::super::state::{OutfittingStoreProps, StoreScreen, StoreState};
use super::checkout::handle_checkout;
use super::quantity::handle_quantity_selection;
use crate::i18n;
use yew::prelude::*;

pub fn handle_menu_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match &state.current_screen {
        StoreScreen::Home => handle_home_selection(index, state, store_state, props),
        StoreScreen::Category(category_id) => {
            handle_category_selection(index, category_id, state, store_state);
        }
        StoreScreen::QuantityPrompt(item_id) => {
            handle_quantity_selection(index, item_id, state, store_state, props);
        }
        StoreScreen::Cart => {
            handle_cart_selection(index, state, store_state, props);
        }
    }
}

fn handle_home_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match index {
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
                crate::a11y::set_status(&category_name);
            }
        }
        5 => {
            let mut new_state = state.clone();
            new_state.current_screen = StoreScreen::Cart;
            new_state.focus_idx = 1;
            store_state.set(new_state);

            crate::a11y::set_status(&i18n::t("store.menu.view_cart"));
        }
        0 => {
            let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
            if remaining_budget >= 0 {
                handle_checkout(state, props);
            } else {
                crate::a11y::set_status(&i18n::t("store.alerts.over_budget"));
            }
        }
        _ => {}
    }
}

fn handle_category_selection(
    index: u8,
    category_id: &str,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
) {
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
        crate::a11y::set_status(&item_name);
    }
}

pub fn handle_back_navigation(state: &StoreState, store_state: &UseStateHandle<StoreState>) {
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

    crate::a11y::set_status(&i18n::t("store.menu.back"));
}

pub fn get_max_menu_index(state: &StoreState) -> u8 {
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

pub fn handle_cart_selection(
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
            crate::a11y::set_status(&i18n::t("store.alerts.over_budget"));
        }
    } else if let Some(cart_line) = state.cart.lines.get((index - 1) as usize) {
        let mut new_state = state.clone();
        new_state.current_screen = StoreScreen::QuantityPrompt(cart_line.item_id.clone());
        new_state.focus_idx = 1;
        store_state.set(new_state);
    }
}
