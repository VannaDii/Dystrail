//! Bounded supply gathering and town work, persisted independently from UI navigation.
use crate::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrailActivities {
    pub foraged_on: Option<u32>,
    pub worked_at: Option<u32>,
    pub local_word: Option<u8>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Forage,
    Glean,
    WorkSupplies,
    WorkCash,
}
impl Activity {
    pub const ROADSIDE: [Self; 2] = [Self::Forage, Self::Glean];
    pub const TOWN: [Self; 2] = [Self::WorkSupplies, Self::WorkCash];
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Forage => "trail.forage",
            Self::Glean => "trail.glean",
            Self::WorkSupplies => "trail.work_supplies",
            Self::WorkCash => "trail.work_cash",
        }
    }
    #[must_use]
    pub const fn minutes(self) -> u16 {
        match self {
            Self::Forage | Self::Glean => 120,
            Self::WorkSupplies | Self::WorkCash => 180,
        }
    }
}
impl GameState {
    #[must_use]
    pub fn can_activity(&self, action: Activity) -> bool {
        if self.current_encounter.is_some()
            || self.breakdown.is_some()
            || self.continuity.crew_care.pending.is_some()
            || self.ending.is_some()
            || self.continuity.abandoned
        {
            return false;
        }
        match action {
            Activity::Forage | Activity::Glean => {
                self.continuity.route_services.stop.is_none()
                    && self
                        .continuity
                        .activities
                        .foraged_on
                        .is_none_or(|day| self.day >= day.saturating_add(3))
                    && self.stats.supplies <= if action == Activity::Forage { 18 } else { 16 }
                    && (action != Activity::Glean || self.stats.hp > 1)
            }
            Activity::WorkSupplies | Activity::WorkCash => {
                self.continuity.route_services.stop.is_some()
                    && self.continuity.activities.worked_at != self.continuity.route_services.stop
                    && (action != Activity::WorkSupplies || self.stats.supplies <= 16)
                    && self.stats.sanity > 0
            }
        }
    }
    pub fn perform_activity(&mut self, action: Activity) -> bool {
        if !self.can_activity(action) {
            return false;
        }
        match action {
            Activity::Forage => {
                self.stats.supplies += 2;
                self.stats.sanity = (self.stats.sanity + 1).min(10);
            }
            Activity::Glean => {
                self.stats.supplies += 4;
                self.stats.hp = (self.stats.hp - 1).max(0);
            }
            Activity::WorkSupplies => {
                self.stats.supplies += 4;
                self.stats.sanity = (self.stats.sanity - 1).max(0);
            }
            Activity::WorkCash => {
                self.budget_cents += 1800;
                self.budget = i32::try_from(self.budget_cents / 100).unwrap_or(0);
                self.stats.sanity = (self.stats.sanity - 1).max(0);
            }
        }
        if matches!(action, Activity::Forage | Activity::Glean) {
            self.continuity.activities.foraged_on = Some(self.day);
        } else {
            self.continuity.activities.worked_at = self.continuity.route_services.stop;
        }
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gathering_and_paid_work_cannot_be_farmed_by_reloading() {
        let mut gs = GameState::default();
        gs.stats.supplies = 5;
        assert!(gs.perform_activity(Activity::Glean));
        assert_eq!(gs.stats.supplies, 9);
        let mut gs: GameState = serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        assert!(!gs.perform_activity(Activity::Forage));
        gs.day += 3;
        assert!(gs.perform_activity(Activity::Forage));
        gs.continuity.route_services.stop = Some(160);
        let cash = gs.budget_cents;
        assert!(gs.perform_activity(Activity::WorkCash));
        assert_eq!(gs.budget_cents, cash + 1800);
        assert!(!gs.perform_activity(Activity::WorkSupplies));
        assert!(!gs.perform_activity(Activity::Forage));
    }
}
