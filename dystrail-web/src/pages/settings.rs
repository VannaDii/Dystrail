use crate::i18n::locales;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct SettingsPageProps {
    pub current_lang: String,
    pub high_contrast: bool,
    pub on_lang_change: Callback<String>,
    pub on_toggle_hc: Callback<bool>,
    pub on_back: Callback<()>,
}

#[function_component(SettingsPage)]
pub fn settings_page(props: &SettingsPageProps) -> Html {
    let container_ref = use_node_ref();
    let on_back_key = props.on_back.clone();
    let on_back_click = props.on_back.clone();
    let on_keydown = {
        let on_back = on_back_key;
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Escape" {
                    on_back.emit(());
                    e.prevent_default();
                }
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = on_back;
            Callback::from(|_e: web_sys::KeyboardEvent| {})
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        let container_ref = container_ref.clone();
        use_effect_with((), move |()| {
            if let Some(el) = container_ref.cast::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
            || {}
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = &container_ref;
    }

    let on_change = {
        let cb = props.on_lang_change.clone();
        #[cfg(target_arch = "wasm32")]
        {
            Callback::from(move |e: web_sys::Event| {
                if let Some(sel) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
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

    let on_toggle = {
        let cb = props.on_toggle_hc.clone();
        let current = props.high_contrast;
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

    html! {
        <div
            class="min-h-screen flex items-center justify-center bg-base-300 font-sans shell-screen"
            onkeydown={on_keydown}
            tabindex="0"
            ref={container_ref}
            data-testid="settings-screen"
        >
            <div class="card border border-base-content bg-base-200 w-[420px] max-w-full rounded-none shadow-none shell-card">
                <div class="card-body items-center text-center gap-4">
                    <h1 class="text-2xl font-bold">{ crate::i18n::t("settings.title") }</h1>
                    <p class="text-xs opacity-60">{ crate::i18n::t("settings.subtitle") }</p>

                    <div class="w-full text-left space-y-4">
                        <div class="space-y-2">
                            <label for="settings-lang" class="text-xs uppercase tracking-wide opacity-70">{ crate::i18n::t("settings.language_label") }</label>
                            <select
                                id="settings-lang"
                                class="select select-bordered w-full font-sans shell-input"
                                onchange={on_change}
                                value={props.current_lang.clone()}
                                data-testid="settings-language"
                            >
                                { for locales().iter().map(|meta| html! { <option value={meta.code}>{ meta.name }</option> }) }
                            </select>
                        </div>

                        <div class="flex items-center justify-between">
                            <span class="text-xs uppercase tracking-wide opacity-70">{ crate::i18n::t("ui.hc_toggle") }</span>
                            <input
                                id="settings-hc"
                                type="checkbox"
                                class="toggle"
                                checked={props.high_contrast}
                                onclick={on_toggle}
                                aria-label={crate::i18n::t("ui.hc_toggle")}
                                data-testid="settings-contrast"
                            />
                        </div>
                    </div>

                    <button class="btn btn-ghost w-full justify-start rounded-none text-left normal-case font-sans shell-btn" onclick={Callback::from(move |_| on_back_click.emit(()))} data-testid="settings-back">
                        { crate::i18n::t("ui.back") }
                    </button>
                </div>
            </div>
        </div>
    }
}
