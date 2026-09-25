//! Ally departures are acknowledged once and remain in the saved trail journal.
use super::{Phase, aftermath::Aftermath, state::AppState};
use crate::{game::GameState, i18n};
use std::collections::BTreeMap;
use yew::prelude::*;

pub fn explain(before: &GameState, after: &mut GameState, report: &mut Aftermath) {
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
    if after.continuity.visual_content.edition == super::visual_content::EDITION {
        let family = format!("ALLY-{:02}", after.day % 6 + 1);
        let unit = super::visual_content::service_unit(after, &family, "departure", after.day);
        after.continuity.visual_content.selections.entry(format!("{family}/departure/{}", after.day)).or_insert(unit.clone());
        report.title = i18n::t(&format!("encounter_copy.{unit}.name"));
        report.message = i18n::tr(&format!("encounter_copy.{unit}.desc"), Some(&BTreeMap::from([("name", contact)])));
        if after.stats.allies == 0 {
            report.message.push(' ');
            report.message.push_str(&i18n::t(&format!("encounter_copy.{unit}.last")));
        }
        return;
    }
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
        title: entry.title.clone(),
        message: entry.message.clone(),
        scene: notice_scene(gs),
        before: entry.before.clone(),
        after: entry.after.clone(),
        resources: entry.resources.clone(),
        details: entry.details.clone(),
        next: super::aftermath::next_phase(&resolved),
    };
    super::view::phases::aftermath::render_aftermath(app, &report)
}

fn notice_scene(gs: &GameState) -> crate::components::ui::journey_scene::SceneStage {
    use crate::components::ui::journey_scene::{SceneStage, ally_art};
    let key = format!("ALLY-{:02}/departure/{}", gs.day % 6 + 1, gs.day);
    gs.continuity.visual_content.selections.get(&key)
        .filter(|unit| ally_art::coordinates(unit).is_some())
        .map_or(SceneStage::Travel(gs.region), |unit| SceneStage::Encounter(unit.clone()))
}

#[must_use]
pub fn is_notice(app: &AppState) -> bool {
    *app.phase == Phase::AllyLoss
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_ally_variants_preserve_crew_and_actual_losses() {
        for day in 1..=6 { for suffix in ["A", "B", "C"] { for remaining in [0, 1] {
            let mut before = GameState::default();
            before.day = day;
            before.stats.allies = remaining + 1;
            before.party.initialize("journalist", 42);
            before.continuity.visual_content.edition = super::super::visual_content::EDITION;
            let family = format!("ALLY-{:02}", day % 6 + 1);
            let unit = format!("{family}-{suffix}");
            before.continuity.visual_content.selections.insert(format!("{family}/departure/{day}"), unit.clone());
            let mut after = before.clone();
            after.stats.allies = remaining;
            after.logs.push("log.ally.lost".into());
            let mut report = Aftermath { title: "original".into(), message: "original".into(), scene: crate::components::ui::journey_scene::SceneStage::Travel(after.region), before: before.stats.clone(), after: after.stats.clone(), next: Phase::Travel, details: vec![], resources: vec![] };
            let snapshot = serde_json::to_value(&after).unwrap();
            explain(&before, &mut after, &mut report);
            assert_eq!(report.title, i18n::t(&format!("encounter_copy.{unit}.name")));
            assert!(!report.message.contains("{name}"));
            let last = i18n::t(&format!("encounter_copy.{unit}.last"));
            assert_eq!(report.message.ends_with(&last), remaining == 0);
            assert_eq!(serde_json::to_value(&after).unwrap(), snapshot);
            for member in &after.party.members { assert!(!report.message.contains(&format!("contact {} ", member.name))); }
            after.logs.pop();
            report.message = "deliberate choice".into();
            explain(&before, &mut after, &mut report);
            assert_eq!(report.message, "deliberate choice");
        }}}
    }
}
