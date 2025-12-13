use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct StatusProps {
    #[prop_or_default]
    pub variant: f::DaisyColor,
    #[prop_or_default]
    pub size: Option<f::DaisySize>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
}

#[f::function_component(Status)]
pub fn status(props: &StatusProps) -> f::Html {
    let mut class = f::class_list(&["status"], &props.class);
    class.push(props.variant.class("status"));
    if let Some(size) = props.size {
        class.push(size.class("status"));
    }
    f::html! { <span class={class} aria-label={f::attr_value(&props.label)}></span> }
}
