use crate::components::daisy_ui::foundation as f;
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct RangeProps {
    pub value: f64,
    #[prop_or(0.0)]
    pub min: f64,
    #[prop_or(100.0)]
    pub max: f64,
    #[prop_or(1.0)]
    pub step: f64,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<f64>,
}

#[f::function_component(Range)]
pub fn range(props: &RangeProps) -> f::Html {
    let on_change = {
        let cb = props.on_change.clone();
        f::Callback::from(move |e: f::InputEvent| {
            if let Some(input) = e.target_dyn_into::<f::HtmlInputElement>()
                && let Ok(val) = input.value().parse::<f64>()
            {
                cb.emit(val);
            }
        })
    };
    let class = f::class_list(&["range"], &props.class);
    f::html! {
        <input
            class={class}
            type="range"
            min={props.min.to_string()}
            max={props.max.to_string()}
            step={props.step.to_string()}
            value={props.value.to_string()}
            oninput={on_change}
        />
    }
}
