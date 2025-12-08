use super::super::state::{OutfittingStoreProps, StoreScreen, StoreState};
use super::announce::{announce_cannot_add, announce_quantity_change};
use crate::game::store::{Cart, StoreItem, calculate_cart_total, calculate_effective_price};
use yew::prelude::*;

pub fn handle_quantity_selection(
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

pub fn can_add_item(
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
