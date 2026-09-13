//! Offline readiness is reported only after the complete build is cached.
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct Snapshot {
    state: String,
    percent: String,
    install_available: bool,
    installed: bool,
}

fn snapshot() -> Snapshot {
    if !cfg!(target_arch = "wasm32") {
        return Snapshot {
            state: "unavailable".into(),
            percent: "0".into(),
            install_available: false,
            installed: false,
        };
    }
    let state =
        web_sys::window().and_then(|w| js_sys::Reflect::get(&w, &"dystrailOffline".into()).ok());
    let read = |key: &str| {
        state
            .as_ref()
            .and_then(|s| js_sys::Reflect::get(s, &key.into()).ok())
    };
    let flag = |key: &str| {
        web_sys::window()
            .and_then(|w| js_sys::Reflect::get(&w, &key.into()).ok())
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    Snapshot {
        state: read("state")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "downloading".into()),
        percent: read("percent")
            .and_then(|v| v.as_f64())
            .map_or_else(|| "0".into(), |n| format!("{:.0}", n.clamp(0.0, 100.0))),
        install_available: flag("dystrailInstallAvailable"),
        installed: flag("dystrailInstalled"),
    }
}

#[function_component(OfflineStatus)]
pub fn offline_status() -> Html {
    crate::i18n::use_language();
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
    let key = format!("offline.{}", status.state);
    let label = crate::i18n::tr(
        &key,
        Some(&std::collections::BTreeMap::from([(
            "percent",
            status.percent.as_str(),
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
    let icon = crate::components::ui::journey_icon::render(match status.state.as_str() {
        "ready" => "offline-ready",
        "error" | "unavailable" => "status-warning",
        "checking" => "sync",
        _ => "download",
    });
    html! {<div class="offline-status" data-offline={status.state.clone()}>
        if matches!(status.state.as_str(),"downloading"|"updating") {<progress class="offline-progress" max="100" value={status.percent.clone()} aria-label={crate::i18n::t("offline.title")} />}
        if status.state=="error" {<button class="offline-retry" onclick={retry}>{crate::i18n::t("offline.retry")}</button>}
        if !status.installed {<span class="pwa-install">
            if status.install_available {<button class="pwa-install-button" onclick={install}>{crate::i18n::t("offline.install")}</button>}
            else {<crate::components::ui::context_help::ContextHelp informational={true} trigger_label={crate::i18n::t("offline.install_steps")} title={crate::i18n::t("offline.install")} text={crate::i18n::t("offline.install_help")} />}
        </span>}
        <div class="offline-readiness"><span class="offline-summary" role="status"><span class="offline-state-icon" aria-hidden="true">{icon}</span><span>{label}</span></span>
        <crate::components::ui::context_help::ContextHelp title={crate::i18n::t("offline.title")} text={crate::i18n::t("offline.help")} /></div>
    </div>}
}
