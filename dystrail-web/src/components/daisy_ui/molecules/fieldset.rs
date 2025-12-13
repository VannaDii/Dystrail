use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct FieldsetProps {
    #[prop_or_default]
    pub legend: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Fieldset)]
pub fn fieldset(props: &FieldsetProps) -> f::Html {
    let class = f::class_list(&["fieldset", "border", "rounded-box", "p-4"], &props.class);
    f::html! {
        <fieldset class={class}>
            { props.legend.as_ref().map(|l| f::html! { <legend class="legend font-semibold">{ l.clone() }</legend> }).unwrap_or_default() }
            { for props.children.iter() }
        </fieldset>
    }
}
