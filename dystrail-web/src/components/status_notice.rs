//! Action feedback stays in view without moving the page or stealing focus.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub message: String,
    pub on_clear: Callback<()>,
}

#[function_component(StatusNotice)]
pub fn status_notice(p: &Props) -> Html {
    crate::i18n::use_language();
    let clear = p.on_clear.clone();
    use_effect_with(p.message.clone(), move |message| {
        let success = [
            "save.saved",
            "save.loaded",
            "save.exported",
            "result.announce.copied",
        ]
        .iter()
        .any(|key| message == &crate::i18n::t(key));
        let window = web_sys::window();
        let callback = Closure::wrap(Box::new(move || clear.emit(())) as Box<dyn FnMut()>);
        let timer = if success {
            window.as_ref().and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    6000,
                )
                .ok()
            })
        } else {
            None
        };
        move || {
            if let (Some(w), Some(id)) = (window, timer) {
                w.clear_timeout_with_handle(id);
            }
            drop(callback);
        }
    });
    html! {<div class="action-feedback" role="status" aria-live="polite" aria-atomic="true">{&p.message}</div>}
}
