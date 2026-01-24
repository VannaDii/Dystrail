#![forbid(unsafe_code)]
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod a11y;
pub mod app;
pub mod components;
pub mod dom;
pub mod game;
pub mod i18n;
#[cfg(test)]
mod i18n_tests;
pub mod input;
pub mod pages;
pub mod paths;
pub mod router;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    // Ensure <html lang, dir> are set at startup according to saved locale
    crate::i18n::set_lang(&crate::i18n::current_lang());
    // Apply saved high-contrast preference
    if crate::a11y::high_contrast_enabled() {
        crate::a11y::set_high_contrast(true);
    }
    yew::Renderer::<app::App>::new().render();
}
