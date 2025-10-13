use crate::i18n;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
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
    let import_text = use_state(|| AttrValue::from(""));
    let on_input = {
        let st = import_text.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                st.set(input.value().into());
            }
        })
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

    // Focus trap and restoration (top-level hook)
    {
        let container_ref = container_ref.clone();
        let ret = p.return_focus_id.clone();
        let open = p.open;
        use_effect_with(
            (open, ret, container_ref),
            move |(open, ret, container_ref)| {
                let focus_target = if *open {
                    container_ref
                        .cast::<web_sys::Element>()
                        .and_then(|el| {
                            el.query_selector_all(
                                "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])",
                            )
                            .ok()
                            .and_then(|list| {
                                list.get(0)
                                    .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                            })
                        })
                } else {
                    None
                };
                if let Some(first) = focus_target {
                    let _ = first.focus();
                }
                let ret_id = ret.clone();
                move || {
                    if let Some(el) = ret_id
                        .clone()
                        .and_then(|id| {
                            web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|doc| doc.get_element_by_id(id.as_ref()))
                        })
                        .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                    {
                        let _ = el.focus();
                    }
                }
            },
        );
    }

    if !p.open {
        return html! {};
    }

    let on_keydown = {
        let container_ref = container_ref.clone();
        let on_close = p.on_close.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                on_close.emit(());
                return;
            }
            if e.key() != "Tab" {
                return;
            }
            let Some(container) = container_ref.cast::<web_sys::Element>() else {
                return;
            };
            let Ok(list) = container.query_selector_all(
                "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])",
            ) else {
                return;
            };
            let len = list.length();
            if len == 0 {
                return;
            }
            let first = list
                .get(0)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
            let last = list
                .get(len - 1)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
            let active = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element());
            let shift = e.shift_key();
            if let (Some(first), Some(last), Some(active)) = (first, last, active) {
                let first_el: web_sys::Element = first.clone().unchecked_into();
                let last_el: web_sys::Element = last.clone().unchecked_into();
                let is_first = active == first_el;
                let is_last = active == last_el;
                if !container.contains(Some(&active)) {
                    e.prevent_default();
                    let _ = first.focus();
                    return;
                }
                if shift && is_first {
                    e.prevent_default();
                    let _ = last.focus();
                } else if !shift && is_last {
                    e.prevent_default();
                    let _ = first.focus();
                }
            }
        })
    };

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
                        <textarea id="import-json" value={(*import_text).clone()} oninput={on_input} rows={6} cols={40}></textarea>
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
