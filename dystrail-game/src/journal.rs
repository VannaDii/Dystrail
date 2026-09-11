//! Persisted presentation history. Entries contain actual outcomes, never predictions.
use crate::{GameState, Stats};
use serde::{Deserialize, Serialize};

/// The saved player-facing account of the journey, including its route checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Continuity {
    #[serde(default)]
    pub interactive_repairs: bool,
    #[serde(default)]
    pub activities: crate::activities::TrailActivities,
    #[serde(default)]
    pub abandoned: bool,
    #[serde(default = "morning")]
    pub clock_minutes: u16,
    #[serde(default)]
    pub journal: Vec<JournalEntry>,
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
            interactive_repairs: false,
            activities: crate::activities::TrailActivities::default(),
            clock_minutes: morning(),
            abandoned: false,
            journal: Vec::new(),
            crew_care: crate::crew_care::CrewCare::default(),
            scene_subject: None,
            route_services: crate::route_services::RouteServices::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub day: u32,
    pub minute: u16,
    pub place: String,
    pub title: String,
    pub message: String,
    pub before: Stats,
    pub after: Stats,
    pub details: Vec<(String, String)>,
}

#[must_use]
pub const fn morning() -> u16 {
    8 * 60
}

impl GameState {
    /// Only completed actions advance the clock. Opening a screen never spends time.
    pub fn advance_clock(&mut self, before: &Self, minutes: u16) {
        if self.day > before.day {
            self.continuity.clock_minutes = morning();
        } else {
            let total = u32::from(before.continuity.clock_minutes) + u32::from(minutes);
            self.day = self.day.saturating_add(total / 1440);
            self.continuity.clock_minutes = u16::try_from(total % 1440).unwrap_or(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_late_repair_spends_all_four_hours_across_midnight() {
        let mut before = GameState::default();
        before.continuity.clock_minutes = 23 * 60;
        let mut after = before.clone();
        after.advance_clock(&before, 240);
        assert_eq!(after.day, before.day + 1);
        assert_eq!(after.continuity.clock_minutes, 3 * 60);
    }
    #[test]
    fn clock_and_actual_history_survive_save_without_navigation_time() {
        let before = GameState::default();
        let mut after = before.clone();
        after.advance_clock(&before, 90);
        assert_eq!(after.continuity.clock_minutes, 570);
        after.continuity.journal.push(JournalEntry {
            day: after.day,
            minute: after.continuity.clock_minutes,
            place: "Omaha".into(),
            title: "Rest".into(),
            message: "Recovered".into(),
            before: before.stats,
            after: after.stats.clone(),
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
