use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct StepsProps {
    pub steps: Vec<f::AttrValue>,
    #[prop_or_default]
    pub current: usize,
    #[prop_or_default]
    pub horizontal: bool,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Steps)]
pub fn steps(props: &StepsProps) -> f::Html {
    let mut class = f::class_list(&["steps"], &props.class);
    if props.horizontal {
        class.push("steps-horizontal");
    } else {
        class.push("steps-vertical");
    }
    f::html! {
        <ul class={class} aria-label="Progress steps">
            { for props.steps.iter().enumerate().map(|(idx, step)| {
                let mut li_class = f::classes!("step");
                if idx <= props.current {
                    li_class.push("step-primary");
                }
                f::html! { <li class={li_class}><span class="sr-only">{ format!("Step {}", idx + 1) }</span><span class="step-label">{ step.clone() }</span></li> }
            })}
        </ul>
    }
}
