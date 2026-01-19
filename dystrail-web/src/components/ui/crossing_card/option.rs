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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn crossing_option_renders_disabled_state() {
        let props = CrossingOptionProps {
            index: 2,
            label: AttrValue::from("Bribe"),
            desc: AttrValue::from("Desc"),
            focused: false,
            disabled: true,
            posinset: 2,
            setsize: 4,
            on_activate: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<CrossingOption>::with_props(props).render());
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(html.contains("tabindex=\"-1\""));
        assert!(html.contains("disabled"));
    }

    #[test]
    fn crossing_option_renders_focused_state() {
        let props = CrossingOptionProps {
            index: 1,
            label: AttrValue::from("Detour"),
            desc: AttrValue::from("Desc"),
            focused: true,
            disabled: false,
            posinset: 1,
            setsize: 4,
            on_activate: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<CrossingOption>::with_props(props).render());
        assert!(html.contains("aria-disabled=\"false\""));
        assert!(html.contains("tabindex=\"0\""));
    }
}
