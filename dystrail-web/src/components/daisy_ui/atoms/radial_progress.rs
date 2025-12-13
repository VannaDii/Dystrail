use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct RadialProgressProps {
    pub value: f32,
    #[prop_or_default]
    pub max: f32,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
}

#[f::function_component(RadialProgress)]
pub fn radial_progress(props: &RadialProgressProps) -> f::Html {
    let max = if props.max <= 0.0 { 100.0 } else { props.max };
    let value = props.value.clamp(0.0, max);
    let percent = (value / max * 100.0).round();
    let class = f::class_list(&["radial-progress"], &props.class);
    let style = format!("--value:{percent}; --size:6rem; --thickness:8px;");
    f::html! {
        <div class="flex items-center gap-3" role="img" aria-label={format!("{percent:.0}%")}>
            <div class={class} style={style}>{ format!("{percent:.0}%") }</div>
            { props.label.as_ref().map(|l| f::html!{ <span>{ l.clone() }</span> }).unwrap_or_default() }
        </div>
    }
}
