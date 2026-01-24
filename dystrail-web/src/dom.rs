#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Storage, Window};

/// Retrieve the global `window` object, if available.
#[must_use]
#[cfg(target_arch = "wasm32")]
pub fn window() -> Option<Window> {
    web_sys::window()
}

/// Retrieve the global `window` object, if available.
#[must_use]
#[cfg(not(target_arch = "wasm32"))]
pub const fn window() -> Option<Window> {
    None
}

/// Retrieve the document object for DOM interactions, if available.
#[must_use]
pub fn document() -> Option<Document> {
    window().and_then(|win| win.document())
}

/// Convert a JavaScript value into a readable string for error reporting.
#[must_use]
pub fn js_error_message(value: &JsValue) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        value
            .as_string()
            .or_else(|| {
                value
                    .dyn_ref::<js_sys::Error>()
                    .map(|err| err.message().into())
            })
            .unwrap_or_else(|| format!("{value:?}"))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = value;
        String::from("js_error")
    }
}

/// Log an error message to the browser console.
#[cfg(target_arch = "wasm32")]
pub fn console_error(message: &str) {
    web_sys::console::error_1(&JsValue::from(message));
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn console_error(message: &str) {
    let _ = message;
}

/// Access the browser `localStorage` handle.
///
/// # Errors
/// Returns an error if the browser window cannot be accessed or `localStorage` is unavailable.
#[cfg(target_arch = "wasm32")]
pub fn local_storage() -> Result<Storage, JsValue> {
    window()
        .ok_or_else(|| js_error_value("window unavailable"))?
        .local_storage()?
        .ok_or_else(|| js_error_value("localStorage unavailable"))
}

#[cfg(not(target_arch = "wasm32"))]
/// Access the browser `localStorage` handle.
///
/// # Errors
/// Returns an error because `localStorage` is unavailable on non-wasm targets.
pub const fn local_storage() -> Result<Storage, JsValue> {
    Err(js_error_value("window unavailable"))
}

#[cfg(target_arch = "wasm32")]
fn js_error_value(message: &str) -> JsValue {
    JsValue::from_str(message)
}

#[cfg(not(target_arch = "wasm32"))]
const fn js_error_value(_message: &str) -> JsValue {
    JsValue::NULL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn window_and_document_return_none_on_host() {
        assert!(window().is_none());
        assert!(document().is_none());
    }

    #[test]
    fn js_error_message_prefers_string() {
        #[cfg(target_arch = "wasm32")]
        {
            let msg = js_error_message(&JsValue::from_str("boom"));
            assert_eq!(msg, "boom");
        }
    }

    #[test]
    fn js_error_message_falls_back_to_debug() {
        let value = JsValue::NULL;
        let msg = js_error_message(&value);
        #[cfg(target_arch = "wasm32")]
        {
            assert_eq!(msg, format!("{value:?}"));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            assert_eq!(msg, "js_error");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn local_storage_errors_without_window() {
        let err = local_storage().unwrap_err();
        #[cfg(target_arch = "wasm32")]
        {
            assert_eq!(js_error_message(&err), "window unavailable");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            assert_eq!(js_error_message(&err), "js_error");
        }
    }

    #[test]
    fn console_error_is_safe_on_host() {
        console_error("test console error");
    }
}
