use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct MenuLineProps {
    pub index: u8,
    pub text: String,
    pub selected: bool,
    pub focused: bool,
    pub on_activate: Callback<u8>,
    pub on_focus: Callback<u8>,
    pub tooltip: String,
}

#[function_component(MenuLine)]
pub fn menu_line(props: &MenuLineProps) -> Html {
    let onclick = {
        let on_activate = props.on_activate.clone();
        let idx = props.index;
        Callback::from(move |_: MouseEvent| on_activate.emit(idx))
    };

    let onfocus = {
        let on_focus = props.on_focus.clone();
        let idx = props.index;
        Callback::from(move |_: FocusEvent| on_focus.emit(idx))
    };

    let classes = classes!(
        "pace-diet-line",
        props.focused.then_some("focused"),
        props.selected.then_some("selected")
    );

    let tooltip_id = format!("tooltip-{}", props.index);

    html! {
        <li
            class={classes}
            role="menuitem"
            tabindex={if props.focused { "0" } else { "-1" }}
            aria-describedby={tooltip_id.clone()}
            aria-current={props.selected.then_some("true")}
            {onclick}
            {onfocus}
        >
            <span class="line-number">{ format!("{}{}", props.index, ")") }</span>
            <span class="line-text">{ props.text.clone() }</span>
            <div id={tooltip_id} class="sr-only">{ props.tooltip.clone() }</div>
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn menu_line_renders_tooltip_and_labels() {
        let props = MenuLineProps {
            index: 1,
            text: String::from("Steady"),
            selected: true,
            focused: true,
            tooltip: String::from("Tip"),
            on_activate: Callback::noop(),
            on_focus: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<MenuLine>::with_props(props).render());
        assert!(html.contains("Steady"));
        assert!(html.contains("tooltip-1"));
    }
}
