//! Ally departures are acknowledged once and remain in the saved trail journal.
use super::{Phase, aftermath::Aftermath, state::AppState};
use crate::{game::GameState, i18n};
use std::collections::BTreeMap;
use yew::prelude::*;

pub fn explain(before: &GameState, after: &GameState, report: &mut Aftermath) {
    let unexpected = after
        .logs
        .iter()
        .skip(before.logs.len())
        .any(|line| line == "log.ally.lost");
    if !unexpected {
        return;
    }
    let names = [
        "Ari", "Blair", "Casey", "Devon", "Ellis", "Finley", "Jules", "Kit",
    ];
    let available: Vec<_> = names
        .into_iter()
        .filter(|name| {
            !after
                .party
                .members
                .iter()
                .any(|member| member.name.eq_ignore_ascii_case(name))
        })
        .collect();
    let index = usize::try_from(after.day).unwrap_or(0);
    let contact = available
        .get(index % available.len().max(1))
        .copied()
        .unwrap_or("Ari");
    report.title = i18n::t("ally_loss.title");
    report.message = i18n::tr(
        &format!("ally_loss.reason_{}", after.day % 6),
        Some(&BTreeMap::from([("name", contact)])),
    );
}

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let Some(entry) = &gs.continuity.ally_notice else {
        return Html::default();
    };
    let mut resolved = gs.clone();
    resolved.continuity.ally_notice = None;
    let report = Aftermath {
        title: i18n::t("ally_loss.title"),
        message: entry.message.clone(),
        scene: crate::components::ui::journey_scene::SceneStage::Travel(gs.region),
        before: entry.before.clone(),
        after: entry.after.clone(),
        resources: entry.resources.clone(),
        details: entry.details.clone(),
        next: super::aftermath::next_phase(&resolved),
    };
    super::view::phases::aftermath::render_aftermath(app, &report)
}

#[must_use]
pub fn is_notice(app: &AppState) -> bool {
    *app.phase == Phase::AllyLoss
}
