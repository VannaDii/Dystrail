use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct ButtonProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub variant: Option<f::DaisyColor>,
    #[prop_or_default]
    pub size: Option<f::DaisySize>,
    #[prop_or_default]
    pub outline: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub button_type: Option<f::AttrValue>,
    #[prop_or_default]
    pub aria_label: Option<f::AttrValue>,
    #[prop_or_default]
    pub onclick: f::Callback<f::MouseEvent>,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Button)]
pub fn button(props: &ButtonProps) -> f::Html {
    let mut classes = f::class_list(&["btn"], &props.class);
    if let Some(variant) = props.variant {
        classes.push(variant.class("btn"));
    }
    if let Some(size) = props.size {
        classes.push(size.class("btn"));
    }
    if props.outline {
        classes.push("btn-outline");
    }
    let aria_label = f::attr_value(&props.aria_label);
    let button_type = props.button_type.clone().unwrap_or_else(|| "button".into());
    f::html! {
        <button
            type={button_type}
            class={classes}
            aria-label={aria_label}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { if props.children.is_empty() {
                props.label.as_ref().map(|l| f::html!{ { l.clone() } }).unwrap_or_default()
            } else {
                props.children.iter().collect::<f::Html>()
            }}
        </button>
    }
}
