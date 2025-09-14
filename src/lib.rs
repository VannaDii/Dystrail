#![forbid(unsafe_code)]
use wasm_bindgen::prelude::*;

pub mod a11y;
mod app;
pub mod components;
pub mod game;
pub mod i18n;
pub mod input;
pub mod routes;

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
