use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct TooltipProps {
    pub text: f::AttrValue,
    #[prop_or_default]
    pub position: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> f::Html {
    let mut class = f::class_list(&["tooltip"], &props.class);
    if let Some(pos) = props.position.as_ref() {
        class.push(pos.clone());
    }
    f::html! {
        <div class={class} data-tip={props.text.clone()}>
            { for props.children.iter() }
        </div>
    }
}
