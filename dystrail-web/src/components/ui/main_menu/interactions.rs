use crate::a11y::set_status;
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn activate_handler(on_select: Option<Callback<u8>>) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        let label_key = match idx {
            1 => "menu.travel",
            2 => "menu.camp",
            3 => "menu.status",
            4 => "menu.pace",
            5 => "menu.diet",
            6 => "menu.inventory",
            7 => "menu.share",
            8 => "menu.settings",
            0 => "menu.quit",
            _ => "",
        };
        let label = i18n::t(label_key);
        let msg = format!("{selected} {label}", selected = i18n::t("menu.selected"));
        set_status(&msg);
        if let Some(cb) = on_select.clone() {
            cb.emit(idx);
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
    focus_idx: UseStateHandle<u8>,
) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
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
            e.prevent_default();
        } else if key == "ArrowDown" {
            let mut next = *focus_idx + 1;
            if next > 8 {
                next = 0;
            }
            focus_idx.set(next);
            e.prevent_default();
        } else if key == "ArrowUp" {
            let mut prev = if *focus_idx == 0 { 8 } else { *focus_idx - 1 };
            if prev == 0 {
                prev = 8;
            }
            focus_idx.set(prev);
            e.prevent_default();
        }
    })
}
