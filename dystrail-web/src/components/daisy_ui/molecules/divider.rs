use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct DividerProps {
    #[prop_or_default]
    pub text: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Divider)]
pub fn divider(props: &DividerProps) -> f::Html {
    let class = f::class_list(&["divider"], &props.class);
    f::html! {
        <div class={class} role="separator">
            { props.text.as_ref().map(|t| f::html!{ <span>{ t.clone() }</span> }).unwrap_or_default() }
        </div>
    }
}
