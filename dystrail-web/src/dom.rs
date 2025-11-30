use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Storage, Window};

/// Retrieve the global `window` object.
///
/// # Panics
/// Panics if executed outside of a browser context where `window` is unavailable.
#[must_use]
pub fn window() -> Window {
    web_sys::window().expect("`window` should be available in web context")
}

/// Retrieve the document object for DOM interactions.
///
/// # Panics
/// Panics when the document cannot be accessed from the current browser window.
#[must_use]
pub fn document() -> Document {
    window()
        .document()
        .expect("`document` should exist in browser context")
}

/// Convert a JavaScript value into a readable string for error reporting.
#[must_use]
pub fn js_error_message(value: &JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            value
                .dyn_ref::<js_sys::Error>()
                .map(|err| err.message().into())
        })
        .unwrap_or_else(|| format!("{value:?}"))
}

/// Log an error message to the browser console.
pub fn console_error(message: &str) {
    web_sys::console::error_1(&JsValue::from(message));
}

/// Access the browser `localStorage` handle.
///
/// # Errors
/// Returns an error if the browser window cannot be accessed or `localStorage` is unavailable.
pub fn local_storage() -> Result<Storage, JsValue> {
    window()
        .local_storage()?
        .ok_or_else(|| JsValue::from_str("localStorage unavailable"))
}
