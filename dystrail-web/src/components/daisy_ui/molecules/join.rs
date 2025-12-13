use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct JoinProps {
    #[prop_or_default]
    pub vertical: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Join)]
pub fn join(props: &JoinProps) -> f::Html {
    let mut class = f::class_list(&["join"], &props.class);
    if props.vertical {
        class.push("join-vertical");
    } else {
        class.push("join-horizontal");
    }
    f::html! { <div class={class}>{ for props.children.iter() }</div> }
}
