use crate::components::daisy_ui::foundation as f;
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct InputProps {
    #[prop_or_default]
    pub value: f::AttrValue,
    #[prop_or_default]
    pub placeholder: Option<f::AttrValue>,
    #[prop_or_default]
    pub input_type: Option<f::AttrValue>,
    #[prop_or_default]
    pub name: Option<f::AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub oninput: f::Callback<String>,
}

#[f::function_component(Input)]
pub fn input(props: &InputProps) -> f::Html {
    let oninput = {
        let cb = props.oninput.clone();
        f::Callback::from(move |e: f::InputEvent| {
            if let Some(input) = e.target_dyn_into::<f::HtmlInputElement>() {
                cb.emit(input.value());
            }
        })
    };
    let class = f::class_list(&["input", "input-bordered"], &props.class);
    let input_type = props.input_type.clone().unwrap_or_else(|| "text".into());
    f::html! {
        <input
            class={class}
            type={input_type}
            name={f::attr_value(&props.name)}
            value={props.value.clone()}
            placeholder={f::attr_value(&props.placeholder)}
            disabled={props.disabled}
            readonly={props.readonly}
            oninput={oninput}
        />
    }
}
