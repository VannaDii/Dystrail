//! A cancellable automatic map preview that does not restart on each travel tick.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_complete: Callback<()>,
}

#[function_component(MapCountdown)]
pub fn map_countdown(p: &Props) -> Html {
    crate::i18n::use_language();
    let seconds = use_state(|| 3_u8);
    let latest = use_mut_ref(|| p.on_complete.clone());
    *latest.borrow_mut() = p.on_complete.clone();
    let displayed = seconds.clone();
    use_effect_with((), move |()| {
        let mut remaining = 3_u8;
        let callback = Closure::wrap(Box::new(move || {
            if remaining == 0 {
                return;
            }
            remaining -= 1;
            displayed.set(remaining);
            if remaining == 0 {
                latest.borrow().emit(());
            }
        }) as Box<dyn FnMut()>);
        let window = web_sys::window();
        let timer = window.as_ref().and_then(|w| {
            w.set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                1000,
            )
            .ok()
        });
        move || {
            if let (Some(w), Some(id)) = (window, timer) {
                w.clear_interval_with_handle(id);
            }
            drop(callback);
        }
    });
    html! {<div class="map-countdown">
        <span>{crate::i18n::tr("route.auto_close",Some(&std::collections::BTreeMap::from([("seconds",seconds.to_string().as_str())])))}</span>
        <progress max="3" value={seconds.to_string()} aria-label={crate::i18n::t("route.auto_progress")} />
    </div>}
}
