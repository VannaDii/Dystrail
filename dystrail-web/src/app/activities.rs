//! Player-initiated gathering and work share the game's receipt and recovery path.
use super::{aftermath::Aftermath, state::AppState};
use crate::{game::activities::Activity, i18n};
use yew::prelude::*;
pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let town = gs.continuity.route_services.stop.is_some();
    let actions = if town {
        Activity::TOWN
    } else {
        Activity::ROADSIDE
    };
    let remaining = gs
        .continuity
        .activities
        .foraged_on
        .map_or(0, |day| day.saturating_add(3).saturating_sub(gs.day));
    html! {<section class="roadside-options" aria-label={i18n::t(if town{"trail.town_work"}else{"trail.gather"})}>
        <h2>{i18n::t(if town{"trail.town_work"}else{"trail.gather"})}<crate::components::ui::context_help::ContextHelp title={i18n::t("trail.gather")} text={i18n::t("trail.gather_help")} /></h2>
        <div class="action-grid">{for actions.into_iter().map(|action|html!{<div class="action-option"><button disabled={!gs.can_activity(action)} onclick={choose(app,action)}>{i18n::t(action.key())}</button><span>{i18n::t(&format!("{}_cost",action.key()))}</span></div>})}</div>
        if !town && remaining>0 {<p class="action-availability">{i18n::tr("trail.gather_wait",Some(&std::collections::BTreeMap::from([("days",remaining.to_string().as_str())])))}</p>}
        if town && gs.continuity.activities.worked_at==gs.continuity.route_services.stop {<p class="action-availability">{i18n::t("trail.work_done")}</p>}
    </section>}
}
fn choose(app: &AppState, action: Activity) -> Callback<MouseEvent> {
    let app = app.clone();
    Callback::from(move |_| {
        if *app.action_lock.borrow() {
            return;
        }
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        let before = session.state().clone();
        if !session.with_state_mut(|gs| gs.perform_activity(action)) {
            return;
        }
        let scene = if before.continuity.route_services.stop.is_some() {
            crate::components::ui::journey_scene::SceneStage::Town
        } else {
            crate::components::ui::journey_scene::SceneStage::Camp
        };
        let mut report = Aftermath {
            title: i18n::t(action.key()),
            message: i18n::t(&format!("{}_done", action.key())),
            scene,
            before: before.stats.clone(),
            after: session.state().stats.clone(),
            details: super::receipt::resource_details(&before, session.state()),
            next: super::aftermath::next_phase(session.state()),
        };
        session.with_state_mut(|gs| {
            super::history::record(&before, gs, &mut report, action.minutes());
        });
        super::history::publish(&app, report, true);
        app.travel_running.set(false);
        app.session.set(Some(session));
    })
}
