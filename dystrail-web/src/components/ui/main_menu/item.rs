use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct MenuItemProps {
    pub index: u8,        // 1..=4
    pub label: AttrValue, // resolved string
    pub focused: bool,    // active styling + aria-current
    pub posinset: u8,     // 1..=setsize
    pub setsize: u8,
    pub primary: bool,
    pub on_activate: Callback<u8>, // called with index
    pub on_focus: Callback<u8>,
}

fn focus_handler(on_focus: Callback<u8>, idx: u8) -> Callback<()> {
    Callback::from(move |()| on_focus.emit(idx))
}

#[function_component(MenuItem)]
pub fn menu_item(p: &MenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        Callback::from(move |_| on.emit(idx))
    };
    let on_focus = focus_handler(p.on_focus.clone(), idx).reform(|_e: FocusEvent| ());

    let mut classes = Classes::from(
        "btn btn-ghost w-full justify-start rounded-none text-left normal-case font-sans shell-btn",
    );
    if p.primary {
        classes.push("text-base-content");
    } else {
        classes.push("text-base-content/80");
    }
    if p.focused {
        classes.push("btn-active");
    }

    let data_action = if p.primary {
        Some(AttrValue::from("start"))
    } else {
        None
    };
    if p.primary {
        classes.push("start");
    }

    html! {
      <li role="none" class="w-full">
        <button
          role="menuitem"
          tabindex="0"
          data-key={idx.to_string()}
          data-action={data_action}
          aria-current={if p.focused { "true" } else { "false" }}
          aria-posinset={p.posinset.to_string()}
          aria-setsize={p.setsize.to_string()}
          onclick={on_click}
          onfocus={on_focus}
          class={classes}
        >
          { if p.primary { html!{ <span class="mr-2">{ ">" }</span> } } else { Html::default() } }
          <span>{ p.label.clone() }</span>
        </button>
      </li>
    }
}

#[cfg(test)]
mod tests {
    use super::focus_handler;
    use std::cell::Cell;
    use std::rc::Rc;
    use yew::Callback;

    #[test]
    fn focus_handler_emits_index() {
        let seen = Rc::new(Cell::new(0_u8));
        let seen_ref = seen.clone();
        let on_focus = Callback::from(move |idx| seen_ref.set(idx));
        let handler = focus_handler(on_focus, 3);
        handler.emit(());
        assert_eq!(seen.get(), 3);
    }
}
