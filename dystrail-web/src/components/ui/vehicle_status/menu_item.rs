use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct VehicleMenuItemProps {
    pub index: u8,
    pub label: AttrValue,
    pub focused: bool,
    pub disabled: bool,
    pub posinset: u8,
    pub setsize: u8,
    pub on_activate: Callback<u8>,
}

fn activate_item(disabled: bool, idx: u8, on_activate: &Callback<u8>) {
    if !disabled {
        on_activate.emit(idx);
    }
}

#[function_component(VehicleMenuItem)]
pub fn vehicle_menu_item(p: &VehicleMenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        let disabled = p.disabled;
        Callback::from(move |_| activate_item(disabled, idx, &on))
    };

    let classes = if p.disabled {
        "ot-menuitem disabled"
    } else {
        "ot-menuitem"
    };

    html! {
      <li role="menuitem"
          tabindex={ if p.focused { "0" } else { "-1" } }
          data-key={idx.to_string()}
          aria-posinset={p.posinset.to_string()}
          aria-setsize={p.setsize.to_string()}
          aria-disabled={p.disabled.to_string()}
          onclick={on_click}
          class={classes}>
         <span class="num">{ format!("{idx})") }</span>
         <span class="label">{ p.label.clone() }</span>
      </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn vehicle_menu_item_activate_emits_when_enabled() {
        let called = Rc::new(Cell::new(None::<u8>));
        let called_ref = called.clone();
        let on_activate = Callback::from(move |idx| called_ref.set(Some(idx)));
        activate_item(false, 4, &on_activate);
        assert_eq!(called.get(), Some(4));
    }
}
