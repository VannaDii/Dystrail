use crate::components::daisy_ui::foundation as f;
#[cfg(target_arch = "wasm32")]
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct ToggleProps {
    #[prop_or_default]
    pub checked: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub on_toggle: f::Callback<bool>,
}

#[f::function_component(Toggle)]
pub fn toggle(props: &ToggleProps) -> f::Html {
    let on_change = {
        let cb = props.on_toggle.clone();
        f::Callback::from(move |e: f::Event| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(input) = e.target_dyn_into::<f::HtmlInputElement>() {
                    cb.emit(input.checked());
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (&e, &cb);
            }
        })
    };
    let class = f::class_list(&["toggle"], &props.class);
    f::html! {
        <label class="label cursor-pointer gap-2">
            { props.label.as_ref().map(|l| f::html!{ <span>{ l.clone() }</span> }).unwrap_or_default() }
            <input
                class={class}
                type="checkbox"
                checked={props.checked}
                disabled={props.disabled}
                onchange={on_change}
            />
        </label>
    }
}
