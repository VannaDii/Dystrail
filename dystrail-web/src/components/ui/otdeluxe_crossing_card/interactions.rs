use crate::game::OtDeluxeCrossingOptions;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn activate_handler(
    on_choice: Callback<u8>,
    resolved: UseStateHandle<bool>,
    options: OtDeluxeCrossingOptions,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if *resolved {
            return;
        }

        let allowed = match idx {
            0 => true,
            1 => options.ford(),
            2 => options.caulk_float(),
            3 => options.ferry(),
            4 => options.guide(),
            _ => false,
        };
        if !allowed {
            return;
        }

        on_choice.emit(idx);
        resolved.set(true);
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
        } else if key == "Escape" {
            activate.emit(0);
            e.prevent_default();
        } else if key == "ArrowDown" {
            let next = match *focus_idx {
                1 => 2,
                2 => 3,
                3 => 4,
                4 => 0,
                _ => 1,
            };
            focus_idx.set(next);
            e.prevent_default();
        } else if key == "ArrowUp" {
            let prev = match *focus_idx {
                0 => 4,
                1 => 0,
                2 => 1,
                3 => 2,
                _ => 3,
            };
            focus_idx.set(prev);
            e.prevent_default();
        }
    })
}
