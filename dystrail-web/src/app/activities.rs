//! Player-initiated gathering and work share the game's receipt and recovery path.
use super::{aftermath::Aftermath, state::AppState};
use crate::components::ui::action_button::ActionButton;
use crate::{
    game::activities::{Activity, FORAGE_COOLDOWN_DAYS},
    i18n,
};
use yew::prelude::*;
pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let town = gs.continuity.route_services.stop.is_some();
    if !town {
        return Html::default();
    }
    html! {<section class="roadside-options" aria-label={i18n::t(if town{"trail.town_work"}else{"trail.gather"})}>
        <h2>{i18n::t(if town{"trail.town_work"}else{"trail.gather"})}<crate::components::ui::context_help::ContextHelp title={i18n::t("trail.gather")} text={i18n::t("trail.gather_help")} /></h2>
        <div class="action-grid">{for Activity::TOWN.into_iter().map(|action|html!{<div class="action-option"><ActionButton disabled={!gs.can_activity(action)} onclick={choose(app,action)} label={i18n::t(action.key())} detail={i18n::t(&format!("{}_cost",action.key()))} /></div>})}</div>
        if town && gs.continuity.activities.worked_at==gs.continuity.route_services.stop {<p class="action-availability">{i18n::t("trail.work_done")}</p>}
    </section>}
}
pub fn render_camp(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    if gs.continuity.route_services.stop.is_some() {
        return Html::default();
    }
    let remaining = gs.forage_cooldown_days();
    html! {<>{for Activity::ROADSIDE.into_iter().map(|action|html!{
        <div class="camp-action camp-gather">
            <ActionButton disabled={!gs.can_activity(action)} onclick={choose(app,action)} label={i18n::t(action.key())} detail={i18n::t(&format!("{}_cost",action.key()))} />

            {gather_blocker(gs,action).map_or_else(
                || crate::components::ui::camp_panel::cooldown(remaining,FORAGE_COOLDOWN_DAYS,"trail.gather_wait"),
                |reason| html!{<p class="camp-action-cost">{i18n::t(reason)}</p>}
            )}
        </div>
    })}</>}
}

fn gather_blocker(gs: &crate::game::GameState, action: Activity) -> Option<&'static str> {
    if gs.breakdown.is_some() {
        Some("trail.gather_repair")
    } else if gs.continuity.crew_care.pending.is_some() {
        Some("trail.gather_care")
    } else if gs.stats.supplies > if action == Activity::Forage { 18 } else { 16 } {
        Some("trail.gather_capacity")
    } else if action == Activity::Glean && gs.stats.hp <= 1 {
        Some("trail.gather_health")
    } else {
        None
    }
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
        let next = super::aftermath::next_phase(session.state());
        let mut report = Aftermath {
            title: i18n::t(action.key()),
            message: i18n::t(&format!("{}_done", action.key())),
            scene,
            before: before.stats.clone(),
            after: session.state().stats.clone(),
            resources: Vec::new(),
            details: super::receipt::resource_details(&before, session.state()),
            next: if Activity::ROADSIDE.contains(&action) && next == super::Phase::Travel {
                super::Phase::Camp
            } else {
                next
            },
        };
        session.with_state_mut(|gs| {
            super::history::record(&before, gs, &mut report, 0);
        });
        super::history::publish(&app, report, true);
        app.travel_running.set(false);
        app.session.set(Some(session));
    })
}
