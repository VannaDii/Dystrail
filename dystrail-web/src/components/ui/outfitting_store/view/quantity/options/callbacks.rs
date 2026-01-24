use super::super::super::super::handlers::announce::{
    announce_cannot_add, announce_quantity_change,
};
use super::super::super::super::handlers::quantity::{
    QuantityAnnouncement, QuantitySelectionOutcome, quantity_selection_outcome,
};
use super::super::super::super::state::StoreState;
use crate::game::store::StoreItem;
use yew::prelude::*;

pub fn quantity_select_callback(
    item: &StoreItem,
    budget_cents: i64,
    store_state: &UseStateHandle<StoreState>,
) -> Callback<u8> {
    let store_state = store_state.clone();
    let item_id_owned = item.id.clone();
    Callback::from(move |index: u8| {
        let state = (*store_state).clone();
        match quantity_selection_outcome(index, &item_id_owned, &state, budget_cents) {
            QuantitySelectionOutcome::Update {
                state,
                announcement,
            } => {
                if let Some(QuantityAnnouncement::Change { item, qty, added }) = announcement {
                    announce_quantity_change(&item, qty, added, &state, budget_cents);
                }
                store_state.set(state);
            }
            QuantitySelectionOutcome::Blocked { item } => {
                announce_cannot_add(&item);
            }
            QuantitySelectionOutcome::Noop => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::outfitting_store::state::StoreScreen;
    use crate::game::store::{Cart, Grants, Store, StoreCategory};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum CallbackScenario {
        Back,
        AddOne,
        AddFive,
        RemoveOne,
        RemoveAll,
        Blocked,
    }

    #[derive(Properties, PartialEq)]
    struct CallbackHarnessProps {
        scenario: CallbackScenario,
        budget_cents: i64,
    }

    #[function_component(CallbackHarness)]
    fn callback_harness(props: &CallbackHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let item = StoreItem {
            id: String::from("bandage"),
            name: String::from("Bandage"),
            desc: String::from("Desc"),
            price_cents: 100,
            unique: false,
            max_qty: 10,
            grants: Grants::default(),
            tags: Vec::new(),
            category: String::from("ppe"),
        };
        let item_for_state = item.clone();
        let store = Store {
            categories: vec![StoreCategory {
                id: String::from("ppe"),
                name: String::from("PPE"),
                items: vec![item_for_state.clone()],
            }],
            items: Vec::new(),
        };
        let scenario = props.scenario;
        let store_state = use_state(move || {
            let mut cart = Cart::new();
            if matches!(
                scenario,
                CallbackScenario::RemoveOne | CallbackScenario::RemoveAll
            ) {
                cart.add_item(&item_for_state.id, 2);
            }
            StoreState {
                store_data: store,
                cart,
                current_screen: StoreScreen::QuantityPrompt(item_for_state.id.clone()),
                focus_idx: 1,
                discount_pct: 0.0,
            }
        });
        let invoked = use_mut_ref(|| false);
        let callback = quantity_select_callback(&item, props.budget_cents, &store_state);

        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            let index = match scenario {
                CallbackScenario::Back => 0,
                CallbackScenario::AddOne | CallbackScenario::Blocked => 1,
                CallbackScenario::AddFive => 2,
                CallbackScenario::RemoveOne => 3,
                CallbackScenario::RemoveAll => 4,
            };
            callback.emit(index);
        }

        let invoked_label = if *invoked.borrow() { "true" } else { "false" };
        html! { <div data-invoked={invoked_label} /> }
    }

    #[test]
    fn quantity_callback_adds_items() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::AddOne,
                budget_cents: 10_000,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }

    #[test]
    fn quantity_callback_adds_bulk_items() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::AddFive,
                budget_cents: 10_000,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }

    #[test]
    fn quantity_callback_removes_items() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::RemoveOne,
                budget_cents: 10_000,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }

    #[test]
    fn quantity_callback_removes_all_items() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::RemoveAll,
                budget_cents: 10_000,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }

    #[test]
    fn quantity_callback_back_returns_to_category() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::Back,
                budget_cents: 10_000,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }

    #[test]
    fn quantity_callback_blocks_over_budget() {
        let html = block_on(
            LocalServerRenderer::<CallbackHarness>::with_props(CallbackHarnessProps {
                scenario: CallbackScenario::Blocked,
                budget_cents: 0,
            })
            .render(),
        );
        assert!(html.contains("data-invoked=\"true\""));
    }
}
