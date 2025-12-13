use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct LoadingProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub size: Option<f::DaisySize>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Loading)]
pub fn loading(props: &LoadingProps) -> f::Html {
    let mut class = f::class_list(&["loading", "loading-spinner"], &props.class);
    if let Some(size) = props.size {
        class.push(size.class("loading"));
    }
    f::html! {
        <div class="inline-flex items-center gap-2" role="status" aria-live="polite">
            <span class={class}></span>
            { props.label.as_ref().map(|l| f::html! { <span>{ l.clone() }</span> }).unwrap_or_default() }
        </div>
    }
}
