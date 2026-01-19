use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub label: AttrValue,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(Button)]
pub fn button(p: &Props) -> Html {
    let onclick = p.onclick.clone();
    let label = p.label.clone();
    html! { <button {onclick}>{ label }</button> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn button_renders_label() {
        let props = Props {
            label: AttrValue::from("Confirm"),
            onclick: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<Button>::with_props(props).render());
        assert!(html.contains("Confirm"));
    }
}
