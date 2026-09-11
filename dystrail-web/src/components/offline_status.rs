//! Offline readiness is reported only after the complete build is cached.
use yew::prelude::*;

fn snapshot() -> (String, String) {
    if !cfg!(target_arch = "wasm32") {
        return ("unavailable".into(), "0".into());
    }
    let state =
        web_sys::window().and_then(|w| js_sys::Reflect::get(&w, &"dystrailOffline".into()).ok());
    let read = |key: &str| {
        state
            .as_ref()
            .and_then(|s| js_sys::Reflect::get(s, &key.into()).ok())
    };
    (
        read("state")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "downloading".into()),
        read("percent")
            .and_then(|v| v.as_f64())
            .map_or_else(|| "0".into(), |n| format!("{n:.0}")),
    )
}

#[function_component(OfflineStatus)]
pub fn offline_status() -> Html {
    let status = use_state(snapshot);
    {
        let status = status.clone();
        use_effect_with((), move |()| {
            use wasm_bindgen::{JsCast, closure::Closure};
            let callback =
                Closure::wrap(
                    Box::new(move |_event: web_sys::Event| status.set(snapshot()))
                        as Box<dyn FnMut(web_sys::Event)>,
                );
            let window = web_sys::window();
            if let Some(w) = &window {
                let _ = w.add_event_listener_with_callback(
                    "dystrail-offline",
                    callback.as_ref().unchecked_ref(),
                );
            }
            move || {
                if let Some(w) = window {
                    let _ = w.remove_event_listener_with_callback(
                        "dystrail-offline",
                        callback.as_ref().unchecked_ref(),
                    );
                }
                drop(callback);
            }
        });
    }
    let key = format!("offline.{}", status.0);
    let label = crate::i18n::tr(
        &key,
        Some(&std::collections::BTreeMap::from([(
            "percent",
            status.1.as_str(),
        )])),
    );
    let retry = Callback::from(move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(event) = web_sys::Event::new("dystrail-offline-retry")
        {
            let _ = window.dispatch_event(&event);
        }
    });
    let install = Callback::from(move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(event) = web_sys::Event::new("dystrail-install")
        {
            let _ = window.dispatch_event(&event);
        }
    });
    let install_available = cfg!(target_arch = "wasm32")
        && web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w, &"dystrailInstallAvailable".into()).ok())
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    html! {<div class="offline-status" data-offline={status.0.clone()}>
        <span role="status">{label}</span>
        <crate::components::ui::context_help::ContextHelp title={crate::i18n::t("offline.title")} text={crate::i18n::t("offline.help")} />
        if status.0=="error" {<button onclick={retry}>{crate::i18n::t("offline.retry")}</button>}
        <span class="pwa-install">
            if install_available {<button onclick={install}>{crate::i18n::t("offline.install")}</button>}
            else {<crate::components::ui::context_help::ContextHelp icon={"↗".to_owned()} title={crate::i18n::t("offline.install")} text={crate::i18n::t("offline.install_help")} />}
        </span>
    </div>}
}
