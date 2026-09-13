//! Keep brief condition notifications across scene changes without interrupting travel.
use crate::game::{Weather, exec_orders::ExecOrder};
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct WeatherNotification(pub bool);

#[derive(Clone, PartialEq, Eq, Default)]
pub struct PolicyNotification(pub bool);

#[derive(Properties, PartialEq)]
pub struct Props {
    pub weather: Option<Weather>,
    pub policy: Option<ExecOrder>,
    pub children: Children,
}

#[function_component(WeatherStatus)]
pub fn weather_status(p: &Props) -> Html {
    let weather = use_condition_notification(p.weather, false);
    let policy = use_condition_notification(p.policy, true);
    html! {
        <ContextProvider<WeatherNotification> context={WeatherNotification(weather)}>
            <ContextProvider<PolicyNotification> context={PolicyNotification(policy)}>
                {for p.children.iter()}
            </ContextProvider<PolicyNotification>>
        </ContextProvider<WeatherNotification>>
    }
}

#[hook]
fn use_condition_notification<T: Copy + PartialEq + 'static>(
    current: Option<T>,
    notify_on_activation: bool,
) -> bool {
    let previous = use_mut_ref(|| current);
    let notifying = use_state(|| false);
    {
        let notifying = notifying.clone();
        use_effect_with(current, move |current| {
            let changed = current.is_some()
                && (notify_on_activation || previous.borrow().is_some())
                && *previous.borrow() != *current;
            *previous.borrow_mut() = *current;
            notifying.set(changed);
            let callback =
                Closure::wrap(Box::new(move || notifying.set(false)) as Box<dyn FnMut()>);
            let window = web_sys::window();
            let timer = if changed {
                window.as_ref().and_then(|w| {
                    w.set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        5_000,
                    )
                    .ok()
                })
            } else {
                None
            };
            move || {
                if let (Some(window), Some(timer)) = (window, timer) {
                    window.clear_timeout_with_handle(timer);
                }
                drop(callback);
            }
        });
    }
    *notifying
}
