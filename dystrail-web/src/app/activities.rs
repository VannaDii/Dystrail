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
        <div class="action-grid">{for Activity::TOWN.into_iter().map(|action|html!{<div class="action-option">{activity_offer(gs,action)}<ActionButton disabled={!gs.can_activity(action)} onclick={choose(app,action)} label={activity_copy(gs,action,"choice_0",&i18n::t(action.key()))} detail={activity_detail(gs,action)} /></div>})}</div>
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
            {activity_offer(gs,action)}
            <ActionButton disabled={!gs.can_activity(action)} onclick={choose(app,action)} label={activity_copy(gs,action,"choice_0",&i18n::t(action.key()))} detail={activity_detail(gs,action)} />

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
        session.with_state_mut(|gs| super::visual_content::record_activity(&before, gs, action));
        let unit = super::visual_content::activity_unit(&before, action);
        let scene = if unit == "ACT-FOODWORK-A" {
            crate::components::ui::journey_scene::SceneStage::EncounterOutcome { unit, choice: 0 }
        } else if before.continuity.route_services.stop.is_some() {
            crate::components::ui::journey_scene::SceneStage::Town
        } else {
            crate::components::ui::journey_scene::SceneStage::Camp
        };
        let next = super::aftermath::next_phase(session.state());
        let mut report = Aftermath {
            title: activity_copy(&before, action, "name", &i18n::t(action.key())),
            message: activity_copy(
                &before,
                action,
                "log_0",
                &i18n::t(&format!("{}_done", action.key())),
            ),
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

fn activity_detail(gs: &crate::game::GameState, action: Activity) -> String {
    use crate::game::data::Effects;
    let effects = match action {
        Activity::Forage => Effects {
            supplies: 2,
            sanity: 1,
            ..Effects::default()
        },
        Activity::Glean => Effects {
            supplies: 4,
            hp: -1,
            ..Effects::default()
        },
        Activity::WorkSupplies => Effects {
            supplies: 4,
            sanity: -1,
            ..Effects::default()
        },
        Activity::WorkCash => Effects {
            cash_cents: 1800,
            sanity: -1,
            ..Effects::default()
        },
    };
    let mut detail = crate::components::ui::choice_effects::describe(&effects, &gs.stats);
    let hours = i18n::fmt_number(f64::from(action.minutes()) / 60.0);
    detail.push(i18n::tr(
        "qualitative.hours",
        Some(&std::collections::BTreeMap::from([(
            "hours",
            hours.as_str(),
        )])),
    ));
    detail.join(" · ")
}

fn activity_copy(
    gs: &crate::game::GameState,
    action: Activity,
    field: &str,
    fallback: &str,
) -> String {
    i18n::encounter_text(
        &super::visual_content::activity_unit(gs, action),
        field,
        fallback,
    )
}

fn activity_offer(gs: &crate::game::GameState, action: Activity) -> Html {
    let description = activity_copy(gs, action, "desc", "");
    if description.is_empty() {
        return Html::default();
    }
    html! {<p class="activity-offer">{description}</p>}
}
