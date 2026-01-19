#[cfg(target_arch = "wasm32")]
use crate::input::{numeric_code_to_index, numeric_key_to_index};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn activate_handler(
    on_choice: Callback<u8>,
    resolved: UseStateHandle<bool>,
    bribe_available: bool,
    permit_available: bool,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if *resolved {
            return;
        }

        let allowed = match idx {
            0 | 1 => true,
            2 => bribe_available,
            3 => permit_available,
            _ => false,
        };
        if !allowed {
            return;
        }

        on_choice.emit(idx);
        resolved.set(true);
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

#[cfg(not(target_arch = "wasm32"))]
pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: &UseStateHandle<u8>,
    resolved: &UseStateHandle<bool>,
) -> Callback<KeyboardEvent> {
    let _ = (activate, focus_idx, resolved);
    Callback::from(|_e: KeyboardEvent| {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[derive(Properties, PartialEq)]
    struct ActivateHarnessProps {
        bribe_available: bool,
        permit_available: bool,
        attempt: u8,
    }

    #[function_component(ActivateHarness)]
    fn activate_harness(props: &ActivateHarnessProps) -> Html {
        let resolved = use_state(|| false);
        let selected = use_mut_ref(|| None::<u8>);
        let invoked = use_mut_ref(|| false);
        let on_choice = {
            let selected = selected.clone();
            Callback::from(move |idx| {
                *selected.borrow_mut() = Some(idx);
            })
        };
        let handler = activate_handler(
            on_choice,
            resolved,
            props.bribe_available,
            props.permit_available,
        );

        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            handler.emit(props.attempt);
        }

        let selected_value = *selected.borrow();
        let selected_label =
            selected_value.map_or_else(|| "none".to_string(), |idx| idx.to_string());
        let resolved_label = if selected_value.is_some() {
            "true"
        } else {
            "false"
        };
        html! {
            <div
                data-selected={selected_label}
                data-resolved={resolved_label}
            />
        }
    }

    #[test]
    fn activate_handler_accepts_allowed_choice() {
        let props = ActivateHarnessProps {
            bribe_available: true,
            permit_available: false,
            attempt: 2,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"2\""));
        assert!(html.contains("data-resolved=\"true\""));
    }

    #[test]
    fn activate_handler_blocks_unavailable_choice() {
        let props = ActivateHarnessProps {
            bribe_available: false,
            permit_available: false,
            attempt: 2,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"none\""));
        assert!(html.contains("data-resolved=\"false\""));
    }
}
