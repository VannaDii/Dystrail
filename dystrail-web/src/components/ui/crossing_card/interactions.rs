use super::view_model::apply_choice;
use crate::a11y::set_status;
use crate::dom;
use crate::game::{CrossingConfig, CrossingKind, GameState};
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn activate_handler(
    game_state: Rc<RefCell<GameState>>,
    config: Rc<CrossingConfig>,
    kind: CrossingKind,
    on_resolved: Callback<()>,
    resolved: UseStateHandle<bool>,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if *resolved {
            return;
        }

        let mut gs = game_state.borrow_mut();
        let result_msg = apply_choice(idx, &mut gs, &config, kind);
        drop(gs);

        if idx == 0 {
            on_resolved.emit(());
        }

        set_status(&result_msg);
        resolved.set(true);

        let on_resolved = on_resolved.clone();
        let timeout = Closure::once(move || {
            on_resolved.emit(());
        });
        if let Some(win) = dom::window() {
            if let Err(err) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout.as_ref().unchecked_ref(),
                1000,
            ) {
                dom::console_error(&format!(
                    "Failed to delay crossing transition: {}",
                    dom::js_error_message(&err)
                ));
            }
        } else {
            dom::console_error("Failed to delay crossing transition: window unavailable");
        }
        timeout.forget();
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
                3 => 0,
                _ => 1,
            };
            focus_idx.set(next);
            e.prevent_default();
        } else if key == "ArrowUp" {
            let prev = match *focus_idx {
                0 => 3,
                1 => 0,
                3 => 2,
                _ => 1,
            };
            focus_idx.set(prev);
            e.prevent_default();
        }
    })
}
