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
        Callback::from(move |_: MouseEvent| {
            on_action.emit(index);
        })
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

const fn navigate_up_index(idx: u8) -> u8 {
    match idx {
        1 => 0,
        0 => 5,
        n => n.saturating_sub(1),
    }
}

const fn navigate_down_index(idx: u8) -> u8 {
    match idx {
        0 => 1,
        5 => 0,
        n => n + 1,
    }
}
