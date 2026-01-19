#[cfg(target_arch = "wasm32")]
use crate::i18n::set_lang;
use crate::i18n::{locales, t};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub on_open_save: Callback<()>,
    pub on_lang_change: Callback<String>,
    pub current_lang: String,
    pub high_contrast: bool,
    pub on_toggle_hc: Callback<bool>,
}

#[function_component(Header)]
pub fn header(p: &Props) -> Html {
    let on_change = {
        let cb = p.on_lang_change.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: web_sys::Event| {
                if let Some(sel) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
                    set_lang(&sel.value());
                    cb.emit(sel.value());
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = cb;
            Callback::from(|_e: web_sys::Event| {})
        }
    };
    let on_hc_toggle = {
        let cb = p.on_toggle_hc.clone();
        let current = p.high_contrast;
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |_| cb.emit(!current))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (cb, current);
            Callback::from(|_| {})
        }
    };
    let open_save = {
        let cb = p.on_open_save.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |_| cb.emit(()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = cb;
            Callback::from(|_| {})
        }
    };
    html! {
        <header role="banner">
            <a href="#main" class="sr-only">{ t("ui.skip_to_content") }</a>
            <div class="header-content">
                <nav aria-label={t("nav.language")} class="header-left">
                    <label for="lang-select" class="sr-only">{ t("nav.language") }</label>
                    <select id="lang-select" onchange={on_change} value={p.current_lang.clone()} aria-label={t("nav.language")}>
                        { for locales().iter().map(|meta| {
                            let value = meta.code.to_string();
                            let label = meta.name;
                            html! { <option value={value}>{ label }</option> }
                        }) }
                    </select>
                </nav>
                <div class="header-right">
                    <button aria-pressed={p.high_contrast.to_string()} onclick={on_hc_toggle} class="hc-toggle">
                        { if p.high_contrast { t("ui.hc_on") } else { t("ui.hc_off") } }
                    </button>
                    <button id="save-open-btn" onclick={open_save}>{ t("save.header") }</button>
                </div>
            </div>
        </header>
    }
}
