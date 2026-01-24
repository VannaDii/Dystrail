#[cfg(target_arch = "wasm32")]
use super::super::state::OutfittingStoreProps;
use super::super::state::{StoreScreen, StoreState};
#[cfg(target_arch = "wasm32")]
use super::announce::{announce_cannot_add, announce_quantity_change};
use crate::game::store::calculate_cart_total;
use crate::game::store::{Cart, StoreItem, calculate_effective_price};
#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

/// Outcome of applying a quantity selection to store state.
pub enum QuantitySelectionOutcome {
    Noop,
    Update {
        state: StoreState,
        announcement: Option<QuantityAnnouncement>,
    },
    Blocked {
        item: StoreItem,
    },
}

/// Announcement metadata for quantity changes.
pub enum QuantityAnnouncement {
    Change {
        item: StoreItem,
        qty: i32,
        added: bool,
    },
}

/// Compute the next store state for a quantity selection without UI side effects.
pub fn quantity_selection_outcome(
    index: u8,
    item_id: &str,
    state: &StoreState,
    budget_cents: i64,
) -> QuantitySelectionOutcome {
    let Some(item) = state.store_data.find_item(item_id) else {
        return QuantitySelectionOutcome::Noop;
    };
    let mut new_state = state.clone();
    let mut announcement = None;

    match index {
        0 => {
            new_state.current_screen = StoreScreen::Category(item.category.clone());
            new_state.focus_idx = 1;
        }
        1 => {
            if can_add_item(
                &new_state.cart,
                item,
                1,
                budget_cents,
                new_state.discount_pct,
            ) {
                new_state.cart.add_item(item_id, 1);
                announcement = Some(QuantityAnnouncement::Change {
                    item: item.clone(),
                    qty: 1,
                    added: true,
                });
            } else {
                return QuantitySelectionOutcome::Blocked { item: item.clone() };
            }
        }
        2 => {
            if can_add_item(
                &new_state.cart,
                item,
                5,
                budget_cents,
                new_state.discount_pct,
            ) {
                new_state.cart.add_item(item_id, 5);
                announcement = Some(QuantityAnnouncement::Change {
                    item: item.clone(),
                    qty: 5,
                    added: true,
                });
            } else {
                return QuantitySelectionOutcome::Blocked { item: item.clone() };
            }
        }
        3 => {
            if new_state.cart.get_quantity(item_id) > 0 {
                new_state.cart.remove_item(item_id, 1);
                announcement = Some(QuantityAnnouncement::Change {
                    item: item.clone(),
                    qty: 1,
                    added: false,
                });
            }
        }
        4 => {
            if new_state.cart.get_quantity(item_id) > 0 {
                new_state.cart.remove_all_item(item_id);
                announcement = Some(QuantityAnnouncement::Change {
                    item: item.clone(),
                    qty: 0,
                    added: false,
                });
            }
        }
        _ => return QuantitySelectionOutcome::Noop,
    }

    new_state.cart.total_cents =
        calculate_cart_total(&new_state.cart, &state.store_data, state.discount_pct);
    QuantitySelectionOutcome::Update {
        state: new_state,
        announcement,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn handle_quantity_selection(
    index: u8,
    item_id: &str,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match quantity_selection_outcome(index, item_id, state, props.game_state.budget_cents) {
        QuantitySelectionOutcome::Update {
            state,
            announcement,
        } => {
            if let Some(QuantityAnnouncement::Change { item, qty, added }) = announcement {
                announce_quantity_change(&item, qty, added, &state, props.game_state.budget_cents);
            }
            store_state.set(state);
        }
        QuantitySelectionOutcome::Blocked { item } => {
            announce_cannot_add(&item);
        }
        QuantitySelectionOutcome::Noop => {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::store::Grants;
    use crate::game::store::{Cart, Store, StoreCategory};

    fn item(unique: bool, max_qty: i32, price_cents: i64) -> StoreItem {
        StoreItem {
            id: String::from("bandage"),
            name: String::from("Bandage"),
            desc: String::from("Desc"),
            price_cents,
            unique,
            max_qty,
            grants: Grants::default(),
            tags: Vec::new(),
            category: String::from("ppe"),
        }
    }

    #[test]
    fn can_add_item_respects_caps_and_budget() {
        let cart = Cart::new();
        let limited = item(false, 1, 100);
        assert!(!can_add_item(&cart, &limited, 2, 500, 0.0));

        let unique = item(true, 5, 100);
        assert!(!can_add_item(&cart, &unique, 2, 500, 0.0));

        let pricey = item(false, 5, 1_000);
        assert!(!can_add_item(&cart, &pricey, 1, 500, 0.0));

        let ok = item(false, 5, 100);
        assert!(can_add_item(&cart, &ok, 1, 500, 0.0));
    }

    fn store_state_with_item(qty: i32) -> (StoreState, StoreItem) {
        let store_item = item(false, 10, 100);
        let store = Store {
            categories: vec![StoreCategory {
                id: String::from("ppe"),
                name: String::from("PPE"),
                items: vec![store_item.clone()],
            }],
            items: Vec::new(),
        };
        let mut cart = Cart::new();
        if qty > 0 {
            cart.add_item(&store_item.id, qty);
        }
        let state = StoreState {
            store_data: store,
            cart,
            current_screen: StoreScreen::QuantityPrompt(store_item.id.clone()),
            focus_idx: 1,
            discount_pct: 0.0,
        };
        (state, store_item)
    }

    #[test]
    fn handle_quantity_selection_adds_items() {
        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(1, &item.id, &state, 10_000);
        match outcome {
            QuantitySelectionOutcome::Update { state, .. } => {
                assert_eq!(state.cart.get_quantity(&item.id), 1);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn handle_quantity_selection_adds_bulk_items() {
        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(2, &item.id, &state, 10_000);
        match outcome {
            QuantitySelectionOutcome::Update { state, .. } => {
                assert_eq!(state.cart.get_quantity(&item.id), 5);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn handle_quantity_selection_removes_items() {
        let (state, item) = store_state_with_item(2);
        let outcome = quantity_selection_outcome(3, &item.id, &state, 10_000);
        match outcome {
            QuantitySelectionOutcome::Update { state, .. } => {
                assert_eq!(state.cart.get_quantity(&item.id), 1);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn handle_quantity_selection_removes_all_items() {
        let (state, item) = store_state_with_item(2);
        let outcome = quantity_selection_outcome(4, &item.id, &state, 10_000);
        match outcome {
            QuantitySelectionOutcome::Update { state, .. } => {
                assert_eq!(state.cart.get_quantity(&item.id), 0);
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn handle_quantity_selection_back_returns_to_category() {
        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(0, &item.id, &state, 10_000);
        match outcome {
            QuantitySelectionOutcome::Update { state, .. } => {
                assert!(matches!(state.current_screen, StoreScreen::Category(_)));
            }
            _ => panic!("expected update"),
        }
    }

    #[test]
    fn handle_quantity_selection_blocks_when_over_budget() {
        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(1, &item.id, &state, 0);
        assert!(matches!(outcome, QuantitySelectionOutcome::Blocked { .. }));
    }

    #[test]
    fn handle_quantity_selection_blocks_bulk_add_when_over_budget() {
        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(2, &item.id, &state, 0);
        assert!(matches!(outcome, QuantitySelectionOutcome::Blocked { .. }));
    }

    #[test]
    fn quantity_selection_returns_noop_for_unknown_item_or_index() {
        let (state, _item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(1, "unknown", &state, 10_000);
        assert!(matches!(outcome, QuantitySelectionOutcome::Noop));

        let (state, item) = store_state_with_item(0);
        let outcome = quantity_selection_outcome(9, &item.id, &state, 10_000);
        assert!(matches!(outcome, QuantitySelectionOutcome::Noop));
    }
}
