use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::game::OtDeluxeRouteDecision;

pub fn activate_handler(
    on_choice: Callback<OtDeluxeRouteDecision>,
    resolved: UseStateHandle<bool>,
    primary: OtDeluxeRouteDecision,
    secondary: OtDeluxeRouteDecision,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if *resolved {
            return;
        }

        let choice = match idx {
            1 => Some(primary),
            2 => Some(secondary),
            _ => None,
        };
        if let Some(choice) = choice {
            on_choice.emit(choice);
            resolved.set(true);
        }
    })
}

pub fn focus_effect(list_ref: NodeRef, focus_idx: &UseStateHandle<u8>) {
    let focus_idx = focus_idx.clone();
    use_effect_with(*focus_idx, move |idx| {
        if let Some(list) = list_ref.cast::<web_sys::Element>() {
            let sel = format!("[role='menuitem'][data-key='{idx}']");
            if let Ok(Some(el)) = list.query_selector(&sel) {
                let _ = el
                    .dyn_into::<web_sys::HtmlElement>()
                    .ok()
                    .map(|e| e.focus());
            }
        }
    });
}

pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: &UseStateHandle<u8>,
    resolved: &UseStateHandle<bool>,
) -> Callback<KeyboardEvent> {
    let focus_idx = focus_idx.clone();
    let resolved = resolved.clone();
    Callback::from(move |e: KeyboardEvent| {
        if *resolved {
            return;
        }

        let key = e.key();

        if let Some(n) = numeric_key_to_index(&key) {
            activate.emit(n);
            e.prevent_default();
            return;
        }
        if let Some(n) = numeric_code_to_index(&e.code()) {
            activate.emit(n);
            e.prevent_default();
            return;
        }
        if key == "Enter" || key == " " {
            activate.emit(*focus_idx);
            e.prevent_default();
        } else if key == "ArrowDown" || key == "ArrowUp" {
            let next = if *focus_idx == 1 { 2 } else { 1 };
            focus_idx.set(next);
            e.prevent_default();
        }
    })
}
