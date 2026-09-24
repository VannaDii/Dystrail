//! The outcome and Trail report share the persisted journal's actual changes.
use super::{aftermath::Aftermath, state::AppState};
use crate::game::{GameState, journal::JournalEntry};

pub fn record(before: &GameState, after: &mut GameState, report: &mut Aftermath, minutes: u16) {
    after
        .continuity
        .turn_journal_start
        .get_or_insert(before.continuity.journal.len());
    after.advance_clock(before, minutes);
    super::policy_bulletin::capture(before, after);
    report.after = after.stats.clone();
    if after.breakdown.is_some() {
        report.scene = crate::components::ui::journey_scene::SceneStage::Breakdown;
    }
    let ally_lost = report.after.allies < report.before.allies;
    if ally_lost {
        super::ally_loss::explain(before, after, report);
        report.next = super::Phase::AllyLoss;
    }
    report.resources = super::receipt::resource_changes(before, after);
    let place = crate::components::ui::route_map::location::location(after);
    let (day, minute) = entry_time(before, after);
    let entry = JournalEntry {
        action_kind: if after.breakdown.is_some() {
            "repair"
        } else {
            crate::components::ui::journey_icon::action_kind(&report.scene)
        }
        .into(),
        day,
        minute,
        pace: Some(before.pace),
        diet: Some(before.diet),
        place,
        title: report.title.clone(),
        message: report.message.clone(),
        before: report.before.clone(),
        after: report.after.clone(),
        resources: report.resources.clone(),
        details: report.details.clone(),
    };
    if ally_lost {
        after.continuity.ally_notice = Some(entry.clone());
    }
    after.continuity.journal.push(entry);
}

/// The last drive belongs to the day it completed, before the overnight reset.
/// Work completed after a rollover retains its actual completion day and time.
const fn entry_time(before: &GameState, after: &GameState) -> (u32, u16) {
    use crate::game::travel_time::{TRAVEL_DAY_END, TRAVEL_DAY_START};
    if after.day == before.day.saturating_add(1)
        && before.continuity.clock_minutes < TRAVEL_DAY_END
        && after.continuity.clock_minutes == TRAVEL_DAY_START
        && after.continuity.driving_minutes_total > before.continuity.driving_minutes_total
    {
        (before.day, TRAVEL_DAY_END)
    } else {
        (after.day, after.continuity.clock_minutes)
    }
}

pub fn publish(state: &AppState, report: Aftermath, show_outcome: bool) {
    if report.next == super::Phase::AllyLoss {
        state.travel_running.set(false);
        state.phase.set(super::Phase::AllyLoss);
        state.aftermath.set(None);
    } else if show_outcome {
        state.aftermath.set(Some(report));
    }
}

#[must_use]
pub fn entry_changes(entry: &JournalEntry) -> Vec<(&'static str, i32, bool)> {
    let report = Aftermath {
        title: String::new(),
        message: String::new(),
        before: entry.before.clone(),
        after: entry.after.clone(),
        resources: Vec::new(),
        details: vec![],
        scene: crate::components::ui::journey_scene::SceneStage::Setup,
        next: super::Phase::Travel,
    };
    report.changes()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_driving_stays_in_its_day_while_next_morning_work_keeps_its_time() {
        use crate::game::travel_time::{TRAVEL_DAY_END, TRAVEL_DAY_START};
        let mut before = GameState::default();
        before.continuity.clock_minutes = TRAVEL_DAY_END - 60;
        let mut after = before.clone();
        after.day += 1;
        after.continuity.clock_minutes = TRAVEL_DAY_START;
        after.continuity.driving_minutes_total += 60;
        assert_eq!(entry_time(&before, &after), (before.day, TRAVEL_DAY_END));

        after.continuity.driving_minutes_total = before.continuity.driving_minutes_total;
        after.continuity.clock_minutes = TRAVEL_DAY_START + 60;
        assert_eq!(
            entry_time(&before, &after),
            (after.day, TRAVEL_DAY_START + 60)
        );

        before.continuity.clock_minutes = TRAVEL_DAY_END;
        after.continuity.driving_minutes_total += 60;
        assert_eq!(
            entry_time(&before, &after),
            (after.day, TRAVEL_DAY_START + 60)
        );
    }

    #[test]
    fn an_ally_loss_is_recorded_once_and_waits_for_acknowledgment_after_reload() {
        let mut before = GameState::default();
        before.stats.allies = 2;
        before.pace = crate::game::PaceId::Heated;
        before.diet = crate::game::DietId::Quiet;
        let mut after = before.clone();
        after.stats.allies = 1;
        after.logs.push("log.ally.lost".into());
        let mut report = Aftermath {
            title: String::new(),
            message: String::new(),
            before: before.stats.clone(),
            after: after.stats.clone(),
            scene: crate::components::ui::journey_scene::SceneStage::Travel(after.region),
            next: super::super::Phase::Travel,
            resources: Vec::new(),
            details: Vec::new(),
        };
        record(&before, &mut after, &mut report, 120);
        after.pace = crate::game::PaceId::Blitz;
        after.diet = crate::game::DietId::Doom;
        let mut loaded: GameState =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert!(report.next == super::super::Phase::AllyLoss);
        assert!(super::super::aftermath::next_phase(&loaded) == super::super::Phase::AllyLoss);
        assert_eq!(loaded.continuity.journal.len(), 1);
        assert_eq!(loaded.continuity.journal[0].pace, Some(before.pace));
        assert_eq!(loaded.continuity.journal[0].diet, Some(before.diet));
        assert_eq!(
            loaded.continuity.ally_notice.as_ref(),
            loaded.continuity.journal.last()
        );
        assert!(
            loaded
                .continuity
                .ally_notice
                .as_ref()
                .is_some_and(|n| !n.message.is_empty()
                    && n.before.allies == 2
                    && n.after.allies == 1)
        );
        loaded.continuity.ally_notice = None;
        assert!(super::super::aftermath::next_phase(&loaded) == super::super::Phase::Travel);
        assert_eq!(loaded.stats, after.stats);
        assert_eq!(loaded.continuity.journal, after.continuity.journal);
    }
}
