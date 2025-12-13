use crate::components::daisy_ui::foundation as f;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    Success,
    Error,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct ValidatorProps {
    #[prop_or_default]
    pub state: Option<ValidationState>,
    #[prop_or_default]
    pub message: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub children: f::Children,
}

#[f::function_component(Validator)]
pub fn validator(props: &ValidatorProps) -> f::Html {
    let mut class = f::class_list(&["validator"], &props.class);
    if let Some(state) = props.state {
        class.push(match state {
            ValidationState::Success => "input-success",
            ValidationState::Error => "input-error",
        });
    }
    f::html! {
        <div class={class}>
            { for props.children.iter() }
            { props.message.as_ref().map(|msg| {
                let msg_class = match props.state {
                    Some(ValidationState::Success) => "validator-hint text-success",
                    Some(ValidationState::Error) => "validator-hint text-error",
                    None => "validator-hint",
                };
                f::html! { <p class={msg_class}>{ msg.clone() }</p> }
            }).unwrap_or_default() }
        </div>
    }
}
