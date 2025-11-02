use js_sys::{Function, Promise};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Response, Storage, Window};

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

/// Yield execution for the requested number of milliseconds.
///
/// # Errors
/// Returns an error if the timer cannot be scheduled or the underlying JavaScript promise rejects.
///
/// # Panics
/// Panics if no browser `window` is available.
#[allow(clippy::future_not_send)] // Wasm futures rely on `JsFuture`, which is not `Send`.
pub async fn sleep_ms(duration_ms: i32) -> Result<(), JsValue> {
    let mut resolve_slot: Option<Function> = None;
    let promise = Promise::new(&mut |resolve, _reject| {
        resolve_slot = Some(resolve);
    });

    let resolve =
        resolve_slot.ok_or_else(|| JsValue::from_str("resolve function should be set"))?;
    let closure = Closure::once(move || {
        let _ = resolve.call0(&JsValue::UNDEFINED);
    });

    let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        duration_ms,
    )?;
    closure.forget();

    JsFuture::from(promise).await?;
    Ok(())
}

/// Perform a fetch request and return the browser `Response`.
///
/// # Errors
/// Returns an error if the fetch request fails or the response cannot be converted to `Response`.
#[allow(clippy::future_not_send)] // Wasm futures rely on `JsFuture`, which is not `Send`.
pub async fn fetch_response(url: &str) -> Result<Response, JsValue> {
    let resp_value = JsFuture::from(window().fetch_with_str(url)).await?;
    resp_value.dyn_into::<Response>()
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
