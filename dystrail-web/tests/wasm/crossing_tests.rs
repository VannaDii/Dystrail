use wasm_bindgen_test::*;
use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use dystrail::game::{GameState, CrossingConfig, CrossingKind, calculate_bribe_cost, can_use_permit, can_afford_bribe};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn create_test_game_state() -> GameState {
    let mut gs = GameState::default();
    gs.budget_cents = 10000; // $100
    gs.receipts.push("test_receipt".to_string());
    gs.inventory.tags.insert("press_pass".to_string());
    gs
}

#[wasm_bindgen_test]
fn calculate_bribe_cost_with_discount() {
    let base_cost = 1000_i64; // $10.00
    let discount_pct = 20_i32; // 20% discount

    let discounted_cost = calculate_bribe_cost(base_cost, discount_pct);

    // Should be 80% of original cost
    assert_eq!(discounted_cost, 800);
}

#[wasm_bindgen_test]
fn can_use_permit_logic() {
    let mut gs = create_test_game_state();
    let kind = CrossingKind::Checkpoint;

    // Should be able to use permit with receipt
    assert!(can_use_permit(&gs, &kind));

    // Remove receipt but keep tag
    gs.receipts.clear();
    assert!(can_use_permit(&gs, &kind));

    // Remove both - should fail
    gs.inventory.tags.clear();
    assert!(!can_use_permit(&gs, &kind));
}

#[wasm_bindgen_test]
fn can_afford_bribe_logic() {
    let mut gs = create_test_game_state();
    let cfg = CrossingConfig::default();

    // Should be able to afford with default budget
    assert!(can_afford_bribe(&gs, &cfg, CrossingKind::Checkpoint));

    // Remove budget - should fail
    gs.budget_cents = 0;
    assert!(!can_afford_bribe(&gs, &cfg, CrossingKind::Checkpoint));
}

#[wasm_bindgen_test]
fn crossing_config_loads_default() {
    let cfg = CrossingConfig::default();

    // Should have checkpoint type
    assert!(cfg.types.contains_key(&CrossingKind::Checkpoint));
    assert!(cfg.types.contains_key(&CrossingKind::BridgeOut));
}

#[wasm_bindgen_test]
fn digit1_activates_detour() {
    let gs = Rc::new(RefCell::new(create_test_game_state()));
    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    let initial_day = gs.borrow().day;
    let initial_supplies = gs.borrow().stats.supplies;

    dispatch_key(&region, "1", "Digit1");

    // Should apply detour effects
    assert_eq!(gs.borrow().day, initial_day + 2); // Default checkpoint detour: +2 days
    assert_eq!(gs.borrow().stats.supplies, initial_supplies - 2); // -2 supplies
}

#[wasm_bindgen_test]
fn digit2_activates_bribe() {
    let gs = Rc::new(RefCell::new(create_test_game_state()));
    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    let initial_budget = gs.borrow().budget_cents;

    dispatch_key(&region, "2", "Digit2");

    // Should deduct bribe cost (default $10.00 = 1000 cents)
    assert!(gs.borrow().budget_cents < initial_budget);
}

#[wasm_bindgen_test]
fn digit3_activates_permit() {
    let gs = Rc::new(RefCell::new(create_test_game_state()));
    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    let initial_cred = gs.borrow().stats.credibility;
    let initial_receipts = gs.borrow().receipts.len();

    dispatch_key(&region, "3", "Digit3");

    // Should use press_pass tag (no receipt consumed) and gain credibility
    assert_eq!(gs.borrow().stats.credibility, initial_cred + 1);
    assert_eq!(gs.borrow().receipts.len(), initial_receipts); // No receipt consumed
}

#[wasm_bindgen_test]
fn disabled_menuitem_has_aria_disabled() {
    let mut gs = create_test_game_state();
    gs.receipts.clear(); // Remove receipts
    gs.inventory.tags.clear(); // Remove permit tags
    let gs = Rc::new(RefCell::new(gs));

    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();
    let permit_item = doc.query_selector("li[data-key='3']").unwrap().expect("permit menuitem exists");

    // Should be disabled when no receipts or permit tags
    assert_eq!(permit_item.get_attribute("aria-disabled").unwrap(), "true");
}

#[wasm_bindgen_test]
fn arrow_keys_change_focus() {
    let gs = Rc::new(RefCell::new(create_test_game_state()));
    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    // Initial focus should be on item 1
    let item1 = doc.query_selector("li[data-key='1']").unwrap().expect("item 1 exists");
    assert_eq!(item1.get_attribute("tabindex").unwrap(), "0");

    // Arrow down should move to item 2
    dispatch_key(&region, "ArrowDown", "ArrowDown");

    // Need to re-query after state change
    let item1_after = doc.query_selector("li[data-key='1']").unwrap().expect("item 1 exists");
    let item2_after = doc.query_selector("li[data-key='2']").unwrap().expect("item 2 exists");

    assert_eq!(item1_after.get_attribute("tabindex").unwrap(), "-1");
    assert_eq!(item2_after.get_attribute("tabindex").unwrap(), "0");
}

#[wasm_bindgen_test]
fn rtl_locale_works() {
    // This test ensures RTL locales don't break the component structure
    dystrail::i18n::set_locale("ar"); // Set Arabic RTL locale

    let gs = Rc::new(RefCell::new(create_test_game_state()));
    let cfg = Rc::new(CrossingConfig::default());
    let on_resolved = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        gloo::utils::document().get_element_by_id("app").unwrap(),
        yew::Props::from(dystrail::components::ui::crossing_card::CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_resolved,
        })
    ).render();

    let doc = gloo::utils::document();

    // Component should still render properly with RTL locale
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");
    assert!(region.is_some());

    let menuitems = doc.query_selector_all("li[role='menuitem']").unwrap();
    assert_eq!(menuitems.length(), 4);

    // Reset locale
    dystrail::i18n::set_locale("en");
}
