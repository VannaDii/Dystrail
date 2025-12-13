use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct LabelProps {
    #[prop_or_default]
    pub for_input: Option<f::AttrValue>,
    #[prop_or_default]
    pub text: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Label)]
pub fn label(props: &LabelProps) -> f::Html {
    let class = f::class_list(&["label"], &props.class);
    f::html! {
        <label class={class} for={f::attr_value(&props.for_input)}>
            { props.text.as_ref().map(|t| f::html!{ <span class="label-text">{ t.clone() }</span> }).unwrap_or_default() }
            { for props.children.iter() }
        </label>
    }
}
