use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::hook;
use yew::prelude::*;

const FOCUSABLE_QUERY: &str = "button:not([disabled]), [href], input:not([disabled]), textarea:not([disabled]), select:not([disabled]), summary, [tabindex]:not([tabindex='-1'])";

fn focusable_elements(container: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    let Ok(nodes) = container.query_selector_all(FOCUSABLE_QUERY) else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|index| nodes.get(index)?.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|element| element.offset_width() > 0 || element.offset_height() > 0)
        .filter(|element| {
            element
                .closest("details:not([open]) > :not(summary)")
                .ok()
                .flatten()
                .is_none()
        })
        .collect()
}

#[hook]
pub fn use_focus_trap(open: bool, return_focus_id: Option<AttrValue>, container_ref: NodeRef) {
    use_effect_with(
        (open, return_focus_id, container_ref),
        move |(open, ret, container_ref)| {
            let focus_target = if cfg!(target_arch = "wasm32") && *open {
                container_ref
                    .cast::<web_sys::Element>()
                    .and_then(|element| focusable_elements(&element).first().cloned())
            } else {
                None
            };

            if let Some(first) = focus_target {
                let _ = first.focus();
            }

            let was_open = *open;
            let ret_id = ret.clone();
            move || {
                let maybe_focus = if cfg!(target_arch = "wasm32") && was_open {
                    ret_id
                        .clone()
                        .and_then(|id| {
                            web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|doc| doc.get_element_by_id(id.as_ref()))
                        })
                        .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                } else {
                    None
                };

                if let Some(el) = maybe_focus {
                    let _ = el.focus();
                }
            }
        },
    );
}

pub fn focus_keydown_handler(
    container_ref: NodeRef,
    on_close: Callback<()>,
) -> Callback<KeyboardEvent> {
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
        let nodes = focusable_elements(&container);
        let first = nodes.first().cloned();
        let last = nodes.last().cloned();
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
