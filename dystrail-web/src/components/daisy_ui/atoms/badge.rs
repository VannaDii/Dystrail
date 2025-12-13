use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct BadgeProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub variant: Option<f::DaisyColor>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Badge)]
pub fn badge(props: &BadgeProps) -> f::Html {
    let mut classes = f::class_list(&["badge"], &props.class);
    if let Some(variant) = props.variant {
        classes.push(variant.class("badge"));
    }
    f::html! {
        <span class={classes} role="status">
            { props.label.as_ref().map(|l| f::html! { { l.clone() } }).unwrap_or_default() }
            { for props.children.iter() }
        </span>
    }
}
