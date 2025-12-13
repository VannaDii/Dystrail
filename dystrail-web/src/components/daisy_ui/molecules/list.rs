use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct ListProps {
    #[prop_or_default]
    pub items: Vec<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(List)]
pub fn list(props: &ListProps) -> f::Html {
    let class = f::class_list(&["list", "list-disc", "ps-5", "space-y-2"], &props.class);
    f::html! {
        <ul class={class}>
            { if props.children.is_empty() {
                props.items.iter().map(|item| f::html!{ <li>{ item.clone() }</li> }).collect::<f::Html>()
            } else {
                props.children.iter().collect::<f::Html>()
            }}
        </ul>
    }
}
