use crate::a11y::set_status;
use crate::i18n;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn activate_handler(on_select: Option<Callback<u8>>) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        let label_key = match idx {
            1 => "menu.start_journey",
            2 => "menu.about",
            3 => "menu.accessibility",
            4 => "menu.quit",
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

#[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
pub fn focus_effect(list_ref: NodeRef, focus_idx: &UseStateHandle<u8>) {
    let _ = (list_ref, focus_idx);
}

#[cfg(target_arch = "wasm32")]
pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: UseStateHandle<u8>,
) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        let key = e.key();
        if key == "Enter" || key == " " {
            activate.emit(*focus_idx);
            e.prevent_default();
        } else if key == "ArrowDown" {
            let mut next = *focus_idx + 1;
            if next > 4 {
                next = 1;
            }
            focus_idx.set(next);
            e.prevent_default();
        } else if key == "ArrowUp" {
            let mut prev = if *focus_idx <= 1 { 4 } else { *focus_idx - 1 };
            if prev < 1 {
                prev = 4;
            }
            focus_idx.set(prev);
            e.prevent_default();
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: UseStateHandle<u8>,
) -> Callback<KeyboardEvent> {
    let _ = (activate, focus_idx);
    Callback::from(|_e: KeyboardEvent| {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn activate_handler_emits_selection() {
        crate::i18n::set_lang("en");
        let selected = Rc::new(RefCell::new(None));
        let selected_handle = selected.clone();
        let on_select = Callback::from(move |idx| {
            *selected_handle.borrow_mut() = Some(idx);
        });

        let handler = activate_handler(Some(on_select));
        handler.emit(2);

        assert_eq!(*selected.borrow(), Some(2));
    }

    #[test]
    fn activate_handler_handles_missing_callback() {
        crate::i18n::set_lang("en");
        let handler = activate_handler(None);
        handler.emit(1);
    }

    #[test]
    fn activate_handler_handles_quit_and_unknown() {
        crate::i18n::set_lang("en");
        let selected = Rc::new(RefCell::new(None));
        let selected_handle = selected.clone();
        let handler = activate_handler(Some(Callback::from(move |idx| {
            *selected_handle.borrow_mut() = Some(idx);
        })));
        handler.emit(4);
        assert_eq!(*selected.borrow(), Some(4));

        let handler = activate_handler(None);
        handler.emit(42);
    }

    #[test]
    fn activate_handler_covers_remaining_labels() {
        crate::i18n::set_lang("en");
        let handler = activate_handler(None);
        for idx in 1..=4 {
            handler.emit(idx);
        }
    }
}
