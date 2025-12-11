use super::super::super::super::handlers::{
    announce_cannot_add, announce_quantity_change, can_add_item,
};
use super::super::super::super::state::{StoreScreen, StoreState};
use crate::game::store::{StoreItem, calculate_cart_total};
use yew::prelude::*;

pub fn quantity_select_callback(
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
