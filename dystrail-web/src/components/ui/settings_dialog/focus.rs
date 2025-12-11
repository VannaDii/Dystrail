use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::hook;
use yew::prelude::*;

const FOCUSABLE_QUERY: &str =
    "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])";

#[hook]
pub fn use_focus_management(open: bool, container_ref: NodeRef) {
    use_effect_with((open, container_ref), move |(open, container_ref)| {
        let mut prev_focus: Option<web_sys::HtmlElement> = None;
        let focus_target = if cfg!(target_arch = "wasm32") && *open {
            prev_focus = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|doc| {
                    doc.active_element()
                        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                });

            container_ref.cast::<web_sys::Element>().and_then(|el| {
                el.query_selector_all(FOCUSABLE_QUERY)
                    .ok()
                    .and_then(|list| {
                        list.get(0)
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                    })
            })
        } else {
            None
        };

        if let Some(first) = focus_target {
            let _ = first.focus();
        }
        move || {
            if let Some(el) = prev_focus {
                let _ = el.focus();
            }
        }
    });
}

pub fn keydown_handler(container_ref: NodeRef, on_close: Callback<()>) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        if !cfg!(target_arch = "wasm32") {
            let _ = e;
            return;
        }
        if e.key() == "Escape" {
            on_close.emit(());
            return;
        }
        if e.key() != "Tab" {
            return;
        }
        let Some(container) = container_ref.cast::<web_sys::Element>() else {
            return;
        };
        let Ok(nodes) = container.query_selector_all(FOCUSABLE_QUERY) else {
            return;
        };
        let len = nodes.length();
        if len == 0 {
            return;
        }
        let first = nodes
            .get(0)
            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
        let last = nodes
            .get(len - 1)
            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());
        let shift = e.shift_key();
        if let (Some(first), Some(last), Some(active)) = (first, last, active) {
            let first_el: web_sys::Element = first.clone().unchecked_into();
            let last_el: web_sys::Element = last.clone().unchecked_into();
            let is_first = active == first_el;
            let is_last = active == last_el;
            if !container.contains(Some(&active)) {
                e.prevent_default();
                let _ = first.focus();
                return;
            }
            if shift && is_first {
                e.prevent_default();
                let _ = last.focus();
            } else if !shift && is_last {
                e.prevent_default();
                let _ = first.focus();
            }
        }
    })
}
