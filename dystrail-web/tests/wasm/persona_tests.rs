use wasm_bindgen_test::*;
use web_sys::{Element, KeyboardEvent, EventTarget};
use yew::prelude::*;

use dystrail_web::components::ui::persona_select::PersonaSelect;
use dystrail_web::dom;
use dystrail_web::game::{personas::Persona, state::GameState};

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

#[function_component(TestHost)]
fn test_host() -> Html {
    let state = use_state(GameState::default);
    let on_selected = {
        let state = state.clone();
        Callback::from(move |per: Persona| {
            let mut gs = (*state).clone();
            gs.apply_persona(&per);
            state.set(gs);
        })
    };
    html! { <PersonaSelect on_selected={Some(on_selected)} /> }
}

#[wasm_bindgen_test]
fn persona_roles_and_live_region() {
    yew::Renderer::<TestHost>::with_root(ensure_app_root()).render();
    let doc = dom::document().expect("document");
    let radios = doc.query_selector("[role='radiogroup']").unwrap();
    assert!(radios.is_some());
    let live = doc.get_element_by_id("persona-helper").expect("live region present");
    assert_eq!(live.get_attribute("aria-live").unwrap(), "polite");
}

#[wasm_bindgen_test]
fn key3_selects_and_updates_live() {
    yew::Renderer::<TestHost>::with_root(ensure_app_root()).render();
    let doc = dom::document().expect("document");
    let panel = doc.query_selector("section.panel").unwrap().unwrap();
    dispatch_key(&panel, "3", "Digit3");
    let live = doc.get_element_by_id("persona-helper").unwrap();
    let text = live.text_content().unwrap_or_default();
    assert!(text.contains("Selected") || text.contains("مبلغ") || text.len() > 0);
    // aria-checked flips
    let third = doc.query_selector("[role='radio'][data-key='3']").unwrap().unwrap();
    assert_eq!(third.get_attribute("aria-checked"), Some("true".into()));
    // Saved persona_id should be whistleblower
    let win = web_sys::window().unwrap();
    let storage = win.local_storage().unwrap().unwrap();
    let saved = storage.get_item("dystrail.save").unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(v.get("persona_id").and_then(|x| x.as_str()), Some("whistleblower"));
}

#[wasm_bindgen_test]
fn continue_disabled_until_selection() {
    yew::Renderer::<TestHost>::with_root(ensure_app_root()).render();
    let doc = dom::document().expect("document");
    let btn = doc.get_element_by_id("persona-continue").unwrap();
    assert!(btn.get_attribute("disabled").is_some());
    // Select 1
    let panel = doc.query_selector("section.panel").unwrap().unwrap();
    dispatch_key(&panel, "1", "Digit1");
    let btn2 = doc.get_element_by_id("persona-continue").unwrap();
    assert!(btn2.get_attribute("disabled").is_none());
}

#[wasm_bindgen_test]
fn selection_persists_to_save() {
    yew::Renderer::<TestHost>::with_root(ensure_app_root()).render();
    let doc = dom::document().expect("document");
    // Select 2 to ensure a deterministic, different pick
    let panel = doc.query_selector("section.panel").unwrap().unwrap();
    dispatch_key(&panel, "2", "Digit2");
    // Read localStorage
    let win = web_sys::window().unwrap();
    let storage = win.local_storage().unwrap().unwrap();
    let saved = storage.get_item("dystrail.save").unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&saved).unwrap();
    assert!(v.get("persona_id").is_some());
    assert!(v.get("budget").is_some());
}
