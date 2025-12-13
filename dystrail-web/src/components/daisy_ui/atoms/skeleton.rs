use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct SkeletonProps {
    #[prop_or_default]
    pub width: Option<f::AttrValue>,
    #[prop_or_default]
    pub height: Option<f::AttrValue>,
    #[prop_or_default]
    pub text: bool,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Skeleton)]
pub fn skeleton(props: &SkeletonProps) -> f::Html {
    let mut class = f::class_list(&["skeleton"], &props.class);
    if props.text {
        class.push("skeleton-text");
    }
    let mut style = String::new();
    if let Some(w) = props.width.as_ref() {
        style.push_str("width:");
        style.push_str(w);
        style.push(';');
    }
    if let Some(h) = props.height.as_ref() {
        style.push_str("height:");
        style.push_str(h);
        style.push(';');
    }
    f::html! { <div class={class} style={style}></div> }
}
