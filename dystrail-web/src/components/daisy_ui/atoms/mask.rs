use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct MaskProps {
    #[prop_or_default]
    pub shape: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Mask)]
pub fn mask(props: &MaskProps) -> f::Html {
    let mut class = f::class_list(&["mask"], &props.class);
    if let Some(shape) = props.shape.as_ref() {
        class.push(shape.clone());
    }
    f::html! {
        <div class={class}>
            { for props.children.iter() }
        </div>
    }
}
