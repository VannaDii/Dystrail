use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct MenuItemProps {
    pub index: u8,        // 0..9
    pub label: AttrValue, // resolved string
    pub focused: bool,    // tabindex 0 vs -1
    pub posinset: u8,     // 1..=setsize
    pub setsize: u8,
    pub on_activate: Callback<u8>, // called with index
}

#[function_component(MenuItem)]
pub fn menu_item(p: &MenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        Callback::from(move |_| on.emit(idx))
    };

    html! {
      <li role="menuitem"
          tabindex={ if p.focused { "0" } else { "-1" } }
          data-key={idx.to_string()}
          aria-posinset={p.posinset.to_string()}
          aria-setsize={p.setsize.to_string()}
          onclick={on_click}
          class="ot-menuitem">
         <span class="num">{ format!("{}{})", idx, ")") }</span>
         <span class="label">{ p.label.clone() }</span>
      </li>
    }
}
