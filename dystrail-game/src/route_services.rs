//! Route services are available on arrival at each real town on the selected road route.
use crate::GameState;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteServices {
    pub stop: Option<u32>,
    #[serde(default)]
    pub route_id: Option<String>,
    pub traded_at: Option<u32>,
    #[serde(default)]
    pub talked_at: Option<u32>,
    #[serde(default)]
    pub map_reviewed: Option<crate::route::MapCheckpoint>,
}
impl GameState {
    /// Depart the previous stop; open a new one only after crossing a settlement marker.
    pub fn update_route_services(&mut self, previous_miles: f32) {
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
        self.continuity.route_services.stop.is_some()
            && self.continuity.route_services.traded_at != self.continuity.route_services.stop
            && match self.route_trade_kind() {
                1 => self.stats.supplies >= 4,
                2 => self.inventory.spares.tire >= 1 && self.stats.supplies <= 15,
                _ => self.stats.supplies >= 3,
            }
    }
    /// Exchanges are atomic; unavailable resources and overflowing goods cannot trade.
    pub fn trade_at_route_stop(&mut self) -> bool {
        let Some(stop) = self.continuity.route_services.stop else {
            return false;
        };
        if !self.can_trade_at_route_stop() {
            return false;
        }
        match self.route_trade_kind() {
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
