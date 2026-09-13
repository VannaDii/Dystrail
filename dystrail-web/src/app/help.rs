//! Contextual tips are a persistent player preference, separate from game saves.
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HelpPreference(pub bool);
impl Default for HelpPreference {
    fn default() -> Self {
        Self(true)
    }
}

#[must_use]
pub fn enabled() -> bool {
    if !cfg!(target_arch = "wasm32") {
        return true;
    }
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("dystrail.help").ok().flatten())
        .is_none_or(|value| value != "0")
}

#[must_use]
pub fn change(value: UseStateHandle<bool>) -> Callback<bool> {
    Callback::from(move |enabled| {
        value.set(enabled);
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("dystrail.help", if enabled { "1" } else { "0" });
        }
    })
}
