use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct OtDeluxeCrossingOptionProps {
    pub index: u8,
    pub label: AttrValue,
    pub desc: AttrValue,
    pub focused: bool,
    pub disabled: bool,
    pub posinset: u8,
    pub setsize: u8,
    pub on_activate: Callback<u8>,
}

#[function_component(OtDeluxeCrossingOption)]
pub fn otdeluxe_crossing_option(props: &OtDeluxeCrossingOptionProps) -> Html {
    let idx = props.index;
    let on_click = {
        let on = props.on_activate.clone();
        let disabled = props.disabled;
        Callback::from(move |_| {
            if !disabled {
                on.emit(idx);
            }
        })
    };

    let desc_id = format!("otdeluxe-desc-{idx}");

    html! {
        <li role="menuitem"
            tabindex={ if props.focused { "0" } else { "-1" } }
            data-key={idx.to_string()}
            aria-posinset={props.posinset.to_string()}
            aria-setsize={props.setsize.to_string()}
            aria-describedby={desc_id.clone()}
            aria-disabled={ if props.disabled { "true" } else { "false" } }
            onclick={on_click}
            class={ classes!("ot-menuitem", if props.disabled { Some("disabled") } else { None }) }>
            <span class="num">{ format!("{idx})") }</span>
            <span class="label">{ props.label.clone() }</span>
            <small id={desc_id} class="muted desc">{ props.desc.clone() }</small>
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn otdeluxe_option_renders_disabled_state() {
        let props = OtDeluxeCrossingOptionProps {
            index: 4,
            label: AttrValue::from("Guide"),
            desc: AttrValue::from("Desc"),
            focused: false,
            disabled: true,
            posinset: 4,
            setsize: 5,
            on_activate: Callback::noop(),
        };
        let html =
            block_on(LocalServerRenderer::<OtDeluxeCrossingOption>::with_props(props).render());
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(html.contains("tabindex=\"-1\""));
    }

    #[test]
    fn otdeluxe_option_renders_focused_state() {
        let props = OtDeluxeCrossingOptionProps {
            index: 1,
            label: AttrValue::from("Ford"),
            desc: AttrValue::from("Desc"),
            focused: true,
            disabled: false,
            posinset: 1,
            setsize: 5,
            on_activate: Callback::noop(),
        };
        let html =
            block_on(LocalServerRenderer::<OtDeluxeCrossingOption>::with_props(props).render());
        assert!(html.contains("aria-disabled=\"false\""));
        assert!(html.contains("tabindex=\"0\""));
    }
}
