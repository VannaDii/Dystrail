use crate::components::daisy_ui::foundation as f;
#[cfg(target_arch = "wasm32")]
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct CheckboxProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub checked: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_toggle: f::Callback<bool>,
}

#[f::function_component(Checkbox)]
pub fn checkbox(props: &CheckboxProps) -> f::Html {
    let on_change = {
        let on_toggle = props.on_toggle.clone();
        f::Callback::from(move |e: f::Event| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(input) = e.target_dyn_into::<f::HtmlInputElement>() {
                    on_toggle.emit(input.checked());
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (&e, &on_toggle);
            }
        })
    };
    let class = f::class_list(&["checkbox"], &props.class);
    f::html! {
        <label class="label cursor-pointer gap-2">
            <input
                class={class}
                type="checkbox"
                checked={props.checked}
                disabled={props.disabled}
                onchange={on_change}
            />
            { props.label.as_ref().map(|l| f::html! { <span>{ l.clone() }</span> }).unwrap_or_default() }
        </label>
    }
}
