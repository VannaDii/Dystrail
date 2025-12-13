use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct IndicatorProps {
    #[prop_or_default]
    pub indicator: Option<f::Html>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Indicator)]
pub fn indicator(props: &IndicatorProps) -> f::Html {
    let class = f::class_list(&["indicator"], &props.class);
    f::html! {
        <div class={class}>
            { props.indicator.clone().map(|node| f::html! { <span class="indicator-item">{ node }</span> }).unwrap_or_default() }
            { for props.children.iter() }
        </div>
    }
}
