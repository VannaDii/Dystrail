//! Route services are available on arrival at each real town on the selected road route.
use crate::GameState;
use serde::{Deserialize, Serialize};
/// One visible, guaranteed benefit per town conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalReward {
    Credibility,
    Receipt,
    Ally,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteServices {
    pub stop: Option<u32>,
    #[serde(default)]
    pub trading: bool,
    #[serde(default)]
    pub route_id: Option<String>,
    pub traded_at: Option<u32>,
    #[serde(default)]
    pub talked_at: Option<u32>,
    /// The benefit actually awarded at `talked_at`.
    #[serde(default)]
    pub talk_reward: Option<LocalReward>,
    #[serde(default)]
    pub map_reviewed: Option<crate::route::MapCheckpoint>,
}
impl GameState {
    /// Previewing a town reward never consumes randomness or changes the run.
    #[must_use]
    pub fn local_conversation_reward(&self) -> Option<LocalReward> {
        let stop = self.continuity.route_services.stop?;
        Some(match (stop / 10) % 3 {
            0 if self.stats.credibility < 20 => LocalReward::Credibility,
            2 if self.stats.allies < 50 => LocalReward::Ally,
            _ => LocalReward::Receipt,
        })
    }
    /// Visiting a conversation again never awards or changes the recorded benefit.
    pub fn claim_local_conversation(&mut self) -> Option<LocalReward> {
        let stop = self.continuity.route_services.stop?;
        if self.continuity.route_services.talked_at == Some(stop) {
            return None;
        }
        let reward = self.local_conversation_reward()?;
        match reward {
            LocalReward::Credibility => self.stats.credibility += 1,
            LocalReward::Ally => self.stats.allies += 1,
            LocalReward::Receipt => self.receipts.push(format!("local-record:{stop}")),
        }
        self.continuity.route_services.talked_at = Some(stop);
        self.continuity.route_services.talk_reward = Some(reward);
        Some(reward)
    }
    /// Depart the previous stop; open a new one only after crossing a settlement marker.
    pub fn update_route_services(&mut self, previous_miles: f32) {
        self.continuity.route_services.trading = false;
        self.sync_route_location();
        let previous = crate::route::physical_at(self, previous_miles);
        let current = crate::route::physical_miles(self);
        self.continuity.route_services.stop = crate::route::for_state(self)
            .and_then(|route| {
                route.stops.iter().rev().find(|stop| {
                    stop.name != "D.C."
                        && f32::from(stop.mile) > previous
                        && f32::from(stop.mile) <= current
                })
            })
            .map(|stop| u32::from(stop.mile));
    }
    /// Each settlement's local surplus determines its exchange.
    #[must_use]
    pub fn route_trade_kind(&self) -> u8 {
        self.continuity
            .route_services
            .stop
            .map_or(0, |mile| u8::try_from((mile / 10) % 3).unwrap_or(0))
    }
    #[must_use]
    pub fn can_trade_at_route_stop(&self) -> bool {
        self.can_route_trade(self.route_trade_kind())
    }
    #[must_use]
    pub fn can_route_trade(&self, kind: u8) -> bool {
        self.continuity.route_services.stop.is_some()
            && self.continuity.route_services.traded_at != self.continuity.route_services.stop
            && kind < 3
            && self.ending.is_none()
            && self.current_encounter.is_none()
            && match kind {
                1 => self.stats.supplies >= 4,
                2 => self.inventory.spares.tire >= 1 && self.stats.supplies <= 15,
                _ => self.stats.supplies >= 3,
            }
    }
    /// Exchanges are atomic; unavailable resources and overflowing goods cannot trade.
    pub fn trade_at_route_stop(&mut self) -> bool {
        self.trade_route_offer(self.route_trade_kind())
    }
    /// Resolve the player-selected offer once per town, with the same affordability checks.
    pub fn trade_route_offer(&mut self, kind: u8) -> bool {
        let Some(stop) = self.continuity.route_services.stop else {
            return false;
        };
        if !self.can_route_trade(kind) {
            return false;
        }
        match kind {
            1 => {
                self.stats.supplies -= 4;
                self.inventory.spares.battery += 1;
            }
            2 => {
                self.inventory.spares.tire -= 1;
                self.stats.supplies += 5;
            }
            _ => {
                self.stats.supplies -= 3;
                self.inventory.spares.tire += 1;
            }
        }
        self.continuity.route_services.traded_at = Some(stop);
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn town_rewards_are_varied_visible_and_claimed_once_across_saves() {
        for (stop, reward) in [
            (0, LocalReward::Credibility),
            (10, LocalReward::Receipt),
            (20, LocalReward::Ally),
        ] {
            let mut gs = GameState::default();
            gs.continuity.route_services.stop = Some(stop);
            let before = gs.clone();
            assert_eq!(gs.local_conversation_reward(), Some(reward));
            assert_eq!(gs.claim_local_conversation(), Some(reward));
            assert_eq!(
                gs.stats.credibility - before.stats.credibility,
                i32::from(reward == LocalReward::Credibility)
            );
            assert_eq!(
                gs.stats.allies - before.stats.allies,
                i32::from(reward == LocalReward::Ally)
            );
            assert_eq!(
                gs.receipts.len() - before.receipts.len(),
                usize::from(reward == LocalReward::Receipt)
            );
            let mut loaded: GameState =
                serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
            assert_eq!(loaded.claim_local_conversation(), None);
            assert_eq!(loaded.stats, gs.stats);
            assert_eq!(loaded.receipts, gs.receipts);
            assert_eq!(loaded.continuity.route_services.talk_reward, Some(reward));
            loaded.continuity.route_services.stop = Some(stop + 30);
            assert_eq!(loaded.claim_local_conversation(), Some(reward));
        }
    }
    #[test]
    fn capped_social_stats_offer_a_receipt_instead_of_an_empty_reward() {
        let mut gs = GameState::default();
        gs.stats.credibility = 20;
        gs.stats.allies = 50;
        for stop in [0, 20] {
            gs.continuity.route_services.stop = Some(stop);
            assert_eq!(gs.local_conversation_reward(), Some(LocalReward::Receipt));
            assert_eq!(gs.claim_local_conversation(), Some(LocalReward::Receipt));
        }
        assert_eq!(gs.stats.credibility, 20);
        assert_eq!(gs.stats.allies, 50);
        assert_eq!(gs.receipts.len(), 2);
    }
    #[test]
    fn arrival_trade_cost_and_departure_survive_save() {
        let mut gs = GameState {
            persona_id: Some("staffer".into()),
            ..GameState::default()
        };
        gs.sync_route_location();
        let route = crate::route::for_state(&gs).unwrap();
        let town = route.stops[1].mile;
        let to_sim = |m: f32| m / route.total_miles * gs.trail_distance;
        let before = to_sim(f32::from(town) - 1.0);
        gs.miles_traveled_actual = to_sim(f32::from(town) + 1.0);
        gs.stats.supplies = 10;
        gs.update_route_services(before);
        assert_eq!(gs.continuity.route_services.stop, Some(u32::from(town)));
        let tires = gs.inventory.spares.tire;
        assert!(gs.trade_at_route_stop());
        assert_eq!(gs.stats.supplies, 6);
        assert_eq!(gs.inventory.spares.tire, tires);
        assert_eq!(gs.inventory.spares.battery, 1);
        let mut loaded: GameState =
            serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        assert!(!loaded.trade_at_route_stop());
        let before = loaded.miles_traveled_actual;
        loaded.miles_traveled_actual += 1.0;
        loaded.update_route_services(before);
        assert_eq!(loaded.continuity.route_services.stop, None);
    }
}
