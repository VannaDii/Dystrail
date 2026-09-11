//! One receipt is shared by the outcome, latest action and persisted journal.
use super::{aftermath::Aftermath, state::AppState};
use crate::{
    game::{GameState, journal::JournalEntry},
    i18n,
};

pub fn record(before: &GameState, after: &mut GameState, report: &mut Aftermath, minutes: u16) {
    after.advance_clock(before, minutes);
    let place = crate::components::ui::route_map::location::location(after);
    after.continuity.journal.push(JournalEntry {
        day: after.day,
        minute: after.continuity.clock_minutes,
        place,
        title: report.title.clone(),
        message: report.message.clone(),
        before: report.before.clone(),
        after: report.after.clone(),
        details: report.details.clone(),
    });
}

pub fn publish(state: &AppState, report: Aftermath, show_outcome: bool) {
    state.last_turn.set(Some(report.clone()));
    if show_outcome {
        state.aftermath.set(Some(report));
    }
}

#[must_use]
pub fn latest(gs: &GameState) -> Option<Aftermath> {
    gs.continuity.journal.last().map(|entry| Aftermath {
        title: entry.title.clone(),
        message: entry.message.clone(),
        before: entry.before.clone(),
        after: entry.after.clone(),
        details: entry.details.clone(),
        scene: crate::components::ui::journey_scene::SceneStage::Travel(gs.region),
        next: super::aftermath::next_phase(gs),
    })
}

#[must_use]
pub fn entry_deltas(entry: &JournalEntry) -> String {
    let report = Aftermath {
        title: String::new(),
        message: String::new(),
        before: entry.before.clone(),
        after: entry.after.clone(),
        details: vec![],
        scene: crate::components::ui::journey_scene::SceneStage::Setup,
        next: super::Phase::Travel,
    };
    report
        .changes()
        .iter()
        .map(|(key, delta, _)| format!("{} {delta:+}", i18n::t(key)))
        .collect::<Vec<_>>()
        .join(" · ")
}
