//! Local backup files use the same save format as copied backups.
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use yew::prelude::*;

pub fn download(text: &str, day: u32) -> Result<(), JsValue> {
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(text));
    let options = web_sys::FilePropertyBag::new();
    options.set_type("application/json");
    let filename = format!("dystrail-day-{day}.json");
    let file = web_sys::File::new_with_str_sequence_and_options(&parts, &filename, &options)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No browser window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document"))?;
    let link = document
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("No document body"))?;
    link.set_hidden(true);
    body.append_child(&link)?;
    let url = match web_sys::Url::create_object_url_with_blob(file.as_ref()) {
        Ok(url) => url,
        Err(error) => {
            link.remove();
            return Err(error);
        }
    };
    link.set_href(&url);
    link.set_download(&filename);
    link.click();
    link.remove();
    let cleanup_url = url.clone();
    let cleanup = Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&cleanup_url);
    });
    if window
        .set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), 1_000)
        .is_err()
    {
        web_sys::Url::revoke_object_url(&url)?;
    }
    Ok(())
}

pub fn file_import(
    on_import: Callback<String>,
    error: UseStateHandle<Option<String>>,
    busy: UseStateHandle<bool>,
) -> Callback<Event> {
    Callback::from(move |event: Event| {
        let Some(input) = event.target_dyn_into::<web_sys::HtmlInputElement>() else {
            return;
        };
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        input.set_value("");
        error.set(None);
        busy.set(true);
        let on_import = on_import.clone();
        let error = error.clone();
        let busy = busy.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let text = wasm_bindgen_futures::JsFuture::from(file.text())
                .await
                .ok()
                .and_then(|value| value.as_string());
            busy.set(false);
            if let Some(text) = text {
                on_import.emit(text);
            } else {
                error.set(Some(crate::i18n::t("save.error")));
            }
        });
    })
}
