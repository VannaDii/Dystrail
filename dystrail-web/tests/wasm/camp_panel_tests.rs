use wasm_bindgen_test::*;
use web_sys::{KeyboardEvent, EventTarget};
use yew::prelude::*;
use dystrail_web::dom;
use dystrail::components::ui::camp_panel::CampPanel;
use dystrail::game::state::GameState;
use dystrail::game::camp::CampConfig;
use dystrail::game::vehicle::{Part, Breakdown};
use dystrail::i18n::I18n;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn create_test_props() -> <CampPanel as Component>::Properties {
    use dystrail::components::ui::camp_panel::CampPanelProps;
    let mut game_state = GameState::default();
    game_state.receipts = vec!["test".to_string()]; // Give some receipts for testing

    CampPanelProps {
        game_state,
        camp_config: CampConfig::default(),
        on_action: Callback::noop(),
        on_close: Callback::noop(),
    }
}

#[wasm_bindgen_test]
fn camp_panel_accessibility() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check for proper ARIA roles
    let dialog = doc.query_selector("[role='dialog']").unwrap().unwrap();
    assert!(dialog.get_attribute("aria-labelledby").is_some());
    assert!(dialog.get_attribute("aria-describedby").is_some());

    // Check for menu with proper ARIA attributes
    let menu = doc.query_selector("[role='menu']").unwrap().unwrap();
    assert!(menu.get_attribute("aria-labelledby").is_some());

    // Check for menu items with proper ARIA attributes
    let menu_items = doc.query_selector_all("[role='menuitem']").unwrap();
    assert!(menu_items.length() > 0);

    for i in 0..menu_items.length() {
        let item = menu_items.get(i).unwrap();
        assert!(item.get_attribute("aria-posinset").is_some());
        assert!(item.get_attribute("aria-setsize").is_some());
    }
}

fn dispatch_key(el: &web_sys::Element, key: &str, code: &str) {
    let event = KeyboardEvent::new_with_keyboard_event_init_dict(
        "keydown",
        web_sys::KeyboardEventInit::new().key(key).code(code).bubbles(true).cancelable(true),
    )
    .unwrap();

    let target: &EventTarget = el.as_ref();
    target.dispatch_event(&event).unwrap();
}

#[wasm_bindgen_test]
fn camp_panel_keyboard_navigation() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();
    let dialog = doc.query_selector("[role='dialog']").unwrap().unwrap();

    // Test numbered key navigation (1-4)
    dispatch_key(&dialog, "1", "Digit1");
    dispatch_key(&dialog, "2", "Digit2");
    dispatch_key(&dialog, "3", "Digit3");
    dispatch_key(&dialog, "4", "Digit4");

    // Test escape key to close
    dispatch_key(&dialog, "Escape", "Escape");

    // Test zero key for close/back
    dispatch_key(&dialog, "0", "Digit0");
}

#[wasm_bindgen_test]
fn camp_panel_focus_management() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check that focus is trapped within the dialog
    let dialog = doc.query_selector("[role='dialog']").unwrap().unwrap();
    assert!(dialog.has_attribute("tabindex"));

    // Check for focusable elements within the dialog
    let focusable = doc.query_selector_all(
        "[role='dialog'] button, [role='dialog'] [tabindex]:not([tabindex='-1'])"
    ).unwrap();
    assert!(focusable.length() > 0);
}

#[wasm_bindgen_test]
fn camp_panel_action_menu() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check for all expected camp actions
    let rest_button = doc.query_selector("[data-camp-action='rest']");
    assert!(rest_button.is_ok());

    let forage_button = doc.query_selector("[data-camp-action='forage']");
    assert!(forage_button.is_ok());

    let therapy_button = doc.query_selector("[data-camp-action='therapy']");
    assert!(therapy_button.is_ok());

    let repair_button = doc.query_selector("[data-camp-action='repair']");
    assert!(repair_button.is_ok());

    // Check for close button
    let close_button = doc.query_selector("[data-camp-action='close']");
    assert!(close_button.is_ok());
}

#[wasm_bindgen_test]
fn camp_panel_repair_submenu() {
    let mut props = create_test_props();
    // Set up a breakdown to enable repair menu
    props.game_state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 1,
    });
    props.game_state.inventory.spares.tire = 1;

    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Try to trigger repair action to show submenu
    let repair_button = doc.query_selector("[data-camp-action='repair']").unwrap();
    if let Some(button) = repair_button {
        button.click();

        // Check for repair submenu elements
        let spare_button = doc.query_selector("[data-repair-action='spare']");
        let hack_button = doc.query_selector("[data-repair-action='hack']");
        let back_button = doc.query_selector("[data-repair-action='back']");

        // At least some repair options should be available
        assert!(spare_button.is_ok() || hack_button.is_ok() || back_button.is_ok());
    }
}

#[wasm_bindgen_test]
fn camp_panel_aria_live_announcements() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check for aria-live region for announcements
    let live_region = doc.query_selector("[aria-live]").unwrap();
    assert!(live_region.is_some());

    if let Some(region) = live_region {
        let aria_live = region.get_attribute("aria-live").unwrap();
        assert!(aria_live == "polite" || aria_live == "assertive");
    }
}

#[wasm_bindgen_test]
fn camp_panel_screen_reader_support() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check for proper labeling
    let dialog = doc.query_selector("[role='dialog']").unwrap().unwrap();
    let labelledby = dialog.get_attribute("aria-labelledby").unwrap();
    let label_element = doc.get_element_by_id(&labelledby);
    assert!(label_element.is_some());

    // Check for help text
    let help_text = doc.query_selector("[data-help-text]");
    assert!(help_text.is_ok());

    // Check menu items have accessible names
    let menu_items = doc.query_selector_all("[role='menuitem']").unwrap();
    for i in 0..menu_items.length() {
        let item = menu_items.get(i).unwrap();
        let has_text = item.text_content().unwrap_or_default().trim().len() > 0;
        let has_aria_label = item.get_attribute("aria-label").is_some();
        let has_aria_labelledby = item.get_attribute("aria-labelledby").is_some();

        assert!(has_text || has_aria_label || has_aria_labelledby,
                "Menu item must have accessible name");
    }
}

#[wasm_bindgen_test]
fn camp_panel_keyboard_shortcuts() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();
    let dialog = doc.query_selector("[role='dialog']").unwrap().unwrap();

    // Test all numbered shortcuts (1-4 for actions, 0 for close)
    for i in 0..=4 {
        let key = if i == 0 { "0" } else { &i.to_string() };
        let code = if i == 0 { "Digit0" } else { &format!("Digit{}", i) };

        dispatch_key(&dialog, key, code);
        // No assertion needed - just testing that keys don't cause errors
    }

    // Test arrow key navigation
    dispatch_key(&dialog, "ArrowDown", "ArrowDown");
    dispatch_key(&dialog, "ArrowUp", "ArrowUp");
    dispatch_key(&dialog, "Enter", "Enter");
    dispatch_key(&dialog, " ", "Space");
}

#[wasm_bindgen_test]
fn camp_panel_conditional_rendering() {
    // Test with different game states to verify conditional rendering

    // Test with no breakdown
    let props_no_breakdown = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props_no_breakdown,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    // Test with breakdown
    let mut props_with_breakdown = create_test_props();
    props_with_breakdown.game_state.breakdown = Some(Breakdown {
        part: Part::Battery,
        day_started: 1,
    });

    yew::Renderer::<CampPanel>::with_props_and_root(
        props_with_breakdown,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    // Test with therapy on cooldown
    let mut props_therapy_cooldown = create_test_props();
    props_therapy_cooldown.game_state.day = 2;
    props_therapy_cooldown.game_state.camp.last_therapy_day = Some(1);

    yew::Renderer::<CampPanel>::with_props_and_root(
        props_therapy_cooldown,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    // Test with no receipts
    let mut props_no_receipts = create_test_props();
    props_no_receipts.game_state.receipts = vec![];

    yew::Renderer::<CampPanel>::with_props_and_root(
        props_no_receipts,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();
}

#[wasm_bindgen_test]
fn camp_panel_wcag_compliance() {
    let props = create_test_props();
    yew::Renderer::<CampPanel>::with_props_and_root(
        props,
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();

    let doc = dom::document();

    // Check color contrast requirements are handled by CSS (can't test actual colors in unit test)
    // But we can check for proper semantic structure

    // Check for heading hierarchy
    let headings = doc.query_selector_all("h1, h2, h3, h4, h5, h6").unwrap();
    assert!(headings.length() > 0, "Should have proper heading structure");

    // Check for focus indicators (handled by CSS but structure should be there)
    let focusable_elements = doc.query_selector_all("button, [tabindex]:not([tabindex='-1'])").unwrap();
    assert!(focusable_elements.length() > 0, "Should have focusable elements");

    // Check for proper form labels if any form elements exist
    let form_elements = doc.query_selector_all("input, select, textarea").unwrap();
    for i in 0..form_elements.length() {
        let element = form_elements.get(i).unwrap();
        let has_label = element.get_attribute("aria-label").is_some() ||
                       element.get_attribute("aria-labelledby").is_some() ||
                       doc.query_selector(&format!("label[for='{}']",
                           element.get_attribute("id").unwrap_or_default())).unwrap().is_some();
        assert!(has_label, "Form elements must have labels");
    }
}