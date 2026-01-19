use crate::components::daisy_ui::foundation as f;
#[cfg(target_arch = "wasm32")]
use f::TargetCast;

#[derive(Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub label: f::AttrValue,
    pub value: f::AttrValue,
    pub disabled: bool,
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct SelectProps {
    pub options: Vec<SelectOption>,
    #[prop_or_default]
    pub value: Option<f::AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<f::AttrValue>,
}

#[f::function_component(Select)]
pub fn select(props: &SelectProps) -> f::Html {
    let class = f::class_list(&["select", "select-bordered"], &props.class);
    let on_change = {
        let cb = props.on_change.clone();
        f::Callback::from(move |e: f::Event| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(sel) = e.target_dyn_into::<f::HtmlSelectElement>() {
                    cb.emit(sel.value().into());
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (&e, &cb);
            }
        })
    };
    f::html! {
        <select
            class={class}
            value={props.value.clone().unwrap_or_default()}
            disabled={props.disabled}
            onchange={on_change}
        >
            { for props.options.iter().map(|opt| {
                f::html! { <option value={opt.value.clone()} disabled={opt.disabled}>{ opt.label.clone() }</option> }
            })}
        </select>
    }
}
