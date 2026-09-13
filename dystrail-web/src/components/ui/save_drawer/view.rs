use super::focus::{focus_keydown_handler, use_focus_trap};
use crate::{components::ui::journey_icon, i18n};
use web_sys::InputEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub open: bool,
    #[prop_or_default]
    pub status: String,
    pub on_close: Callback<()>,
    pub on_save: Callback<()>,
    pub on_load: Callback<()>,
    pub on_export: Callback<()>,
    pub on_download: Callback<()>,
    pub on_import: Callback<String>,
    #[prop_or(true)]
    pub can_save: bool,
    #[prop_or_default]
    pub return_focus_id: Option<AttrValue>,
}

#[function_component(SaveDrawer)]
pub fn save_drawer(p: &Props) -> Html {
    crate::i18n::use_language();
    let container_ref = use_node_ref();
    let file_ref = use_node_ref();
    use_focus_trap(p.open, p.return_focus_id.clone(), container_ref.clone());
    let import_text = use_state(|| AttrValue::from(""));
    let file_error = use_state(|| None::<String>);
    let reading_file = use_state(|| false);
    let on_input = {
        let st = import_text.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                st.set(input.value().into());
            }
        })
    };
    let close = p.on_close.reform(|_: MouseEvent| ());
    let save = p.on_save.reform(|_: MouseEvent| ());
    let load = p.on_load.reform(|_: MouseEvent| ());
    let export_btn = p.on_export.reform(|_: MouseEvent| ());
    let download = p.on_download.reform(|_: MouseEvent| ());
    let choose_file = {
        let file_ref = file_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_ref.cast::<web_sys::HtmlInputElement>() {
                input.click();
            }
        })
    };
    let file_import = super::transfer::file_import(
        p.on_import.clone(),
        file_error.clone(),
        reading_file.clone(),
    );
    let import_btn = {
        let cb = p.on_import.clone();
        let val = import_text.clone();
        Callback::from(move |_| cb.emit((*val).to_string()))
    };
    super::super::dismiss::use_outside_dismiss(container_ref.clone(), p.open, p.on_close.clone());
    if !p.open {
        return html! {};
    }
    let on_keydown = focus_keydown_handler(container_ref.clone(), p.on_close.clone());
    let status = file_error.as_ref().unwrap_or(&p.status);
    html! {
        <div class="drawer save-manager" role="dialog" aria-modal="true" aria-labelledby="save-title" onkeydown={on_keydown}>
            <div class="drawer-body" ref={container_ref}>
                <header class="save-manager-header">
                    <h2 id="save-title">{journey_icon::render("save")}{i18n::t("save.title")}</h2>
                    <button class="save-close" onclick={close} aria-label={i18n::t("save.close")}>{journey_icon::render("close")}</button>
                </header>
                <div class="save-manager-content">
                    <section class="save-section" aria-labelledby="save-device-title">
                        <h3 id="save-device-title">{i18n::t("save.device")}</h3>
                        <p>{i18n::t("save.device_help")}</p>
                        <div class="save-actions">
                            <button class="save-action retro-btn-primary" onclick={save} disabled={!p.can_save}>{journey_icon::render("save")}<span>{i18n::t("save.save")}</span></button>
                            <button class="save-action" onclick={load}>{journey_icon::render("load")}<span>{i18n::t("save.load")}</span></button>
                        </div>
                    </section>
                    <section class="save-section" aria-labelledby="save-backup-title">
                        <h3 id="save-backup-title">{i18n::t("save.backup")}</h3>
                        <p>{i18n::t("save.backup_help")}</p>
                        <div class="save-actions">
                            <button class="save-action" onclick={download} disabled={!p.can_save}>{journey_icon::render("download")}<span>{i18n::t("save.download")}</span></button>
                            <button class="save-action" onclick={choose_file} disabled={*reading_file}>{journey_icon::render("upload")}<span>{i18n::t(if *reading_file {"save.reading"} else {"save.restore_file"})}</span></button>
                            <input id="save-file" class="save-file-input" type="file" accept=".json,application/json" ref={file_ref} onchange={file_import} tabindex="-1" aria-label={i18n::t("save.restore_file")} />
                        </div>
                    </section>
                    <details class="save-text-backup">
                        <summary>{i18n::t("save.text_backup")}</summary>
                        <div class="save-text-content">
                            <button class="save-action save-copy" onclick={export_btn} disabled={!p.can_save}>{journey_icon::render("copy")}<span>{i18n::t("save.export")}</span></button>
                            <label for="import-json">{i18n::t("save.import_label")}</label>
                            <textarea id="import-json" value={(*import_text).clone()} oninput={on_input} rows={3} spellcheck="false" />
                            <button class="save-action" onclick={import_btn} disabled={import_text.trim().is_empty()}>{journey_icon::render("upload")}<span>{i18n::t("save.import_button")}</span></button>
                        </div>
                    </details>
                    if !status.is_empty() {<p class="save-notice" role="status" aria-live="polite">{status}</p>}
                </div>
            </div>
        </div>
    }
}
