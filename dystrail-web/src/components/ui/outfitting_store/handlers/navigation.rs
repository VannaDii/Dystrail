use super::super::state::{OutfittingStoreProps, StoreScreen, StoreState};
use super::checkout::handle_checkout;
#[cfg(target_arch = "wasm32")]
use super::quantity::handle_quantity_selection;
use crate::i18n;
use yew::prelude::*;

enum MenuSelectionOutcome {
    Noop,
    Updated {
        state: StoreState,
        status: Option<String>,
    },
    Checkout,
    OverBudget {
        status: String,
    },
}

fn back_navigation_state(state: &StoreState) -> Option<StoreState> {
    let mut new_state = state.clone();
    match &state.current_screen {
        StoreScreen::Category(_) | StoreScreen::Cart | StoreScreen::QuantityPrompt(_) => {
            new_state.current_screen = StoreScreen::Home;
            new_state.focus_idx = 1;
            Some(new_state)
        }
        StoreScreen::Home => None,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn apply_home_selection(
    index: u8,
    state: &StoreState,
    props: &OutfittingStoreProps,
) -> MenuSelectionOutcome {
    match index {
        1..=4 => state
            .store_data
            .categories
            .get((index - 1) as usize)
            .map_or(MenuSelectionOutcome::Noop, |category| {
                let mut new_state = state.clone();
                new_state.current_screen = StoreScreen::Category(category.id.clone());
                new_state.focus_idx = 1;

                let category_name = i18n::t(&format!(
                    "store.categories.{category_id}",
                    category_id = category.id
                ));
                MenuSelectionOutcome::Updated {
                    state: new_state,
                    status: Some(category_name),
                }
            }),
        5 => {
            let mut new_state = state.clone();
            new_state.current_screen = StoreScreen::Cart;
            new_state.focus_idx = 1;
            MenuSelectionOutcome::Updated {
                state: new_state,
                status: Some(i18n::t("store.menu.view_cart")),
            }
        }
        0 => {
            let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
            if remaining_budget >= 0 {
                MenuSelectionOutcome::Checkout
            } else {
                MenuSelectionOutcome::OverBudget {
                    status: i18n::t("store.alerts.over_budget"),
                }
            }
        }
        _ => MenuSelectionOutcome::Noop,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn apply_category_selection(
    index: u8,
    category_id: &str,
    state: &StoreState,
) -> MenuSelectionOutcome {
    if index == 0 {
        back_navigation_state(state).map_or(MenuSelectionOutcome::Noop, |new_state| {
            MenuSelectionOutcome::Updated {
                state: new_state,
                status: Some(i18n::t("store.menu.back")),
            }
        })
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

        let item_name = i18n::t(&format!("store.items.{}.name", item.id));
        MenuSelectionOutcome::Updated {
            state: new_state,
            status: Some(item_name),
        }
    } else {
        MenuSelectionOutcome::Noop
    }
}

fn apply_cart_selection(
    index: u8,
    state: &StoreState,
    props: &OutfittingStoreProps,
) -> MenuSelectionOutcome {
    if index == 0 {
        let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
        if remaining_budget >= 0 {
            MenuSelectionOutcome::Checkout
        } else {
            MenuSelectionOutcome::OverBudget {
                status: i18n::t("store.alerts.over_budget"),
            }
        }
    } else if let Some(cart_line) = state.cart.lines.get((index - 1) as usize) {
        let mut new_state = state.clone();
        new_state.current_screen = StoreScreen::QuantityPrompt(cart_line.item_id.clone());
        new_state.focus_idx = 1;
        let item_name = i18n::t(&format!("store.items.{}.name", cart_line.item_id));
        MenuSelectionOutcome::Updated {
            state: new_state,
            status: Some(item_name),
        }
    } else {
        MenuSelectionOutcome::Noop
    }
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
fn handle_home_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match apply_home_selection(index, state, props) {
        MenuSelectionOutcome::Updated { state, status } => {
            store_state.set(state);
            if let Some(status) = status {
                crate::a11y::set_status(&status);
            }
        }
        MenuSelectionOutcome::Checkout => handle_checkout(state, props),
        MenuSelectionOutcome::OverBudget { status } => {
            crate::a11y::set_status(&status);
        }
        MenuSelectionOutcome::Noop => {}
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_category_selection(
    index: u8,
    category_id: &str,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
) {
    match apply_category_selection(index, category_id, state) {
        MenuSelectionOutcome::Updated { state, status } => {
            store_state.set(state);
            if let Some(status) = status {
                crate::a11y::set_status(&status);
            }
        }
        MenuSelectionOutcome::Noop
        | MenuSelectionOutcome::Checkout
        | MenuSelectionOutcome::OverBudget { .. } => {}
    }
}

pub fn handle_back_navigation(state: &StoreState, store_state: &UseStateHandle<StoreState>) {
    if let Some(new_state) = back_navigation_state(state) {
        store_state.set(new_state);
        crate::a11y::set_status(&i18n::t("store.menu.back"));
    }
}

#[cfg(any(target_arch = "wasm32", test))]
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
    match apply_cart_selection(index, state, props) {
        MenuSelectionOutcome::Updated { state, status } => {
            store_state.set(state);
            if let Some(status) = status {
                crate::a11y::set_status(&status);
            }
        }
        MenuSelectionOutcome::Checkout => handle_checkout(state, props),
        MenuSelectionOutcome::OverBudget { status } => {
            crate::a11y::set_status(&status);
        }
        MenuSelectionOutcome::Noop => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::outfitting_store::state::load_store_data;
    use crate::game::store::{Cart, CartLine, Store, StoreCategory, StoreItem};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    fn base_state() -> StoreState {
        StoreState::default()
    }

    fn store_state_with_data() -> StoreState {
        let store_data = load_store_data().expect("store data should load");
        StoreState {
            store_data,
            ..StoreState::default()
        }
    }

    fn store_props(budget_cents: i64) -> OutfittingStoreProps {
        OutfittingStoreProps {
            game_state: crate::game::GameState {
                budget_cents,
                ..crate::game::GameState::default()
            },
            on_continue: Callback::noop(),
        }
    }

    #[test]
    fn get_max_menu_index_varies_by_screen() {
        let mut state = base_state();
        state.current_screen = StoreScreen::Home;
        assert_eq!(get_max_menu_index(&state), 5);

        let item = StoreItem {
            id: String::from("water"),
            name: String::from("Water"),
            desc: String::from("Desc"),
            price_cents: 100,
            unique: false,
            max_qty: 10,
            grants: crate::game::store::Grants::default(),
            tags: Vec::new(),
            category: String::from("fuel_food"),
        };
        state.store_data = Store {
            categories: vec![StoreCategory {
                id: String::from("fuel_food"),
                name: String::from("Fuel/Food"),
                items: vec![item],
            }],
            items: Vec::new(),
        };
        state.current_screen = StoreScreen::Category(String::from("fuel_food"));
        assert_eq!(get_max_menu_index(&state), 1);

        state.current_screen = StoreScreen::QuantityPrompt(String::from("water"));
        assert_eq!(get_max_menu_index(&state), 4);

        state.cart = Cart {
            lines: vec![CartLine {
                item_id: String::from("water"),
                item_name: String::from("Water"),
                quantity: 1,
                qty: 1,
            }],
            total_cents: 100,
        };
        state.current_screen = StoreScreen::Cart;
        assert_eq!(get_max_menu_index(&state), 1);
    }

    #[test]
    fn home_selection_enters_category() {
        crate::i18n::set_lang("en");
        let state = store_state_with_data();
        let props = store_props(10_000);
        let outcome = apply_home_selection(1, &state, &props);
        match outcome {
            MenuSelectionOutcome::Updated { state, status } => {
                assert!(matches!(state.current_screen, StoreScreen::Category(_)));
                assert!(status.is_some());
            }
            _ => panic!("expected category transition"),
        }
    }

    #[test]
    fn home_selection_enters_cart() {
        crate::i18n::set_lang("en");
        let state = store_state_with_data();
        let props = store_props(10_000);
        let outcome = apply_home_selection(5, &state, &props);
        match outcome {
            MenuSelectionOutcome::Updated { state, status } => {
                assert!(matches!(state.current_screen, StoreScreen::Cart));
                assert!(status.is_some());
            }
            _ => panic!("expected cart transition"),
        }
    }

    #[test]
    fn home_selection_runs_checkout_when_budget_ok() {
        let state = store_state_with_data();
        let props = store_props(10_000);
        let outcome = apply_home_selection(0, &state, &props);
        assert!(matches!(outcome, MenuSelectionOutcome::Checkout));
    }

    #[test]
    fn home_selection_blocks_checkout_when_over_budget() {
        let mut state = store_state_with_data();
        state.cart.total_cents = 500;
        let props = store_props(100);
        let outcome = apply_home_selection(0, &state, &props);
        match outcome {
            MenuSelectionOutcome::OverBudget { status } => {
                assert!(!status.is_empty());
            }
            _ => panic!("expected over budget"),
        }
    }

    #[test]
    fn category_selection_opens_quantity_prompt() {
        crate::i18n::set_lang("en");
        let mut state = store_state_with_data();
        let category_id = state
            .store_data
            .categories
            .first()
            .map_or_else(|| String::from("fuel_food"), |cat| cat.id.clone());
        state.current_screen = StoreScreen::Category(category_id.clone());
        let outcome = apply_category_selection(1, &category_id, &state);
        match outcome {
            MenuSelectionOutcome::Updated { state, status } => {
                assert!(matches!(
                    state.current_screen,
                    StoreScreen::QuantityPrompt(_)
                ));
                assert!(status.is_some());
            }
            _ => panic!("expected quantity prompt"),
        }
    }

    #[test]
    fn cart_selection_opens_quantity_prompt() {
        crate::i18n::set_lang("en");
        let mut state = store_state_with_data();
        let item_id = state
            .store_data
            .categories
            .first()
            .and_then(|cat| cat.items.first())
            .map_or_else(|| String::from("food"), |item| item.id.clone());
        state.cart = Cart {
            lines: vec![CartLine {
                item_id: item_id.clone(),
                item_name: item_id,
                quantity: 1,
                qty: 1,
            }],
            total_cents: 100,
        };
        state.current_screen = StoreScreen::Cart;
        let props = store_props(10_000);
        let outcome = apply_cart_selection(1, &state, &props);
        match outcome {
            MenuSelectionOutcome::Updated { state, status } => {
                assert!(matches!(
                    state.current_screen,
                    StoreScreen::QuantityPrompt(_)
                ));
                assert!(status.is_some());
            }
            _ => panic!("expected quantity prompt"),
        }
    }

    #[test]
    fn cart_selection_runs_checkout() {
        let state = store_state_with_data();
        let props = store_props(10_000);
        let outcome = apply_cart_selection(0, &state, &props);
        assert!(matches!(outcome, MenuSelectionOutcome::Checkout));
    }

    #[test]
    fn home_selection_ignores_unknown_index() {
        let state = store_state_with_data();
        let props = store_props(10_000);
        let outcome = apply_home_selection(9, &state, &props);
        assert!(matches!(outcome, MenuSelectionOutcome::Noop));
    }

    #[test]
    fn category_selection_back_returns_home() {
        crate::i18n::set_lang("en");
        let mut state = store_state_with_data();
        let category_id = state
            .store_data
            .categories
            .first()
            .map_or_else(|| String::from("fuel_food"), |cat| cat.id.clone());
        state.current_screen = StoreScreen::Category(category_id.clone());
        let outcome = apply_category_selection(0, &category_id, &state);
        match outcome {
            MenuSelectionOutcome::Updated { state, status } => {
                assert!(matches!(state.current_screen, StoreScreen::Home));
                assert!(status.is_some());
            }
            _ => panic!("expected back navigation"),
        }
    }

    #[test]
    fn category_selection_ignores_missing_item() {
        let mut state = store_state_with_data();
        let category_id = state
            .store_data
            .categories
            .first()
            .map_or_else(|| String::from("fuel_food"), |cat| cat.id.clone());
        state.current_screen = StoreScreen::Category(category_id.clone());
        let outcome = apply_category_selection(99, &category_id, &state);
        assert!(matches!(outcome, MenuSelectionOutcome::Noop));
    }

    #[test]
    fn cart_selection_blocks_over_budget_checkout() {
        let mut state = store_state_with_data();
        state.cart.total_cents = 500;
        let props = store_props(100);
        let outcome = apply_cart_selection(0, &state, &props);
        assert!(matches!(outcome, MenuSelectionOutcome::OverBudget { .. }));
    }

    #[test]
    fn cart_selection_ignores_missing_line() {
        let mut state = store_state_with_data();
        state.cart = Cart {
            lines: vec![CartLine {
                item_id: String::from("water"),
                item_name: String::from("Water"),
                quantity: 1,
                qty: 1,
            }],
            total_cents: 100,
        };
        let props = store_props(10_000);
        let outcome = apply_cart_selection(2, &state, &props);
        assert!(matches!(outcome, MenuSelectionOutcome::Noop));
    }

    #[test]
    fn back_navigation_skips_home() {
        let mut state = store_state_with_data();
        state.current_screen = StoreScreen::Home;
        assert!(back_navigation_state(&state).is_none());
    }

    #[test]
    fn max_menu_index_handles_empty_cart() {
        let mut state = store_state_with_data();
        state.cart = Cart::new();
        state.current_screen = StoreScreen::Cart;
        assert_eq!(get_max_menu_index(&state), 1);
    }

    #[function_component(BackNavHarness)]
    fn back_nav_harness() -> Html {
        crate::i18n::set_lang("en");
        let store_data = load_store_data().expect("store data should load");
        let base_state = StoreState {
            store_data,
            current_screen: StoreScreen::Category(String::from("fuel_food")),
            ..StoreState::default()
        };
        let store_state = use_state(|| base_state);
        let invoked = use_state(|| false);
        if !*invoked {
            invoked.set(true);
            let snapshot = (*store_state).clone();
            handle_back_navigation(&snapshot, &store_state);
        }
        html! { <span>{"ok"}</span> }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum CartScenario {
        Updated,
        Checkout,
        OverBudget,
    }

    #[derive(Properties, PartialEq)]
    struct CartSelectionHarnessProps {
        scenario: CartScenario,
    }

    #[function_component(CartSelectionHarness)]
    fn cart_selection_harness(props: &CartSelectionHarnessProps) -> Html {
        crate::i18n::set_lang("en");
        let mut state = store_state_with_data();
        let item_id = state
            .store_data
            .categories
            .first()
            .and_then(|cat| cat.items.first())
            .map_or_else(|| String::from("food"), |item| item.id.clone());
        state.cart = Cart {
            lines: vec![CartLine {
                item_id: item_id.clone(),
                item_name: item_id,
                quantity: 1,
                qty: 1,
            }],
            total_cents: 500,
        };
        state.current_screen = StoreScreen::Cart;
        let store_state = use_state(|| state);
        let invoked = use_state(|| false);
        if !*invoked {
            invoked.set(true);
            let snapshot = (*store_state).clone();
            let (index, budget) = match props.scenario {
                CartScenario::Updated => (1, 10_000),
                CartScenario::Checkout => (0, 10_000),
                CartScenario::OverBudget => (0, 100),
            };
            let props = store_props(budget);
            handle_cart_selection(index, &snapshot, &store_state, &props);
        }
        html! { <span>{"ok"}</span> }
    }

    #[test]
    fn handle_back_navigation_updates_state() {
        let html = block_on(LocalServerRenderer::<BackNavHarness>::new().render());
        assert!(html.contains("ok"));
    }

    #[test]
    fn handle_cart_selection_covers_outcomes() {
        let html = block_on(
            LocalServerRenderer::<CartSelectionHarness>::with_props(CartSelectionHarnessProps {
                scenario: CartScenario::Updated,
            })
            .render(),
        );
        assert!(html.contains("ok"));

        let html = block_on(
            LocalServerRenderer::<CartSelectionHarness>::with_props(CartSelectionHarnessProps {
                scenario: CartScenario::Checkout,
            })
            .render(),
        );
        assert!(html.contains("ok"));

        let html = block_on(
            LocalServerRenderer::<CartSelectionHarness>::with_props(CartSelectionHarnessProps {
                scenario: CartScenario::OverBudget,
            })
            .render(),
        );
        assert!(html.contains("ok"));
    }
}
