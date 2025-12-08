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

#[function_component(VehicleMenuItem)]
pub fn vehicle_menu_item(p: &VehicleMenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        let disabled = p.disabled;
        Callback::from(move |_| {
            if !disabled {
                on.emit(idx);
            }
        })
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
