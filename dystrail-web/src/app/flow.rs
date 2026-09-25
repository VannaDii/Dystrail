//! Sustained travel stops at player decisions and never runs in a hidden document.
use super::{Phase, state::AppState};
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TravelSpeed {
    #[default]
    #[serde(alias = "Step")]
    Normal,
    Fast,
}
impl TravelSpeed {
    #[must_use]
    pub const fn duration(self) -> i32 {
        match self {
            Self::Fast => 900,
            Self::Normal => 2600,
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
        && (*app.phase == Phase::Travel || (*app.phase == Phase::Map && *app.map_automatic))
        && app.pending_turn.is_none()
        && app.aftermath.is_none()
        && !super::policy_bulletin::is_pending(app)
        && !super::crossing_presentation::is_pending(app)
        && !*app.show_save
        && !*app.show_abandon
        && !*app.town_open;
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

#[must_use]
pub fn resume(app: &AppState) -> Callback<()> {
    let app = app.clone();
    Callback::from(move |()| {
        if matches!(*app.phase, Phase::Town | Phase::Camp) {
            let Some(session) = app.session.as_ref() else {
                return;
            };
            let next = super::aftermath::next_phase(session.state());
            if next == Phase::Town {
                super::town::depart(&app).emit(());
                return;
            }
            app.phase.set(next);
            app.travel_running.set(next == Phase::Travel);
            return;
        }
        app.travel_running.set(true);
    })
}

pub fn controls(app: &AppState) -> Html {
    let fast = *app.travel_speed == TravelSpeed::Fast;
    let breakdown = app
        .session
        .as_ref()
        .is_some_and(|s| s.state().breakdown.is_some());
    let camping = *app.phase == Phase::Camp && !breakdown;
    let decision = app.aftermath.is_some()
        || super::crossing_presentation::is_pending(app)
        || matches!(*app.phase, Phase::Encounter | Phase::Boss | Phase::AllyLoss);
    let locked = decision || *app.town_open || app.pending_turn.is_some();
    let in_town = app
        .session
        .as_ref()
        .is_some_and(|s| s.state().continuity.route_services.stop.is_some());
    let can_resume = !decision
        && app.pending_turn.is_none()
        && (!*app.town_open || in_town)
        && matches!(*app.phase, Phase::Travel | Phase::Camp | Phase::Town)
        && app.data_ready()
        && app.session.as_ref().is_some_and(|s| {
            s.state().breakdown.is_none()
                && s.state().current_encounter.is_none()
                && s.state().continuity.crew_care.pending.is_none()
                && s.state().continuity.ally_notice.is_none()
                && !s.state().continuity.abandoned
                && s.state().ending.is_none()
        });
    html! {<div class="journey-actions">
    <button class="fast-mode-toggle" role="switch" aria-checked={fast.to_string()} aria-label={crate::i18n::t("journey.fast_mode")} onclick={{let speed=app.travel_speed.clone();Callback::from(move |_|speed.set(if fast {TravelSpeed::Normal}else{TravelSpeed::Fast}))}}><span class="switch-track" aria-hidden="true"><span /></span><span class="journey-action-label">{crate::i18n::t("journey.fast")}</span></button>
    <button class="camp-toggle" aria-pressed={camping.to_string()} disabled={locked || breakdown} onclick={{let app=app.clone();Callback::from(move |_|{
        if app.session.as_ref().is_some_and(|s|s.state().breakdown.is_some()) { return; }
        let next = if camping {
            app.session.as_ref().map_or(Phase::Travel, |s| super::aftermath::next_phase(s.state()))
        } else { Phase::Camp };
        app.travel_running.set(false);
        app.phase.set(next);
    })}}>{crate::components::ui::journey_icon::render("camp")}<span>{crate::i18n::t("ux.camp")}</span></button>
    <button disabled={locked || *app.phase==Phase::Map} onclick={{let app=app.clone();Callback::from(move |_|{app.map_return.set(Some(*app.phase));app.travel_running.set(false);app.map_automatic.set(false);app.phase.set(Phase::Map);})}}>{crate::components::ui::journey_icon::render("route")}<span>{crate::i18n::t("journey.route_button")}</span></button>
        <button class="retro-btn-primary" disabled={!can_resume} onclick={{let resume=resume(app);Callback::from(move |_|resume.emit(()))}}>{crate::components::ui::journey_icon::render("travel")}<span>{crate::i18n::t(if in_town {"journey.depart"} else {"journey.travel_button"})}</span></button>
    </div>}
}
