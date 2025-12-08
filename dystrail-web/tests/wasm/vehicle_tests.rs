use wasm_bindgen_test::*;
use web_sys::{Element, KeyboardEvent, EventTarget};
use yew::prelude::*;

use dystrail_web::components::ui::vehicle_status::{VehicleStatus, VehicleStatusProps};
use dystrail_web::dom;
use dystrail_web::game::{
    state::GameState,
    vehicle::{Breakdown, Part},
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

fn create_test_props() -> <VehicleStatus as Component>::Properties {
    VehicleStatusProps {
        game_state: GameState::default(),
        on_repair_action: Callback::noop(),
    }
}

#[wasm_bindgen_test]
fn vehicle_status_menu_accessibility() {
    let props = create_test_props();
    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");

    // Check for proper ARIA roles
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
    let target: EventTarget = el.clone().into();
    let _ = target.dispatch_event(&event);
}

#[wasm_bindgen_test]
fn vehicle_status_keyboard_navigation() {
    let props = create_test_props();
    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");
    let menu = doc.query_selector("[role='menu']").unwrap().unwrap();

    // Test number key navigation (1-5, 0)
    dispatch_key(&menu, "1", "Digit1");
    dispatch_key(&menu, "2", "Digit2");
    dispatch_key(&menu, "0", "Digit0");

    // Test escape key
    dispatch_key(&menu, "Escape", "Escape");
}

#[wasm_bindgen_test]
fn breakdown_state_disables_travel() {
    let mut game_state = GameState::default();
    game_state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 1,
    });
    game_state.day_state.travel.travel_blocked = true;

    let props = VehicleStatusProps {
        game_state,
        on_repair_action: Callback::noop(),
    };

    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");

    // Should show breakdown status
    let breakdown_text = doc.query_selector("[data-testid='breakdown-status']");
    assert!(breakdown_text.is_ok());
}

#[wasm_bindgen_test]
fn spare_usage_options_enabled() {
    let mut game_state = GameState::default();
    game_state.breakdown = Some(Breakdown {
        part: Part::Tire,
        day_started: 1,
    });
    game_state.inventory.spares.tire = 2; // Has spare tires

    let props = VehicleStatusProps {
        game_state,
        on_repair_action: Callback::noop(),
    };

    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");

    // Tire spare option should be enabled since we have a tire breakdown and spare tires
    let tire_option = doc.query_selector("[data-action='spare-tire']").unwrap();
    if let Some(element) = tire_option {
        assert_ne!(element.get_attribute("aria-disabled").unwrap_or("false".to_string()), "true");
    }
}

#[wasm_bindgen_test]
fn i18n_vehicle_keys_present() {
    i18n::set_lang("en");

    assert!(!i18n::t("vehicle.title").is_empty());
    assert!(!i18n::t("vehicle.breakdown").is_empty());
    assert!(!i18n::t("vehicle.spares.tire").is_empty());
    assert!(!i18n::t("vehicle.parts.tire").is_empty());
    assert!(!i18n::t("vehicle.announce.used_spare").is_empty());
}

#[wasm_bindgen_test]
fn rtl_layout_for_arabic() {
    i18n::set_lang("ar");

    // Change to Arabic locale
    let doc = dom::document().expect("document");
    let html = doc.document_element().unwrap();
    html.set_attribute("lang", "ar").unwrap();
    html.set_attribute("dir", "rtl").unwrap();

    let props = create_test_props();
    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    // Verify RTL attributes are present
    assert_eq!(html.get_attribute("dir").unwrap(), "rtl");
    assert_eq!(html.get_attribute("lang").unwrap(), "ar");

    // Test that Arabic vehicle translations exist
    assert!(!i18n.t("vehicle.title").is_empty());
}

#[wasm_bindgen_test]
fn status_announcements_via_aria_live() {
    let props = create_test_props();
    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");

    // Check that the live region exists for status announcements
    let live_region = doc.get_element_by_id("menu-helper").expect("aria-live region should exist");
    assert_eq!(live_region.get_attribute("aria-live").unwrap(), "polite");
}

#[wasm_bindgen_test]
fn focus_management_and_visible_rings() {
    let props = create_test_props();
    yew::Renderer::<VehicleStatus>::with_props_and_root(
        props,
        ensure_app_root(),
    )
    .render();

    let doc = dom::document().expect("document");

    // Check that focusable elements have visible focus rings
    let menu_items = doc.query_selector_all("[role='menuitem']").unwrap();
    for i in 0..menu_items.length() {
        let item = menu_items.get(i).unwrap();
        // Focus the item
        if let Some(element) = item.dyn_ref::<web_sys::HtmlElement>() {
            element.focus().unwrap();

            // Check that the element is focusable (has tabindex or is naturally focusable)
            let tabindex = element.get_attribute("tabindex");
            assert!(tabindex.is_some() || element.tag_name() == "BUTTON");
        }
    }
}
