use crate::components::daisy_ui::foundation as f;
#[cfg(target_arch = "wasm32")]
use f::TargetCast;

#[derive(f::Properties, PartialEq, Clone)]
pub struct FileInputProps {
    #[prop_or_default]
    pub label: Option<f::AttrValue>,
    #[prop_or_default]
    pub accept: Option<f::AttrValue>,
    #[prop_or_default]
    pub multiple: bool,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<Vec<String>>,
}

#[f::function_component(FileInput)]
pub fn file_input(props: &FileInputProps) -> f::Html {
    let on_change = {
        let on_change = props.on_change.clone();
        #[cfg(target_arch = "wasm32")]
        {
            f::Callback::from(move |e: f::Event| {
                if let Some(input) = e.target_dyn_into::<f::HtmlInputElement>()
                    && let Some(files) = input.files()
                {
                    let mut names = Vec::new();
                    for idx in 0..files.length() {
                        if let Some(file) = files.item(idx) {
                            names.push(file.name());
                        }
                    }
                    on_change.emit(names);
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = on_change;
            f::Callback::from(|_e: f::Event| {})
        }
    };
    let class = f::class_list(&["file-input"], &props.class);
    f::html! {
        <label class="form-control w-full gap-2">
            { props.label.as_ref().map(|l| f::html! { <span class="label-text">{ l.clone() }</span> }).unwrap_or_default() }
            <input
                class={class}
                type="file"
                accept={f::attr_value(&props.accept)}
                multiple={props.multiple}
                onchange={on_change}
            />
        </label>
    }
}
