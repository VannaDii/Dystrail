use wasm_bindgen_test::*;
use web_sys::{KeyboardEvent, EventTarget};
use yew::prelude::*;
use dystrail_web::dom;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn menu_roles_and_aria_live_present() {
    yew::Renderer::<crate::components::ui::main_menu::MainMenu>::with_root(
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();
    let doc = dom::document();
    // Expect main menu container and live region
    assert!(doc.get_element_by_id("main-menu").is_some());
    let helper = doc.get_element_by_id("menu-helper").expect("live region present");
    assert_eq!(helper.get_attribute("aria-live").unwrap(), "polite");
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
fn digit3_triggers_status_update() {
    yew::Renderer::<crate::components::ui::main_menu::MainMenu>::with_root(
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();
    let doc = dom::document();
    // keydown on region element
    let region = doc
        .query_selector("section[role='region']")
        .unwrap()
        .expect("region exists");
    dispatch_key(&region, "3", "Digit3");
    let helper = doc.get_element_by_id("menu-helper").unwrap();
    let text = helper.text_content().unwrap_or_default();
    assert!(text.contains("3") || text.contains("Status") || text.contains("Selected"));
}

#[wasm_bindgen_test]
fn roving_tabindex_moves_with_arrows() {
    yew::Renderer::<crate::components::ui::main_menu::MainMenu>::with_root(
        dom::document().get_element_by_id("app").unwrap(),
    )
    .render();
    let doc = dom::document();
    let region = doc
        .query_selector("section[role='region']")
        .unwrap()
        .expect("region exists");
    dispatch_key(&region, "ArrowDown", "ArrowDown");
    // One item should have tabindex=0
    let focused = doc.query_selector_all("#main-menu [role='menuitem'][tabindex='0']").unwrap();
    assert_eq!(focused.length(), 1);
}

#[wasm_bindgen_test]
fn esc_closes_settings_dialog() {
    // Mount the settings dialog open
    #[function_component(TestHost)]
    fn test_host() -> Html {
        let open = use_state(|| true);
        let on_close = { let open = open.clone(); Callback::from(move |_| open.set(false)) };
        html!{ <crate::components::ui::settings_dialog::SettingsDialog open={*open} on_close={on_close} /> }
    }
    yew::Renderer::<TestHost>::with_root(dom::document().get_element_by_id("app").unwrap()).render();
    let doc = dom::document();
    let dlg = doc.query_selector(".drawer").unwrap();
    assert!(dlg.is_some());
    let root = doc.query_selector(".drawer").unwrap().unwrap();
    let event = KeyboardEvent::new_with_keyboard_event_init_dict(
        "keydown",
        web_sys::KeyboardEventInit::new().key("Escape").code("Escape").bubbles(true).cancelable(true),
    ).unwrap();
    let target: EventTarget = root.clone().into();
    let _ = target.dispatch_event(&event);
    // After ESC the dialog should unmount
    let dlg_after = doc.query_selector(".drawer").unwrap();
    assert!(dlg_after.is_none());
}

#[wasm_bindgen_test]
fn rtl_dir_applies_on_locale_switch() {
    crate::i18n::set_lang("ar");
    let doc = dom::document();
    let html = doc.document_element().unwrap();
    assert_eq!(html.get_attribute("dir"), Some("rtl".into()));
    crate::i18n::set_lang("en");
    let html2 = dom::document().document_element().unwrap();
    assert_eq!(html2.get_attribute("dir"), Some("ltr".into()));
}
