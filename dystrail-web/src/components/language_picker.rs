use crate::i18n::{locales, set_lang, t};
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_lang_change: Callback<String>,
    pub current_lang: String,
}
#[function_component(LanguagePicker)]
pub fn language_picker(p: &Props) -> Html {
    crate::i18n::use_language();
    let open = use_state(|| false);
    let node = use_node_ref();
    let close = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    crate::components::ui::dismiss::use_outside_dismiss(node.clone(), *open, close);
    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };
    let language_keys = {
        let open = open.clone();
        let node = node.clone();
        Callback::from(move |event: KeyboardEvent| {
            use wasm_bindgen::JsCast;
            let key = event.key();
            if !matches!(
                key.as_str(),
                "ArrowDown" | "ArrowUp" | "Home" | "End" | "Escape"
            ) {
                return;
            }
            event.prevent_default();
            let Some(root) = node.cast::<web_sys::Element>() else {
                return;
            };
            if key == "Escape" {
                open.set(false);
                if let Ok(Some(button)) = root.query_selector("button")
                    && let Ok(button) = button.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = button.focus();
                }
                return;
            }
            if !*open {
                open.set(true);
                return;
            }
            let Ok(options) = root.query_selector_all("[role=option]") else {
                return;
            };
            let count = options.length();
            if count == 0 {
                return;
            }
            let active = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element());
            let current = (0..count)
                .find(|i| {
                    options
                        .get(*i)
                        .is_some_and(|n| active.as_ref().is_some_and(|a| n == *a.as_ref()))
                })
                .unwrap_or(0);
            let next = match key.as_str() {
                "Home" => 0,
                "End" => count - 1,
                "ArrowUp" => (current + count - 1) % count,
                _ => (current + 1) % count,
            };
            if let Some(option) = options
                .get(next)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = option.focus();
            }
        })
    };
    {
        let node = node.clone();
        use_effect_with(*open, move |open| {
            if *open {
                use wasm_bindgen::JsCast;
                if let Some(root) = node.cast::<web_sys::Element>()
                    && let Ok(Some(option)) =
                        root.query_selector("[role=option][aria-selected=true]")
                    && let Ok(option) = option.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = option.focus();
                }
            }
        });
    }
    html! {            <div class="language-picker" ref={node} onkeydown={language_keys}><button aria-haspopup="listbox" aria-expanded={open.to_string()} onclick={toggle} aria-label={t("nav.language")}>{locales().iter().find(|m|m.code==p.current_lang).map_or("English",|m|m.name)}<span aria-hidden="true">{" ▾"}</span></button>
        if *open {<div class="language-options" role="listbox" aria-label={t("nav.language")}>{for locales().iter().map(|meta|{let cb=p.on_lang_change.clone();let open=open.clone();let code=meta.code;html!{<button role="option" aria-selected={(code==p.current_lang).to_string()} onclick={Callback::from(move |_|{set_lang(code);cb.emit(code.to_owned());open.set(false);})}>{meta.name}</button>}})}</div>}
    </div>}
}
