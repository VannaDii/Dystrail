//! Sustained travel stops at player decisions and never runs in a hidden document.
use super::{Phase, state::AppState};
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TravelSpeed {
    #[default]
    Normal,
    Fast,
    Step,
}
impl TravelSpeed {
    #[must_use]
    pub const fn duration(self) -> i32 {
        match self {
            Self::Fast => 900,
            Self::Normal | Self::Step => 2600,
        }
    }
}

#[must_use]
pub fn visible() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .is_some_and(|d| !d.hidden())
}

#[hook]
pub fn use_travel_flow(app: &AppState) {
    let advance = super::view::handlers::build_travel(app);
    let running = *app.travel_running;
    let ready = running
        && *app.phase == Phase::Travel
        && app.pending_turn.is_none()
        && app.aftermath.is_none()
        && !*app.show_save
        && !*app.show_abandon
        && !*app.town_open
        && !*app.weather_notice;
    let revision = app
        .session
        .as_ref()
        .map_or(0, |s| s.state().continuity.journal.len());
    use_effect_with((ready, revision), move |(ready, _)| {
        let callback = Closure::wrap(Box::new(move || {
            if visible() {
                advance.emit(());
            }
        }) as Box<dyn FnMut()>);
        let window = web_sys::window();
        let id = if *ready {
            window.as_ref().and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    180,
                )
                .ok()
            })
        } else {
            None
        };
        move || {
            if let (Some(w), Some(id)) = (window, id) {
                w.clear_timeout_with_handle(id);
            }
            drop(callback);
        }
    });
    let running = app.travel_running.clone();
    use_effect_with((), move |()| {
        let callback = Closure::wrap(Box::new(move || {
            if !visible() {
                running.set(false);
            }
        }) as Box<dyn FnMut()>);
        let document = web_sys::window().and_then(|w| w.document());
        if let Some(d) = &document {
            let _ = d.add_event_listener_with_callback(
                "visibilitychange",
                callback.as_ref().unchecked_ref(),
            );
        }
        move || {
            if let Some(d) = document {
                let _ = d.remove_event_listener_with_callback(
                    "visibilitychange",
                    callback.as_ref().unchecked_ref(),
                );
            }
            drop(callback);
        }
    });
}

#[hook]
pub fn use_stage_entry(app: &AppState) {
    let key = (*app.phase, *app.town_open, app.aftermath.is_some());
    use_effect_with(key, |_| {
        if let Some(w) = web_sys::window() {
            w.scroll_to_with_x_and_y(0.0, 0.0);
        }
    });
}

#[must_use]
pub fn resume(app: &AppState) -> Callback<()> {
    let app = app.clone();
    let advance = super::view::handlers::build_travel(&app);
    Callback::from(move |()| {
        if *app.travel_speed == TravelSpeed::Step {
            advance.emit(());
        } else {
            app.travel_running.set(true);
        }
    })
}

pub fn controls(app: &AppState) -> Html {
    html! {<div class="journey-controls"><div class="speed-controls" role="group" aria-label={crate::i18n::t("journey.speed")}>
        {for [(TravelSpeed::Normal,"journey.normal"),(TravelSpeed::Fast,"journey.fast"),(TravelSpeed::Step,"journey.step")].into_iter().map(|(speed,key)|{
            let app=app.clone(); let selected=*app.travel_speed==speed;
            html!{<button aria-pressed={selected.to_string()} onclick={Callback::from(move |_|{app.travel_speed.set(speed);if speed==TravelSpeed::Step {app.travel_running.set(false);}})}>{crate::i18n::t(key)}</button>}
        })}
    </div><button onclick={{let app=app.clone();Callback::from(move |_|{app.travel_running.set(false);app.phase.set(Phase::Camp);})}}>{crate::i18n::t("ux.camp")}</button>
    <button onclick={{let app=app.clone();Callback::from(move |_|{app.travel_running.set(false);app.phase.set(Phase::Map);})}}>{crate::i18n::t("journey.map")}</button></div>}
}
