//! Explicit breakdown choices, including the cashless route through local contacts.
use super::{aftermath::Aftermath, state::AppState};
use crate::components::ui::action_button::ActionButton;
use crate::{game::repairs::RepairChoice, i18n};
use yew::prelude::*;

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let Some(b) = &gs.breakdown else {
        return Html::default();
    };
    let part = i18n::t(b.part.key());
    let town = crate::game::route::upcoming(gs)
        .next()
        .map_or("D.C.", |s| s.name.as_str());
    html! {<section class="roadside-options" aria-labelledby="repair-title">
        <h2 id="repair-title">{i18n::tr("trail.repair_title",Some(&std::collections::BTreeMap::from([("part",part.as_str())])))}</h2>
        <p>{i18n::tr("trail.repair_context",Some(&std::collections::BTreeMap::from([("town",town)])))}</p>
        <div class="action-grid">{for RepairChoice::ALL.into_iter().map(|choice|{
            let label=i18n::tr(choice.key(),Some(&std::collections::BTreeMap::from([("part",part.as_str()),("cost",i18n::fmt_currency(gs.replacement_cost(b.part)).as_str())])));
            let cost=match choice {
                RepairChoice::Onboard=>format!("{part} −1 · {}",i18n::t("trail.one_hour")),
                RepairChoice::Purchase=>format!("{} −{} · {}",i18n::t("play.cash"),i18n::fmt_currency(gs.replacement_cost(b.part)),i18n::t("trail.ninety_minutes")),
                RepairChoice::Barter=>format!("{} −4 · {}",i18n::t("ux.supplies"),i18n::t("trail.two_hours")),
                RepairChoice::Radio=>format!("{} −{} · {} −{} · {}",i18n::t("ux.sanity"),gs.stats.sanity.min(2),i18n::t("play.morale"),gs.stats.morale.min(1),i18n::t("trail.four_hours")),
            };
            let gain = (100.0-gs.vehicle.health).clamp(0.0,if choice==RepairChoice::Radio {3.0}else{8.0});
            html!{<div class="action-option"><ActionButton disabled={!gs.can_repair(choice)} onclick={choose(app,choice)} {label} detail={format!("{cost} · {} +{}%",i18n::t("play.vehicle"),i18n::fmt_number(f64::from(gain)))} /></div>}
        })}</div>
        <crate::components::ui::context_help::ContextHelp title={i18n::t("play.vehicle")} text={i18n::t("trail.repair_help")} />
    </section>}
}
fn choose(app: &AppState, choice: RepairChoice) -> Callback<MouseEvent> {
    let app = app.clone();
    Callback::from(move |_| {
        if *app.action_lock.borrow() {
            return;
        }
        let Some(mut session) = (*app.session).clone() else {
            return;
        };
        let before = session.state().clone();
        let Some(b) = &before.breakdown else {
            return;
        };
        let part = i18n::t(b.part.key());
        if !session.with_state_mut(|gs| gs.choose_repair(choice)) {
            return;
        }
        let mut report = Aftermath {
            title: i18n::tr(
                "trail.repair_title",
                Some(&std::collections::BTreeMap::from([("part", part.as_str())])),
            ),
            message: i18n::tr(
                "trail.repaired",
                Some(&std::collections::BTreeMap::from([("part", part.as_str())])),
            ),
            scene: crate::components::ui::journey_scene::SceneStage::Breakdown,
            before: before.stats.clone(),
            after: session.state().stats.clone(),
            next: super::aftermath::next_phase(session.state()),
            resources: Vec::new(),
            details: super::receipt::resource_details(&before, session.state()),
        };
        session.with_state_mut(|gs| {
            super::history::record(&before, gs, &mut report, choice.minutes());
        });
        super::history::publish(&app, report, true);
        app.session.set(Some(session));
    })
}
