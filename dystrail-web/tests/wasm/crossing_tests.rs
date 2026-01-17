use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_test::*;
use web_sys::{Element, EventTarget, KeyboardEvent};
use yew::prelude::*;

use dystrail_web::components::ui::crossing_card::{CrossingCard, CrossingCardProps};
use dystrail_web::dom;
use dystrail_web::game::{
    CrossingConfig, CrossingKind, GameState, calculate_bribe_cost, can_afford_bribe, can_use_permit,
};
use dystrail_web::i18n;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn ensure_app_root() -> Element {
    let doc = dom::document().expect("document");
    if let Some(root) = doc.get_element_by_id("app") {
        return root;
    }
    let root = doc.create_element("div").expect("create app root");
    root.set_id("app");
    doc.body()
        .expect("document body")
        .append_child(&root)
        .expect("append app root");
    root
}

fn dispatch_key(el: &web_sys::Element, key: &str, code: &str) {
    let event = KeyboardEvent::new_with_keyboard_event_init_dict(
        "keydown",
        web_sys::KeyboardEventInit::new().key(key).code(code).bubbles(true).cancelable(true),
    )
    .unwrap();
    let target: EventTarget = el.clone().into();
    let _ = target.dispatch_event(&event);
}

fn capture_choice() -> (Callback<u8>, Rc<Cell<Option<u8>>>) {
    let last_choice = Rc::new(Cell::new(None));
    let captured = last_choice.clone();
    let callback = Callback::from(move |idx: u8| {
        captured.set(Some(idx));
    });
    (callback, last_choice)
}

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
    let gs = Rc::new(create_test_game_state());
    let cfg = Rc::new(CrossingConfig::default());
    let (on_choice, choice) = capture_choice();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    dispatch_key(&region, "1", "Digit1");

    assert_eq!(choice.get(), Some(1));
}

#[wasm_bindgen_test]
fn digit2_activates_bribe() {
    let gs = Rc::new(create_test_game_state());
    let cfg = Rc::new(CrossingConfig::default());
    let (on_choice, choice) = capture_choice();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    dispatch_key(&region, "2", "Digit2");

    assert_eq!(choice.get(), Some(2));
}

#[wasm_bindgen_test]
fn digit3_activates_permit() {
    let gs = Rc::new(create_test_game_state());
    let cfg = Rc::new(CrossingConfig::default());
    let (on_choice, choice) = capture_choice();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs.clone(),
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    dispatch_key(&region, "3", "Digit3");

    assert_eq!(choice.get(), Some(3));
}

#[wasm_bindgen_test]
fn disabled_menuitem_has_aria_disabled() {
    let mut gs = create_test_game_state();
    gs.receipts.clear(); // Remove receipts
    gs.inventory.tags.clear(); // Remove permit tags
    let gs = Rc::new(gs);

    let cfg = Rc::new(CrossingConfig::default());
    let (on_choice, choice) = capture_choice();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");
    let permit_item = doc.query_selector("li[data-key='3']").unwrap().expect("permit menuitem exists");
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");

    // Should be disabled when no receipts or permit tags
    assert_eq!(permit_item.get_attribute("aria-disabled").unwrap(), "true");
    dispatch_key(&region, "3", "Digit3");
    assert_eq!(choice.get(), None);
}

#[wasm_bindgen_test]
fn arrow_keys_change_focus() {
    let gs = Rc::new(create_test_game_state());
    let cfg = Rc::new(CrossingConfig::default());
    let on_choice = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");
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
    i18n::set_lang("ar"); // Set Arabic RTL locale

    let gs = Rc::new(create_test_game_state());
    let cfg = Rc::new(CrossingConfig::default());
    let on_choice = Callback::noop();

    yew::Renderer::<CrossingCard>::with_root_and_props(
        ensure_app_root(),
        yew::Props::from(CrossingCardProps {
            game_state: gs,
            config: cfg,
            kind: CrossingKind::Checkpoint,
            on_choice,
        }),
    )
    .render();

    let doc = dom::document().expect("document");

    // Component should still render properly with RTL locale
    let region = doc.query_selector("section[role='region']").unwrap().expect("region exists");
    assert!(region.has_attribute("role"));

    let menuitems = doc.query_selector_all("li[role='menuitem']").unwrap();
    assert_eq!(menuitems.length(), 4);

    // Reset locale
    i18n::set_lang("en");
}
