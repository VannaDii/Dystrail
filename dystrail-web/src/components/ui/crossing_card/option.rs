use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct CrossingOptionProps {
    pub index: u8, // 1, 2, 3, or 0
    pub label: AttrValue,
    pub desc: AttrValue,
    pub focused: bool,
    pub disabled: bool,
    pub posinset: u8,
    pub setsize: u8,
    pub on_activate: Callback<u8>,
}

#[function_component(CrossingOption)]
pub fn crossing_option(p: &CrossingOptionProps) -> Html {
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

    let desc_id = format!("desc-{idx}");

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
