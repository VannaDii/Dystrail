use super::focus::{focus_keydown_handler, use_focus_trap};
use crate::i18n;
use web_sys::InputEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub open: bool,
    pub on_close: Callback<()>,
    pub on_save: Callback<()>,
    pub on_load: Callback<()>,
    pub on_export: Callback<()>,
    pub on_import: Callback<String>,
    #[prop_or_default]
    pub return_focus_id: Option<AttrValue>,
}

#[function_component(SaveDrawer)]
pub fn save_drawer(p: &Props) -> Html {
    let container_ref = use_node_ref();
    use_focus_trap(p.open, p.return_focus_id.clone(), container_ref.clone());

    let import_text = use_state(|| AttrValue::from(""));
    let on_input = {
        #[cfg(target_arch = "wasm32")]
        {
            let on_input_value = {
                let st = import_text.clone();
                Callback::from(move |value: AttrValue| st.set(value))
            };
            Callback::from(move |e: InputEvent| {
                if let Some(input) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                    on_input_value.emit(input.value().into());
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Callback::from(|_e: InputEvent| {})
        }
    };
    let close = {
        let cb = p.on_close.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let save = {
        let cb = p.on_save.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let load = {
        let cb = p.on_load.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let export_btn = {
        let cb = p.on_export.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let import_btn = {
        let cb = p.on_import.clone();
        let val = import_text.clone();
        Callback::from(move |_| cb.emit((*val).to_string()))
    };

    if !p.open {
        return html! {};
    }

    let on_keydown = focus_keydown_handler(&container_ref, p.on_close.clone());

    html! {
        <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="save-title" ref={container_ref} onkeydown={on_keydown}>
            <div class="drawer-body">
                <h2 id="save-title">{ i18n::t("save.title") }</h2>
                <div class="controls">
                    <button onclick={save.clone()}>{ i18n::t("save.save") }</button>
                    <button onclick={load.clone()}>{ i18n::t("save.load") }</button>
                    <button onclick={export_btn.clone()}>{ i18n::t("save.export") }</button>
                </div>
                <div class="panel">
                    <div class="field">
                        <label for="import-json"><strong>{ i18n::t("save.import_label") }</strong></label>
                        <textarea id="import-json" value={(*import_text).clone()} oninput={on_input} rows={6} cols={40} />
                    </div>
                    <div class="controls">
                        <button onclick={import_btn}>{ i18n::t("save.import_button") }</button>
                    </div>
                </div>
                <div class="controls">
                    <button onclick={close}>{ i18n::t("save.close") }</button>
                </div>
            </div>
        </div>
    }
}
