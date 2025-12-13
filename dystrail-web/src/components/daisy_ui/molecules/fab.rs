use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct FabProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub icon: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub onclick: f::Callback<f::MouseEvent>,
}

#[f::function_component(Fab)]
pub fn fab(props: &FabProps) -> f::Html {
    let mut class = f::class_list(
        &["btn", "btn-primary", "btn-circle", "shadow-lg"],
        &props.class,
    );
    class.push("fab");
    f::html! {
        <button class={class} aria-label={props.label.clone()} onclick={props.onclick.clone()}>
            { props.icon.clone().unwrap_or_else(|| f::Html::from(f::html!{ "＋" })) }
            { props.label.as_ref().map(|l| f::html! { <span class="sr-only">{ l.clone() }</span> }).unwrap_or_default() }
        </button>
    }
}
