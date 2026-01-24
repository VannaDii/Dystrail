use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct RoutePromptOptionProps {
    pub index: u8, // 1 or 2
    pub label: AttrValue,
    pub desc: AttrValue,
    pub focused: bool,
    pub disabled: bool,
    pub posinset: u8,
    pub setsize: u8,
    pub on_activate: Callback<u8>,
}

fn activate_option(disabled: bool, idx: u8, on_activate: &Callback<u8>) {
    if !disabled {
        on_activate.emit(idx);
    }
}

#[function_component(RoutePromptOption)]
pub fn route_prompt_option(p: &RoutePromptOptionProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        let disabled = p.disabled;
        Callback::from(move |_| activate_option(disabled, idx, &on))
    };

    let desc_id = format!("route-desc-{idx}");

    html! {
        <li role="menuitem"
            tabindex={ if p.focused { "0" } else { "-1" } }
            data-key={idx.to_string()}
            aria-posinset={p.posinset.to_string()}
            aria-setsize={p.setsize.to_string()}
            aria-describedby={desc_id.clone()}
            aria-disabled={ if p.disabled { "true" } else { "false" } }
            onclick={on_click}
            class={ classes!("ot-menuitem", if p.disabled { Some("disabled") } else { None }) }>
            <span class="num">{ format!("{idx})") }</span>
            <span class="label">{ p.label.clone() }</span>
            <small id={desc_id} class="muted desc">{ p.desc.clone() }</small>
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn route_prompt_option_activate_emits_when_enabled() {
        let called = Rc::new(Cell::new(None::<u8>));
        let called_ref = called.clone();
        let on_activate = Callback::from(move |idx| called_ref.set(Some(idx)));
        activate_option(false, 2, &on_activate);
        assert_eq!(called.get(), Some(2));
    }
}
