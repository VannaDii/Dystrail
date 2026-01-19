#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::hook;
use yew::prelude::*;

#[cfg(target_arch = "wasm32")]
const FOCUSABLE_QUERY: &str =
    "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])";

#[cfg(target_arch = "wasm32")]
#[hook]
pub fn use_focus_trap(open: bool, return_focus_id: Option<AttrValue>, container_ref: NodeRef) {
    use_effect_with(
        (open, return_focus_id, container_ref),
        move |(open, ret, container_ref)| {
            let focus_target = if *open {
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

            let ret_id = ret.clone();
            move || {
                let maybe_focus = ret_id
                    .clone()
                    .and_then(|id| {
                        web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|doc| doc.get_element_by_id(id.as_ref()))
                    })
                    .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok());

                if let Some(el) = maybe_focus {
                    let _ = el.focus();
                }
            }
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[hook]
pub fn use_focus_trap(open: bool, return_focus_id: Option<AttrValue>, container_ref: NodeRef) {
    let _ = (open, return_focus_id, container_ref);
}

#[cfg(target_arch = "wasm32")]
pub fn focus_keydown_handler(
    container_ref: &NodeRef,
    on_close: Callback<()>,
) -> Callback<KeyboardEvent> {
    let container_ref = container_ref.clone();
    Callback::from(move |e: KeyboardEvent| {
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

#[cfg(not(target_arch = "wasm32"))]
pub fn focus_keydown_handler(
    container_ref: &NodeRef,
    on_close: Callback<()>,
) -> Callback<KeyboardEvent> {
    let _ = container_ref;
    Callback::from(move |_e: KeyboardEvent| {
        let _ = &on_close;
    })
}
