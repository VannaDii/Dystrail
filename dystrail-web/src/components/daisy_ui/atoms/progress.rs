use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Clone)]
pub struct ProgressProps {
    pub value: f32,
    #[prop_or_default]
    pub max: f32,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub variant: Option<f::DaisyColor>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Progress)]
pub fn progress(props: &ProgressProps) -> f::Html {
    let mut class = f::class_list(&["progress", "w-full"], &props.class);
    if let Some(variant) = props.variant {
        class.push(variant.class("progress"));
    }
    let max = if props.max <= 0.0 { 100.0 } else { props.max };
    let value = props.value.clamp(0.0, max);
    let percent = (value / max * 100.0).round();
    f::html! {
        <div class="flex items-center gap-2">
            <progress class={class} value={value.to_string()} max={max.to_string()} aria-valuenow={value.to_string()} aria-valuemax={max.to_string()}></progress>
            { props.label.as_ref().map_or_else(
                || f::html!{ <span class="text-sm text-base-content/70">{ format!("{percent:.0}%") }</span> },
                |l| f::html!{ <span class="text-sm">{ l.clone() }</span> },
            ) }
        </div>
    }
}
