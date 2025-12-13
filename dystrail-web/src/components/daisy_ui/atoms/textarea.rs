use crate::components::daisy_ui::foundation as f;
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct TextareaProps {
    #[prop_or_default]
    pub value: f::AttrValue,
    #[prop_or_default]
    pub placeholder: Option<f::AttrValue>,
    #[prop_or_default]
    pub rows: Option<u32>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub oninput: f::Callback<String>,
}

#[f::function_component(Textarea)]
pub fn textarea(props: &TextareaProps) -> f::Html {
    let oninput = {
        let cb = props.oninput.clone();
        f::Callback::from(move |e: f::InputEvent| {
            if let Some(input) = e.target_dyn_into::<f::HtmlTextAreaElement>() {
                cb.emit(input.value());
            }
        })
    };
    let class = f::class_list(&["textarea", "textarea-bordered", "w-full"], &props.class);
    f::html! {
        <textarea
            class={class}
            placeholder={f::attr_value(&props.placeholder)}
            rows={props.rows.unwrap_or(3).to_string()}
            value={props.value.clone()}
            oninput={oninput}
            disabled={props.disabled}
        />
    }
}
