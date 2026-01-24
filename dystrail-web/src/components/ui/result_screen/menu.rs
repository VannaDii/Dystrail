use yew::prelude::*;

pub(super) fn render_menu_item(
    current_focus: u8,
    index: u8,
    label: &str,
    on_action: &Callback<u8>,
) -> Html {
    let is_focused = current_focus == index;
    let tabindex = if is_focused { "0" } else { "-1" };
    let action_callback = {
        let on_action = on_action.clone();
        Callback::from(move |_: MouseEvent| emit_menu_action(&on_action, index))
    };

    let display_index = if index == 0 { "0" } else { &index.to_string() };

    html! {
        <li
            role="menuitem"
            tabindex={tabindex}
            class={classes!("menu-item", if is_focused { Some("focused") } else { None })}
            onclick={action_callback}
            data-index={index.to_string()}
            aria-label={format!("{display_index} {label}")}
        >
            { format!("{display_index}) {label}") }
        </li>
    }
}

fn emit_menu_action(on_action: &Callback<u8>, index: u8) {
    on_action.emit(index);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn handle_keyboard(
    current_focus: u8,
    event: &KeyboardEvent,
    on_action: &Callback<u8>,
) -> u8 {
    let key = event.key();

    if let Some(action) = parse_numeric_key(&key) {
        event.prevent_default();
        on_action.emit(action);
        return current_focus;
    }

    match key.as_str() {
        "ArrowUp" => {
            event.prevent_default();
            navigate_up_index(current_focus)
        }
        "ArrowDown" => {
            event.prevent_default();
            navigate_down_index(current_focus)
        }
        "Enter" | " " => {
            event.prevent_default();
            on_action.emit(current_focus);
            current_focus
        }
        "Escape" => {
            event.prevent_default();
            on_action.emit(0);
            current_focus
        }
        _ => current_focus,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn parse_numeric_key(key: &str) -> Option<u8> {
    match key {
        "1" | "Digit1" => Some(1),
        "2" | "Digit2" => Some(2),
        "3" | "Digit3" => Some(3),
        "4" | "Digit4" => Some(4),
        "5" | "Digit5" => Some(5),
        "0" | "Digit0" => Some(0),
        _ => None,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const fn navigate_up_index(idx: u8) -> u8 {
    match idx {
        1 => 0,
        0 => 5,
        n => n.saturating_sub(1),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const fn navigate_down_index(idx: u8) -> u8 {
    match idx {
        0 => 1,
        5 => 0,
        n => n + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[function_component(MenuItemHarness)]
    fn menu_item_harness() -> Html {
        let on_action = Callback::from(|_: u8| {});
        render_menu_item(1, 1, "Replay", &on_action)
    }

    #[test]
    fn render_menu_item_outputs_label_and_index() {
        let html = block_on(LocalServerRenderer::<MenuItemHarness>::new().render());
        assert!(html.contains("menu-item"));
        assert!(html.contains("Replay"));
    }

    #[test]
    fn render_menu_item_handles_zero_index() {
        #[function_component(ZeroMenuItemHarness)]
        fn zero_menu_item_harness() -> Html {
            let on_action = Callback::from(|_: u8| {});
            render_menu_item(0, 0, "Title", &on_action)
        }

        let html = block_on(LocalServerRenderer::<ZeroMenuItemHarness>::new().render());
        assert!(html.contains("0) Title"));
    }

    #[test]
    fn navigate_index_wraps() {
        assert_eq!(navigate_up_index(1), 0);
        assert_eq!(navigate_up_index(0), 5);
        assert_eq!(navigate_up_index(3), 2);
        assert_eq!(navigate_down_index(5), 0);
        assert_eq!(navigate_down_index(0), 1);
        assert_eq!(navigate_down_index(3), 4);
    }

    #[test]
    fn parse_numeric_key_matches_digit_variants() {
        assert_eq!(parse_numeric_key("1"), Some(1));
        assert_eq!(parse_numeric_key("Digit2"), Some(2));
        assert_eq!(parse_numeric_key("5"), Some(5));
        assert_eq!(parse_numeric_key("Digit0"), Some(0));
        assert_eq!(parse_numeric_key("X"), None);
    }

    #[test]
    fn emit_menu_action_emits_index() {
        let called = Rc::new(Cell::new(None::<u8>));
        let called_ref = called.clone();
        let on_action = Callback::from(move |idx| called_ref.set(Some(idx)));
        emit_menu_action(&on_action, 4);
        assert_eq!(called.get(), Some(4));
    }
}
