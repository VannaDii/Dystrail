use crate::i18n::{set_lang, t};
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub on_open_save: Callback<()>,
    pub on_lang_change: Callback<String>,
    pub current_lang: String,
}

#[function_component(Header)]
pub fn header(p: &Props) -> Html {
    let on_change = {
        let cb = p.on_lang_change.clone();
        Callback::from(move |e: web_sys::Event| {
            if let Some(sel) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
            {
                set_lang(&sel.value());
                cb.emit(sel.value());
            }
        })
    };
    let open_save = {
        let cb = p.on_open_save.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {
        <header role="banner">
            <a href="#main" class="sr-only">{ t("ui.skip_to_content") }</a>
            <div class="header-content">
                <nav aria-label={t("nav.language")} class="header-left">
                    <label for="lang-select" class="sr-only">{ t("nav.language") }</label>
                    <select id="lang-select" onchange={on_change} value={p.current_lang.clone()} aria-label={t("nav.language")}>
                        <option value="en">{"English"}</option>
                        <option value="zh">{"中文"}</option>
                        <option value="hi">{"हिन्दी"}</option>
                        <option value="es">{"Español"}</option>
                        <option value="fr">{"Français"}</option>
                        <option value="ar">{"العربية"}</option>
                        <option value="bn">{"বাংলা"}</option>
                        <option value="pt">{"Português"}</option>
                        <option value="ru">{"Русский"}</option>
                        <option value="ja">{"日本語"}</option>
                        <option value="it">{"Italiano"}</option>
                    </select>
                </nav>
                <div class="header-right">
                    <button id="save-open-btn" onclick={open_save}>{ t("save.header") }</button>
                </div>
            </div>
        </header>
    }
}
