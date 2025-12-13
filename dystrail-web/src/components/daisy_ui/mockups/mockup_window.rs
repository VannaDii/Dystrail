use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct MockupWindowProps {
    #[prop_or_default]
    pub title: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(MockupWindow)]
pub fn mockup_window(props: &MockupWindowProps) -> f::Html {
    let class = f::class_list(&["mockup-window", "border"], &props.class);
    f::html! {
        <div class={class}>
            <div class="bar">{ props.title.clone().unwrap_or_else(|| "Window".into()) }</div>
            <div class="p-4 bg-base-200">{ for props.children.iter() }</div>
        </div>
    }
}
