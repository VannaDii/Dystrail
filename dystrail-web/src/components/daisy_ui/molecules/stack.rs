use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct StackProps {
    #[prop_or_default]
    pub align_top: bool,
    #[prop_or_default]
    pub align_start: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Stack)]
pub fn stack(props: &StackProps) -> f::Html {
    let mut class = f::class_list(&["stack"], &props.class);
    if props.align_top {
        class.push("stack-top");
    }
    if props.align_start {
        class.push("stack-start");
    }
    f::html! {
        <div class={class}>
            { for props.children.iter() }
        </div>
    }
}
