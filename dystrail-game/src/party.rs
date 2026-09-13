//! Named crew identities and presence, independent of presentation and simulation RNG.
use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

pub const PERSONAS: [&str; 6] = [
    "staffer",
    "satirist",
    "whistleblower",
    "lobbyist",
    "organizer",
    "journalist",
];
const NAMES: [&str; 24] = [
    "Alex", "Avery", "Blair", "Casey", "Charlie", "Dakota", "Drew", "Emerson", "Finley", "Harper",
    "Jamie", "Jordan", "Kai", "Morgan", "Quinn", "Reese", "Riley", "River", "Robin", "Rowan",
    "Sage", "Sam", "Skyler", "Taylor",
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MemberStatus {
    #[default]
    Active,
    Dead,
    Departed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    pub persona: String,
    pub name: String,
    #[serde(default)]
    pub status: MemberStatus,
}
/// Legacy name fields remain readable by older saves and result exporters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Party {
    #[serde(default)]
    pub leader: String,
    #[serde(default)]
    pub companions: Vec<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub members: Vec<CrewMember>,
}
impl Party {
    /// Member identity is authoritative; legacy saves may only have the leader field.
    #[must_use]
    pub fn player_name(&self, player: Option<&str>) -> &str {
        self.members
            .iter()
            .find(|m| Some(m.persona.as_str()) == player && !m.name.trim().is_empty())
            .map_or(self.leader.as_str(), |m| m.name.as_str())
    }
    /// Initialize once. Existing identities, names and absences always win on restore.
    pub fn initialize(&mut self, player: &str, naming_seed: u64) {
        if !self.members.is_empty() {
            return;
        }
        let mut names = NAMES;
        names.shuffle(&mut SmallRng::seed_from_u64(naming_seed));
        let mut old_names = self.companions.iter();
        self.members = PERSONAS
            .iter()
            .enumerate()
            .map(|(i, persona)| CrewMember {
                persona: (*persona).to_owned(),
                name: if *persona == player {
                    self.leader.clone()
                } else {
                    old_names
                        .next()
                        .cloned()
                        .unwrap_or_else(|| names[i].to_owned())
                },
                status: MemberStatus::Active,
            })
            .collect();
    }
    pub fn sync_names(&mut self, player: &str) {
        self.leader = self
            .members
            .iter()
            .find(|m| m.persona == player)
            .map_or_else(String::new, |m| m.name.clone());
        self.companions = self
            .members
            .iter()
            .filter(|m| m.persona != player)
            .map(|m| m.name.clone())
            .collect();
    }
    /// Apply an explicit story outcome. Naming and rehydration cannot revive a member.
    pub fn set_status(&mut self, persona: &str, status: MemberStatus) -> bool {
        if let Some(member) = self.members.iter_mut().find(|m| m.persona == persona) {
            member.status = status;
            true
        } else {
            false
        }
    }
    #[must_use]
    pub fn occupants(&self) -> Vec<(usize, &CrewMember)> {
        let mut seats: Vec<_> = self
            .members
            .iter()
            .enumerate()
            .filter(|(_, m)| m.status == MemberStatus::Active)
            .collect();
        // An available crew member takes the wheel if the usual driver is absent.
        if !seats.iter().any(|(seat, _)| *seat == 4)
            && let Some((seat, _)) = seats.first_mut()
        {
            *seat = 4;
        }
        seats
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_are_unique_reproducible_and_do_not_repopulate_absent_members() {
        let mut party = Party::default();
        party.initialize("journalist", 7);
        assert_eq!(party.members.len(), 6);
        assert_eq!(party.members[5].name, "");
        let names: std::collections::BTreeSet<_> = party.members.iter().map(|m| &m.name).collect();
        assert_eq!(names.len(), 6);
        party.members[5].name = "Any Full Name".into();
        party.sync_names("journalist");
        assert_eq!(party.leader, "Any Full Name");
        assert!(party.set_status("organizer", MemberStatus::Departed));
        assert!(party.set_status("satirist", MemberStatus::Dead));
        let saved = serde_json::to_string(&party).unwrap();
        let mut loaded: Party = serde_json::from_str(&saved).unwrap();
        loaded.initialize("journalist", 99);
        assert_eq!(loaded, party);
        assert_eq!(loaded.occupants().len(), 4);
        assert!(loaded.occupants().iter().any(|(seat, _)| *seat == 4));
        assert!(
            !loaded
                .occupants()
                .iter()
                .any(|(_, m)| matches!(m.persona.as_str(), "satirist" | "organizer"))
        );
        assert!(!loaded.set_status("unknown", MemberStatus::Dead));
    }
    #[test]
    fn legacy_names_survive_upgrade() {
        let mut party: Party =
            serde_json::from_str(r#"{"leader":"Vanna","companions":["Robin","Kai"]}"#).unwrap();
        party.initialize("journalist", 1);
        party.sync_names("journalist");
        assert_eq!(party.leader, "Vanna");
        assert_eq!(party.members[0].name, "Robin");
        assert_eq!(party.members[1].name, "Kai");
    }
}
