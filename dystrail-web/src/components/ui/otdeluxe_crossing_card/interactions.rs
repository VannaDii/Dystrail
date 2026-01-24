use crate::game::OtDeluxeCrossingOptions;
#[cfg(target_arch = "wasm32")]
use crate::input::{numeric_code_to_index, numeric_key_to_index};
#[cfg(target_arch = "wasm32")]
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
        options: OtDeluxeCrossingOptions,
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
        let handler = activate_handler(on_choice, resolved, props.options);

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
    fn activate_handler_accepts_allowed_option() {
        let options = OtDeluxeCrossingOptions::empty().with_ford();
        let props = ActivateHarnessProps {
            options,
            attempt: 1,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"1\""));
        assert!(html.contains("data-resolved=\"true\""));
    }

    #[test]
    fn activate_handler_rejects_disabled_option() {
        let options = OtDeluxeCrossingOptions::empty().with_ford();
        let props = ActivateHarnessProps {
            options,
            attempt: 4,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"none\""));
        assert!(html.contains("data-resolved=\"false\""));
    }

    #[test]
    fn activate_handler_accepts_back_option() {
        let props = ActivateHarnessProps {
            options: OtDeluxeCrossingOptions::empty(),
            attempt: 0,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"0\""));
        assert!(html.contains("data-resolved=\"true\""));
    }

    #[test]
    fn activate_handler_accepts_caulk_float_option() {
        let props = ActivateHarnessProps {
            options: OtDeluxeCrossingOptions::empty().with_caulk_float(),
            attempt: 2,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"2\""));
        assert!(html.contains("data-resolved=\"true\""));
    }

    #[test]
    fn activate_handler_accepts_ferry_option() {
        let props = ActivateHarnessProps {
            options: OtDeluxeCrossingOptions::empty().with_ferry(),
            attempt: 3,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"3\""));
        assert!(html.contains("data-resolved=\"true\""));
    }

    #[test]
    fn activate_handler_rejects_unknown_option() {
        let props = ActivateHarnessProps {
            options: OtDeluxeCrossingOptions::empty(),
            attempt: 9,
        };
        let html = block_on(LocalServerRenderer::<ActivateHarness>::with_props(props).render());
        assert!(html.contains("data-selected=\"none\""));
        assert!(html.contains("data-resolved=\"false\""));
    }
}
