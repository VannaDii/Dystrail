//! Ephemeral presentation state; never serialized into a saved run.
use crate::app::phase::Phase;
use crate::components::ui::journey_scene::SceneStage;
use crate::game::state::{GameState, Stats};

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Aftermath {
    pub title: String,
    pub message: String,
    pub scene: SceneStage,
    pub before: Stats,
    pub after: Stats,
    pub next: Phase,
    pub details: Vec<(String, String)>,
}

impl Aftermath {
    #[must_use]
    pub fn changes(&self) -> Vec<(&'static str, i32, bool)> {
        [
            (
                "ux.supplies",
                self.after.supplies - self.before.supplies,
                false,
            ),
            ("ux.health", self.after.hp - self.before.hp, false),
            ("ux.sanity", self.after.sanity - self.before.sanity, false),
            (
                "play.credibility",
                self.after.credibility - self.before.credibility,
                false,
            ),
            ("play.morale", self.after.morale - self.before.morale, false),
            ("play.allies", self.after.allies - self.before.allies, false),
            ("ux.pants", self.after.pants - self.before.pants, true),
        ]
        .into_iter()
        .filter(|(_, delta, _)| *delta != 0)
        .map(|(key, delta, inverted)| (key, delta, if inverted { delta > 0 } else { delta < 0 }))
        .collect()
    }
}

#[must_use]
pub const fn next_phase(gs: &GameState) -> Phase {
    if gs.continuity.abandoned || gs.ending.is_some() || gs.boss.outcome.attempted {
        Phase::Result
    } else if gs.continuity.crew_care.pending.is_some() {
        Phase::CrewCare
    } else if gs.current_encounter.is_some() {
        Phase::Encounter
    } else if gs.breakdown.is_some() {
        Phase::Travel
    } else if gs.boss.readiness.ready && !gs.boss.outcome.attempted {
        Phase::Boss
    } else if gs.continuity.route_services.stop.is_some() {
        Phase::Town
    } else {
        Phase::Travel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_boss_resumes_results_even_without_an_explicit_ending() {
        let mut gs = GameState::default();
        assert!(next_phase(&gs) == Phase::Travel);
        gs.boss.readiness.ready = true;
        assert!(next_phase(&gs) == Phase::Boss);
        gs.boss.outcome.attempted = true;
        assert!(next_phase(&gs) == Phase::Result);
    }

    #[test]
    fn abandoning_is_terminal_even_with_unresolved_crew_or_town_stops() {
        let mut gs = GameState::default();
        gs.continuity.route_services.stop = Some(100);
        gs.continuity.crew_care.pending = Some("organizer".into());
        assert!(next_phase(&gs) == Phase::CrewCare);
        gs.continuity.abandoned = true;
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        assert!(next_phase(&restored) == Phase::Result);
    }

    #[test]
    fn reports_actual_capped_changes_and_inverted_danger() {
        let before = Stats {
            supplies: 19,
            pants: 99,
            ..Stats::default()
        };
        let mut after = before.clone();
        after.supplies = 20;
        after.pants = 100;
        let feedback = Aftermath {
            title: String::new(),
            message: String::new(),
            scene: SceneStage::Travel(crate::game::Region::Heartland),
            before,
            after,
            next: Phase::Travel,
            details: vec![],
        };
        assert_eq!(
            feedback.changes(),
            vec![("ux.supplies", 1, false), ("ux.pants", 1, true)]
        );
    }
}
