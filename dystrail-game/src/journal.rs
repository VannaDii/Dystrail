//! Persisted presentation history. Entries contain actual outcomes, never predictions.
use crate::journey::{DayRecord, TravelDayKind};
use crate::{GameState, Stats};
use serde::{Deserialize, Serialize};

/// Presentation edition and sealed story choices; never used by simulation RNG.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualContent {
    #[serde(default)]
    pub edition: u16,
    #[serde(default)]
    pub selections: std::collections::BTreeMap<String, String>,
}

/// The saved player-facing account of the journey, including its route checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Continuity {
    #[serde(default)]
    pub visual_content: VisualContent,
    #[serde(default)]
    pub weather_impact: Option<crate::weather_impact::WeatherImpact>,
    #[serde(default)]
    pub interactive_repairs: bool,
    #[serde(default)]
    pub activities: crate::activities::TrailActivities,
    #[serde(default)]
    pub abandoned: bool,
    #[serde(default = "morning")]
    pub clock_minutes: u16,
    /// Actual time spent moving, excluding work, encounters and overnight rollover.
    #[serde(default)]
    pub driving_minutes_total: u32,
    /// Uncharged direct pace fatigue in 1/300-sanity units; retained across days and saves.
    #[serde(default)]
    pub pace_fatigue_remainder: u16,
    /// Driving exposure when the latest road encounter began; parked time is excluded.
    #[serde(default)]
    pub last_encounter_driving_minutes: Option<u32>,
    #[serde(default)]
    pub journal: Vec<JournalEntry>,
    /// First journal entry of the latest travel step and its following player decisions.
    #[serde(default)]
    pub turn_journal_start: Option<usize>,
    /// A loss that the player must acknowledge before continuing the journey.
    #[serde(default)]
    pub ally_notice: Option<JournalEntry>,
    #[serde(default)]
    pub crew_care: crate::crew_care::CrewCare,
    #[serde(default)]
    pub scene_subject: Option<String>,
    #[serde(default)]
    pub route_services: crate::route_services::RouteServices,
}
impl Default for Continuity {
    fn default() -> Self {
        Self {
            visual_content: VisualContent::default(),
            weather_impact: None,
            interactive_repairs: false,
            activities: crate::activities::TrailActivities::default(),
            clock_minutes: morning(),
            driving_minutes_total: 0,
            pace_fatigue_remainder: 0,
            last_encounter_driving_minutes: None,
            abandoned: false,
            journal: Vec::new(),
            turn_journal_start: None,
            ally_notice: None,
            crew_care: crate::crew_care::CrewCare::default(),
            scene_subject: None,
            route_services: crate::route_services::RouteServices::default(),
        }
    }
}

impl Continuity {
    /// Start a fresh Trail report without removing anything from the full journal.
    pub const fn begin_turn(&mut self) {
        self.turn_journal_start = Some(self.journal.len());
    }

    /// Chronological entries from the current turn. Unstarted histories show their latest entry.
    #[must_use]
    pub fn turn_entries(&self) -> &[JournalEntry] {
        let start = self
            .turn_journal_start
            .unwrap_or_else(|| self.journal.len().saturating_sub(1));
        self.journal.get(start..).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    #[serde(default)]
    pub action_kind: String,
    pub day: u32,
    pub minute: u16,
    /// Settings used for this action, independent of later player changes.
    #[serde(default)]
    pub pace: Option<crate::PaceId>,
    #[serde(default)]
    pub diet: Option<crate::DietId>,
    pub place: String,
    pub title: String,
    pub message: String,
    pub before: Stats,
    pub after: Stats,
    pub details: Vec<(String, String)>,
    #[serde(default)]
    pub resources: Vec<ResourceChange>,
}

/// Resource values use cents, hundredths of vehicle condition, tenths of miles, days or item counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceChange {
    pub key: String,
    pub before: i64,
    pub after: i64,
}

#[must_use]
pub const fn morning() -> u16 {
    8 * 60
}

impl GameState {
    /// Actions share the driving window; unfinished work resumes next morning.
    /// Opening a screen never spends time, and overnight rest never erases work hours.
    pub fn advance_clock(&mut self, before: &Self, minutes: u16) {
        if minutes == 0 {
            return;
        }
        if self.day > before.day {
            self.continuity.clock_minutes = morning();
        } else {
            let mut remaining = minutes;
            self.continuity.clock_minutes = before
                .continuity
                .clock_minutes
                .max(crate::travel_time::TRAVEL_DAY_START);
            while remaining > 0 {
                if self.continuity.clock_minutes >= crate::travel_time::TRAVEL_DAY_END {
                    self.start_of_day();
                    self.day_state.lifecycle.suppress_stop_ratio = true;
                    self.end_of_day();
                    self.continuity.clock_minutes = morning();
                }
                let spent = remaining.min(self.travel_minutes_available());
                if spent > 0 && self.ledger.current_day_record.is_none() {
                    // Elapsed work is recorded even while daily resource costs stay lazy.
                    let day_index = u16::try_from(self.day.saturating_sub(1)).unwrap_or(u16::MAX);
                    self.ledger.current_day_record =
                        Some(DayRecord::new(day_index, TravelDayKind::NonTravel, 0.0));
                }
                self.continuity.clock_minutes += spent;
                remaining -= spent;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(message: &str) -> JournalEntry {
        JournalEntry {
            action_kind: "travel".into(),
            day: 1,
            minute: morning(),
            pace: Some(crate::PaceId::Steady),
            diet: Some(crate::DietId::Mixed),
            place: "Seattle".into(),
            title: message.into(),
            message: message.into(),
            before: Stats::default(),
            after: Stats::default(),
            details: Vec::new(),
            resources: Vec::new(),
        }
    }

    #[test]
    fn turn_history_preserves_every_action_and_resets_only_when_travel_begins() {
        let mut history = Continuity::default();
        history.journal.push(entry("Earlier encounter"));
        history.begin_turn();
        history.journal.push(entry("Reached a stop"));
        history.journal.push(entry("Talked to locals"));
        let mut purchase = entry("Bought supplies after midnight");
        purchase.day = 2;
        history.journal.push(purchase);
        let mut restored: Continuity =
            serde_json::from_str(&serde_json::to_string(&history).unwrap()).unwrap();
        assert_eq!(restored.turn_entries(), &history.journal[1..]);
        assert_eq!(
            restored
                .turn_entries()
                .iter()
                .rev()
                .map(|entry| entry.message.as_str())
                .collect::<Vec<_>>(),
            [
                "Bought supplies after midnight",
                "Talked to locals",
                "Reached a stop"
            ]
        );
        restored.begin_turn();
        assert!(restored.turn_entries().is_empty());
        restored.journal.push(entry("Next travel step"));
        assert_eq!(restored.turn_entries(), &restored.journal[4..]);
        assert_eq!(&restored.journal[..4], history.journal);
    }

    #[test]
    fn an_empty_or_out_of_range_turn_has_no_entries() {
        let mut history = Continuity::default();
        assert!(history.turn_entries().is_empty());
        history.journal.push(entry("Latest action"));
        assert_eq!(history.turn_entries(), history.journal);
        history.turn_journal_start = Some(usize::MAX);
        assert!(history.turn_entries().is_empty());
    }

    #[test]
    fn old_saves_ignore_the_retired_stat_without_changing_other_resources() {
        let gs = GameState::default();
        let mut legacy = serde_json::to_value(&gs).unwrap();
        legacy["stats"]["pants"] = serde_json::json!(100);
        legacy.as_object_mut().unwrap().remove("ally_notice");
        let loaded: GameState = serde_json::from_value(legacy).unwrap();
        assert_eq!(loaded.stats, gs.stats);
        assert_eq!(loaded.budget_cents, gs.budget_cents);
        assert!(loaded.ending.is_none());
        assert!(loaded.continuity.ally_notice.is_none());
        assert!(
            serde_json::to_value(&loaded).unwrap()["stats"]
                .get("pants")
                .is_none()
        );
    }
    #[test]
    fn an_after_hours_repair_uses_all_four_hours_of_the_next_active_day() {
        let mut before = GameState::default();
        before.continuity.clock_minutes = 23 * 60;
        let mut after = before.clone();
        after.advance_clock(&before, 240);
        assert_eq!(after.day, before.day + 1);
        assert_eq!(after.continuity.clock_minutes, morning() + 4 * 60);
    }
    #[test]
    fn clock_and_actual_history_survive_save_without_navigation_time() {
        let before = GameState::default();
        let mut after = before.clone();
        after.advance_clock(&before, 90);
        assert_eq!(after.continuity.clock_minutes, 570);
        after.continuity.journal.push(JournalEntry {
            action_kind: "camp".into(),
            day: after.day,
            minute: after.continuity.clock_minutes,
            pace: Some(before.pace),
            diet: Some(before.diet),
            place: "Omaha".into(),
            title: "Rest".into(),
            message: "Recovered".into(),
            before: before.stats,
            after: after.stats.clone(),
            resources: Vec::new(),
            details: vec![],
        });
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert_eq!(restored.continuity.journal, after.continuity.journal);
        assert_eq!(
            restored.continuity.clock_minutes,
            after.continuity.clock_minutes
        );
        let mut next = restored.clone();
        next.day += 1;
        next.advance_clock(&restored, 30);
        assert!(
            next.day * 1440 + u32::from(next.continuity.clock_minutes)
                > restored.day * 1440 + u32::from(restored.continuity.clock_minutes)
        );
    }
}
