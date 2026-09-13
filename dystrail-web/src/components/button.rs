use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub label: AttrValue,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(Button)]
pub fn button(p: &Props) -> Html {
    crate::i18n::use_language();
    let onclick = p.onclick.clone();
    let label = p.label.clone();
    html! { <button {onclick}>{ label }</button> }
}
