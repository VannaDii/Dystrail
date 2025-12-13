use crate::components::daisy_ui::foundation as f;
use f::TargetCast;

fn apply_theme(theme: &str) {
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(root) = document.document_element()
    {
        let _ = root.set_attribute("data-theme", theme);
    }
}

#[derive(f::Properties, PartialEq, Clone)]
pub struct ThemeControllerProps {
    pub themes: Vec<f::AttrValue>,
    #[prop_or_default]
    pub value: Option<f::AttrValue>,
    #[prop_or_default]
    pub class: f::Classes,
    #[prop_or_default]
    pub on_change: f::Callback<f::AttrValue>,
}

#[f::function_component(ThemeController)]
pub fn theme_controller(props: &ThemeControllerProps) -> f::Html {
    let class = f::class_list(&["select", "select-bordered"], &props.class);
    let selected = props
        .value
        .clone()
        .unwrap_or_else(|| props.themes.first().cloned().unwrap_or_default());
    let on_change = {
        let cb = props.on_change.clone();
        f::Callback::from(move |e: f::Event| {
            if let Some(sel) = e.target_dyn_into::<f::HtmlSelectElement>() {
                let value: f::AttrValue = sel.value().into();
                apply_theme(&value);
                cb.emit(value);
            }
        })
    };
    f::html! {
        <select class={class} value={selected} onchange={on_change} aria-label="Theme selector">
            { for props.themes.iter().map(|theme| f::html!{ <option value={theme.clone()}>{ theme.clone() }</option> }) }
        </select>
    }
}
