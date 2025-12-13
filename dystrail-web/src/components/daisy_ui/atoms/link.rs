use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct LinkProps {
    pub href: f::AttrValue,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub new_tab: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Link)]
pub fn link(props: &LinkProps) -> f::Html {
    let class = f::class_list(&["link"], &props.class);
    let target: Option<f::AttrValue> = if props.new_tab {
        Some("_blank".into())
    } else {
        None
    };
    let rel: Option<f::AttrValue> = if props.new_tab {
        Some("noopener noreferrer".into())
    } else {
        None
    };
    f::html! {
        <a class={class} href={props.href.clone()} target={target} rel={rel}>
            { if props.children.is_empty() {
                props.label.clone().map(f::Html::from).unwrap_or_default()
            } else {
                props.children.iter().collect::<f::Html>()
            } }
        </a>
    }
}
