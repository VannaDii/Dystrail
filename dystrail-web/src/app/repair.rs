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
    let unit = unit(gs).unwrap();
    let town = crate::game::route::upcoming(gs)
        .next()
        .map_or("D.C.", |s| s.name.as_str());
    html! {<section class="roadside-options" aria-labelledby="repair-title">
        <h2 id="repair-title">{i18n::encounter_text(&unit,"name",&i18n::tr("trail.repair_title",Some(&std::collections::BTreeMap::from([("part",part.as_str())]))))}</h2>
        <p>{i18n::encounter_text(&unit,"desc","")}</p>
        <p>{i18n::tr("trail.repair_context",Some(&std::collections::BTreeMap::from([("town",town)])))}</p>
        <div class="action-grid">{for RepairChoice::ALL.into_iter().map(|choice|{
            let label=i18n::tr(choice.key(),Some(&std::collections::BTreeMap::from([("part",part.as_str()),("cost",i18n::fmt_currency(gs.replacement_cost(b.part)).as_str())])));
            use crate::components::ui::choice_effects::qualitative_stat;
            let vehicle=qualitative_stat("play.vehicle","gain");
            let detail=match choice {
                RepairChoice::Onboard=>format!("{vehicle} · {}",i18n::t("trail.one_hour")),
                RepairChoice::Purchase=>format!("{} · {vehicle} · {}",qualitative_stat("play.cash","cost"),i18n::t("trail.ninety_minutes")),
                RepairChoice::Barter=>format!("{} · {vehicle} · {}",qualitative_stat("ux.supplies","cost"),i18n::t("trail.two_hours")),
                RepairChoice::Radio=>format!("{} · {} · {vehicle} · {}",qualitative_stat("ux.sanity","cost"),qualitative_stat("play.morale","cost"),i18n::t("trail.four_hours")),
            };
            html!{<div class="action-option"><ActionButton disabled={!gs.can_repair(choice)} onclick={choose(app,choice)} {label} {detail} /></div>}
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
            title: i18n::encounter_text(
                &unit(&before).unwrap(),
                "name",
                &i18n::tr(
                    "trail.repair_title",
                    Some(&std::collections::BTreeMap::from([("part", part.as_str())])),
                ),
            ),
            message: i18n::encounter_text(
                &unit(&before).unwrap(),
                if choice == RepairChoice::Purchase
                    && before.continuity.route_services.stop.is_none()
                {
                    "roadside"
                } else {
                    match choice {
                        RepairChoice::Onboard => "log_0",
                        RepairChoice::Purchase => "log_1",
                        RepairChoice::Barter => "log_2",
                        RepairChoice::Radio => "log_3",
                    }
                },
                &i18n::tr(
                    "trail.repaired",
                    Some(&std::collections::BTreeMap::from([("part", part.as_str())])),
                ),
            ),
            scene: crate::components::ui::journey_scene::SceneStage::Breakdown,
            before: before.stats.clone(),
            after: session.state().stats.clone(),
            next: super::aftermath::next_phase(session.state()),
            resources: Vec::new(),
            details: super::receipt::resource_details(&before, session.state()),
        };
        session.with_state_mut(|gs| {
            record(&before, gs, choice);
            super::history::record(&before, gs, &mut report, choice.minutes());
        });
        super::history::publish(&app, report, true);
        app.session.set(Some(session));
    })
}

fn unit(gs: &crate::game::GameState) -> Option<String> {
    let b = gs.breakdown.as_ref()?;
    let family = match b.part {
        crate::game::vehicle::Part::Tire => "REPAIR-TIRE",
        crate::game::vehicle::Part::Battery => "REPAIR-BATTERY",
        crate::game::vehicle::Part::Alternator => "REPAIR-ALTERNATOR",
        crate::game::vehicle::Part::FuelPump => "REPAIR-FUELPUMP",
    };
    Some(super::visual_content::service_unit(
        gs,
        family,
        &format!("repair/{}", gs.continuity.driving_minutes_total),
        u32::try_from(b.day_started).unwrap_or_default(),
    ))
}
fn record(
    before: &crate::game::GameState,
    after: &mut crate::game::GameState,
    choice: RepairChoice,
) {
    let Some(unit) = unit(before) else { return };
    let family = unit.rsplit_once('-').unwrap().0;
    let key = format!(
        "{family}/repair/{}/{}",
        before.continuity.driving_minutes_total,
        before.breakdown.as_ref().unwrap().day_started.max(0)
    );
    after
        .continuity
        .visual_content
        .selections
        .insert(key.clone(), unit);
    after.continuity.visual_content.outcomes.insert(
        key,
        match choice {
            RepairChoice::Onboard => 0,
            RepairChoice::Purchase => 1,
            RepairChoice::Barter => 2,
            RepairChoice::Radio => 3,
        },
    );
}
