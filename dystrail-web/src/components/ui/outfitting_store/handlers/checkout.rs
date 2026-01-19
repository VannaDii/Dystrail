#[cfg(any(target_arch = "wasm32", test))]
use super::super::state::OutfittingStoreProps;
#[cfg(any(target_arch = "wasm32", test))]
use crate::game::store::Grants;

#[cfg(any(target_arch = "wasm32", test))]
pub fn handle_checkout(
    state: &crate::components::ui::outfitting_store::state::StoreState,
    props: &OutfittingStoreProps,
) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::outfitting_store::state::StoreState;
    use crate::game::GameState;
    use crate::game::store::{Cart, CartLine, Grants, Store, StoreCategory, StoreItem};
    use std::cell::RefCell;
    use std::rc::Rc;
    use yew::Callback;

    type CheckoutPayload = (GameState, Grants, Vec<String>);
    type CheckoutCapture = Rc<RefCell<Option<CheckoutPayload>>>;

    #[test]
    fn handle_checkout_emits_updated_state() {
        let item = StoreItem {
            id: String::from("water"),
            name: String::from("Water"),
            desc: String::from("Desc"),
            price_cents: 100,
            unique: false,
            max_qty: 10,
            grants: Grants {
                supplies: 2,
                ..Grants::default()
            },
            tags: vec![String::from("hydrated")],
            category: String::from("fuel_food"),
        };
        let category = StoreCategory {
            id: String::from("fuel_food"),
            name: String::from("Fuel/Food"),
            items: vec![item.clone()],
        };
        let store = Store {
            categories: vec![category],
            items: Vec::new(),
        };

        let StoreItem { id, name, .. } = item;
        let cart = Cart {
            lines: vec![CartLine {
                item_id: id,
                item_name: name,
                quantity: 2,
                qty: 2,
            }],
            total_cents: 200,
        };

        let state = StoreState {
            store_data: store,
            cart,
            current_screen: crate::components::ui::outfitting_store::state::StoreScreen::Cart,
            focus_idx: 1,
            discount_pct: 0.0,
        };

        let game_state = GameState {
            budget_cents: 500,
            ..GameState::default()
        };

        let captured: CheckoutCapture = Rc::new(RefCell::new(None));
        let captured_clone = captured.clone();
        let props = OutfittingStoreProps {
            game_state,
            on_continue: Callback::from(move |payload| {
                *captured_clone.borrow_mut() = Some(payload);
            }),
        };

        handle_checkout(&state, &props);
        let payload = captured.borrow().clone().expect("payload");
        assert_eq!(payload.1.supplies, 4);
        assert!(payload.2.contains(&String::from("hydrated")));
    }
}
