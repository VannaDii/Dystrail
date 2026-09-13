//! Capability-based sharing never needs game credentials or a posting service.
use super::image::ShareImage;
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};

fn navigator() -> Option<JsValue> {
    web_sys::window().map(|w| w.navigator().into())
}
fn method(name: &str) -> Option<(JsValue, Function)> {
    let navigator = navigator()?;
    let function = Reflect::get(&navigator, &name.into())
        .ok()?
        .dyn_into::<Function>()
        .ok()?;
    Some((navigator, function))
}
fn payload(image: &ShareImage, text: &str) -> Result<Object, JsValue> {
    let parts = Array::new();
    parts.push(&image.blob);
    let options = web_sys::FilePropertyBag::new();
    options.set_type("image/png");
    let file =
        web_sys::File::new_with_blob_sequence_and_options(&parts, "dystopian-trail.png", &options)?;
    let files = Array::new();
    files.push(&file);
    let data = Object::new();
    Reflect::set(&data, &"files".into(), &files)?;
    Reflect::set(&data, &"title".into(), &"Dystopian Trail".into())?;
    Reflect::set(&data, &"text".into(), &text.into())?;
    Ok(data)
}
#[must_use]
pub fn supports_files(image: &ShareImage) -> bool {
    let Some((navigator, can_share)) = method("canShare") else {
        return false;
    };
    method("share").is_some()
        && payload(image, "Dystopian Trail")
            .ok()
            .and_then(|data| can_share.call1(&navigator, &data).ok())
            .is_some_and(|v| v.as_bool() == Some(true))
}
/// Invoke synchronously inside the click callback to preserve browser user activation.
pub fn start(image: &ShareImage, text: &str) -> Result<Promise, JsValue> {
    let (navigator, share) =
        method("share").ok_or_else(|| JsValue::from_str("Sharing unavailable"))?;
    share
        .call1(&navigator, &payload(image, text)?.into())?
        .dyn_into::<Promise>()
}
#[must_use]
pub fn cancelled(error: &JsValue) -> bool {
    Reflect::get(error, &"name".into())
        .ok()
        .and_then(|v| v.as_string())
        .is_some_and(|name| name == "AbortError")
}
#[must_use]
pub fn intent(base: &str, text: &str) -> String {
    format!("{base}{}", js_sys::encode_uri_component(text))
}
