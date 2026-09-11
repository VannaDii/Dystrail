//! Named fatigue and aid decisions. Presentation never invents or revives a traveler.
use crate::{GameState, party::MemberStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewCare {
    #[serde(default)]
    pub reason: u8,
    pub strain: BTreeMap<String, u8>,
    pub pending: Option<String>,
    pub last_check_day: u32,
}

impl GameState {
    /// Periodic checks use a stable rotation without consuming the simulation RNG.
    pub fn check_crew(&mut self, previous_day: u32) {
        if self.day <= previous_day
            || self.day < self.continuity.crew_care.last_check_day.saturating_add(5)
            || self.continuity.crew_care.pending.is_some()
            || self.ending.is_some()
            || self.boss.readiness.ready
        {
            return;
        }
        let active: Vec<_> = self
            .party
            .members
            .iter()
            .filter(|m| m.status == MemberStatus::Active)
            .collect();
        if active.is_empty() {
            return;
        }
        let struggling = active.iter().find(|m| {
            self.continuity
                .crew_care
                .strain
                .get(&m.persona)
                .copied()
                .unwrap_or(0)
                > 0
        });
        let offset = usize::try_from(
            u64::from(self.day / 5).wrapping_add(self.seed)
                % u64::try_from(active.len()).unwrap_or(1),
        )
        .unwrap_or(0);
        let member = struggling.copied().unwrap_or(active[offset]);
        let persona = member.persona.clone();
        let strain = self
            .continuity
            .crew_care
            .strain
            .entry(persona.clone())
            .or_default();
        *strain = strain.saturating_add(1).min(3);
        self.continuity.crew_care.pending = Some(persona);
        if *strain == 1 {
            self.continuity.crew_care.reason =
                u8::try_from((self.seed.wrapping_add(u64::from(self.day / 5))) % 8).unwrap_or(0);
        }
        self.continuity.crew_care.last_check_day = self.day;
    }

    /// Care costs two supplies; shelter leaves the expedition; pressing on carries strain.
    /// At critical strain, pressing on without care causes a clearly forecast fatal loss.
    pub fn resolve_crew_care(&mut self, choice: usize) -> Option<&'static str> {
        let persona = self.continuity.crew_care.pending.clone()?;
        if !self
            .party
            .members
            .iter()
            .any(|m| m.persona == persona && m.status == MemberStatus::Active)
        {
            return None;
        }
        let strain = self
            .continuity
            .crew_care
            .strain
            .get(&persona)
            .copied()
            .unwrap_or(1);
        let key = match choice {
            0 if self.stats.supplies >= 2 => {
                self.stats.supplies -= 2;
                self.stats.morale = (self.stats.morale + 1).min(10);
                self.continuity.crew_care.strain.remove(&persona);
                "journey.care_helped"
            }
            1 if self.persona_id.as_deref() != Some(persona.as_str()) => {
                self.party.set_status(&persona, MemberStatus::Departed);
                self.continuity.crew_care.strain.remove(&persona);
                self.stats.morale = (self.stats.morale - 1).max(0);
                "journey.care_sheltered"
            }
            2 => {
                self.stats.sanity = (self.stats.sanity - 1).max(0);
                if strain >= 3 {
                    self.party.set_status(&persona, MemberStatus::Dead);
                    if self.persona_id.as_deref() == Some(persona.as_str()) {
                        self.ending = Some(crate::Ending::Collapse {
                            cause: crate::CollapseCause::Disease,
                        });
                    }
                    self.stats.morale = (self.stats.morale - 3).max(0);
                    "journey.care_lost"
                } else {
                    "journey.care_deferred"
                }
            }
            _ => return None,
        };
        self.continuity.crew_care.pending = None;
        Some(key)
    }

    /// A stationary recovery day helps everyone still present, not just a global meter.
    pub fn recover_crew(&mut self) {
        self.continuity.crew_care.strain.clear();
        self.continuity.crew_care.pending = None;
        self.continuity.crew_care.last_check_day = self.day;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn natural_check_care_deferral_and_absence_are_persistent() {
        let mut gs = GameState {
            persona_id: Some("journalist".into()),
            ..GameState::default()
        };
        gs.party.initialize("journalist", 7);
        gs.day = 5;
        gs.check_crew(4);
        let actor = gs.continuity.crew_care.pending.clone().unwrap();
        assert!(gs.resolve_crew_care(2).is_some());
        gs.day = 10;
        gs.check_crew(9);
        assert_eq!(
            gs.continuity.crew_care.pending.as_deref(),
            Some(actor.as_str())
        );
        let json = serde_json::to_string(&gs).unwrap();
        let mut restored: GameState = serde_json::from_str(&json).unwrap();
        let before = restored.stats.supplies;
        assert_eq!(restored.resolve_crew_care(0), Some("journey.care_helped"));
        assert_eq!(restored.stats.supplies, before - 2);
        assert!(restored.continuity.crew_care.strain.is_empty());
        restored.continuity.crew_care.pending = Some("organizer".into());
        assert!(restored.resolve_crew_care(1).is_some());
        assert!(
            !restored
                .party
                .occupants()
                .iter()
                .any(|(_, m)| m.persona == "organizer")
        );
        restored.continuity.crew_care.pending = Some("satirist".into());
        restored
            .continuity
            .crew_care
            .strain
            .insert("satirist".into(), 3);
        assert_eq!(restored.resolve_crew_care(2), Some("journey.care_lost"));
        assert!(
            !restored
                .party
                .occupants()
                .iter()
                .any(|(_, m)| m.persona == "satirist")
        );
    }
}
