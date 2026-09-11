//! Deferred presentation of a single already-resolved engine action.
use super::{aftermath::Aftermath, state::AppState};
use crate::{game::JourneySession, i18n};
use std::rc::Rc;
use yew::prelude::*;
pub struct PendingTurn {
    pub session: JourneySession,
    pub report: Aftermath,
    pub logs: Vec<String>,
}

pub fn render_transit(state: &AppState, pending: &Rc<PendingTurn>) -> Html {
    let Some(_) = state.session.as_ref() else {
        return Html::default();
    };
    let complete = {
        let state = state.clone();
        let pending = pending.clone();
        Callback::from(move |()| {
            if !*state.action_lock.borrow() {
                return;
            }
            *state.action_lock.borrow_mut() = false;
            let old_weather = state
                .session
                .as_ref()
                .map(|s| s.state().weather_state.today);
            let weather = pending.session.state().weather_state.today;
            state.weather_notice.set(
                old_weather != Some(weather) && weather != crate::game::weather::Weather::Clear,
            );
            let gs = pending.session.state();
            let changed_weather =
                old_weather != Some(weather) && weather != crate::game::weather::Weather::Clear;
            let next = if changed_weather {
                pending.report.next
            } else {
                super::map::after_transit(gs, pending.report.next)
            };
            if next != super::Phase::Travel
                || changed_weather
                || gs.breakdown.is_some()
                || gs.stats.supplies <= 2
                || gs.stats.hp <= 2
                || gs.stats.sanity <= 2
                || gs.stats.pants >= 80
                || *state.travel_speed == super::flow::TravelSpeed::Step
                || !super::flow::visible()
            {
                state.travel_running.set(false);
            }
            state.logs.set(pending.logs.clone());
            state.session.set(Some(pending.session.clone()));
            state.phase.set(next);
            state.last_turn.set(Some(pending.report.clone()));
            state.pending_turn.set(None);
        })
    };
    html! {<TravelTransition key={pending.session.state().continuity.journal.len()} on_complete={complete} duration={state.travel_speed.duration()} />}
}

#[derive(Properties, Clone)]
struct Props {
    on_complete: Callback<()>,
    duration: i32,
}
impl PartialEq for Props {
    fn eq(&self, o: &Self) -> bool {
        self.on_complete == o.on_complete && self.duration == o.duration
    }
}
#[function_component(TravelTransition)]
fn travel_transition(p: &Props) -> Html {
    // Keep one cancelable timer. Leaving this screen cannot commit a stale turn.
    {
        let done = p.on_complete.clone();
        let duration = p.duration;
        use_effect_with((), move |()| {
            use wasm_bindgen::{JsCast, closure::Closure};
            let callback = Closure::wrap(Box::new(move || done.emit(())) as Box<dyn FnMut()>);
            let window = web_sys::window();
            let timer = window.as_ref().and_then(|win| {
                win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    duration,
                )
                .ok()
            });
            move || {
                if let (Some(win), Some(id)) = (window, timer) {
                    win.clear_timeout_with_handle(id);
                }
                drop(callback);
            }
        });
    }
    Html::default()
}

pub fn render_last_turn(report: &Aftermath) -> Html {
    let changes = report.changes();
    html! {<aside class="turn-receipt" aria-label={i18n::t("play.last_turn")}>
        <span class="receipt-mark" aria-hidden="true">{"▤"}</span>
        <div class="receipt-body"><div class="receipt-heading"><strong>{&report.title}</strong>
        <div class="receipt-changes">{for changes.iter().map(|(key,n,bad)|html!{<span class={if *bad{"harmful"}else{"helpful"}}>{format!("{} {n:+}",i18n::t(key))}</span>})}</div></div>
        <p>{&report.message}</p>
        <dl class="receipt-details">{for report.details.iter().filter(|(name,_)|name!=&i18n::t("journey.when")).map(|(name,value)|html!{<div><dt>{name}</dt><dd>{value}</dd></div>})}</dl></div>
    </aside>}
}
