//! Real road routes. Simulation progress is projected onto each persona's physical route.
use crate::{GameState, Region};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
#[derive(Debug, Clone, Deserialize)]
pub struct RoadPoint {
    pub mile: f32,
    pub x: f32,
    pub y: f32,
    pub lon: f32,
    pub lat: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Settlement {
    pub name: String,
    pub state: String,
    pub mile: u16,
    pub x: f32,
    pub y: f32,
    pub lon: f32,
    pub lat: f32,
    pub region: Region,
    pub scene: String,
}
#[derive(Debug, Deserialize)]
pub struct RoadRoute {
    pub id: String,
    pub total_miles: f32,
    pub stops: Vec<Settlement>,
    pub points: Vec<RoadPoint>,
}
#[must_use]
pub fn routes() -> &'static [RoadRoute] {
    static ROUTES: OnceLock<Vec<RoadRoute>> = OnceLock::new();
    ROUTES.get_or_init(|| {
        serde_json::from_str(include_str!("../data/routes.json")).unwrap_or_default()
    })
}
#[must_use]
pub fn for_persona(persona: &str) -> Option<&'static RoadRoute> {
    routes().iter().find(|r| r.id == persona)
}
#[must_use]
pub fn for_state(gs: &GameState) -> Option<&'static RoadRoute> {
    let persona = gs
        .continuity
        .route_services
        .route_id
        .as_deref()
        .or(gs.persona_id.as_deref())
        .unwrap_or("staffer");
    for_persona(persona)
}
#[must_use]
pub fn origin(persona: &str) -> &'static str {
    for_persona(persona)
        .and_then(|r| r.stops.first())
        .map_or("Sacramento", |s| s.name.as_str())
}
#[must_use]
pub fn physical_miles(gs: &GameState) -> f32 {
    physical_at(gs, gs.miles_traveled_actual)
}
/// Convert a road-mile delta into the route's internal progress units.
#[must_use]
pub fn simulation_distance(gs: &GameState, road_miles: f32) -> f32 {
    for_state(gs).map_or(road_miles, |route| {
        road_miles * gs.trail_distance.max(1.0) / route.total_miles.max(1.0)
    })
}
#[must_use]
pub fn physical_at(gs: &GameState, simulation_miles: f32) -> f32 {
    for_state(gs).map_or(simulation_miles, |r| {
        (simulation_miles / gs.trail_distance.max(1.0)).clamp(0.0, 1.0) * r.total_miles
    })
}
#[must_use]
pub fn settlement(gs: &GameState, mile: u32) -> Option<&'static Settlement> {
    for_state(gs)?
        .stops
        .iter()
        .find(|s| u32::from(s.mile) == mile)
}
#[must_use]
pub fn previous(gs: &GameState) -> Option<&'static Settlement> {
    for_state(gs)?
        .stops
        .iter()
        .rev()
        .find(|s| f32::from(s.mile) <= physical_miles(gs))
}
pub fn upcoming(gs: &GameState) -> impl Iterator<Item = &'static Settlement> {
    let miles = physical_miles(gs);
    let arrived = gs.miles_traveled_actual >= gs.trail_distance;
    for_state(gs)
        .into_iter()
        .flat_map(|r| r.stops.iter())
        .filter(move |s| !arrived && f32::from(s.mile) > miles)
}
/// Legacy simulations without a persona retain their original regional thresholds.
#[must_use]
pub const fn region(miles: f32) -> Region {
    if miles < 700.0 {
        Region::Heartland
    } else if miles < 1400.0 {
        Region::RustBelt
    } else {
        Region::Beltway
    }
}
#[must_use]
pub const fn region_key(region: Region) -> &'static str {
    match region {
        Region::PacificCoast => "region.pacific_coast",
        Region::MountainWest => "region.mountain_west",
        Region::Southwest => "region.southwest",
        Region::Heartland => "region.heartland",
        Region::RustBelt => "region.rustbelt",
        Region::Beltway => "region.beltway",
    }
}
#[must_use]
pub fn road_scene(gs: &GameState) -> &'static str {
    previous(gs).map_or("open-heartland-prairie", |s| s.scene.as_str())
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapCheckpoint {
    day_group: u32,
    settlement: u16,
    region: Region,
    #[serde(default)]
    route: String,
}
impl MapCheckpoint {
    #[must_use]
    pub fn current(gs: &GameState) -> Self {
        Self {
            day_group: gs.day.saturating_sub(1) / 5,
            settlement: previous(gs).map_or(0, |s| s.mile),
            region: gs.region,
            route: for_state(gs).map_or_else(String::new, |r| r.id.clone()),
        }
    }
}
impl GameState {
    /// A persisted origin cannot change when names, weather or turn state change.
    pub fn sync_route_location(&mut self) {
        if self.continuity.route_services.route_id.is_none() && self.persona_id.is_some() {
            self.continuity
                .route_services
                .route_id
                .clone_from(&self.persona_id);
            if self.continuity.route_services.stop.is_some() {
                let traded =
                    self.continuity.route_services.traded_at == self.continuity.route_services.stop;
                self.continuity.route_services.stop = previous(self)
                    .filter(|s| s.mile > 0)
                    .map(|s| u32::from(s.mile));
                if traded {
                    self.continuity.route_services.traded_at = self.continuity.route_services.stop;
                }
            }
        }
        self.region = if self.continuity.route_services.route_id.is_some() {
            previous(self).map_or(Region::Heartland, |s| s.region)
        } else {
            region(self.miles_traveled_actual)
        };
    }
}
#[cfg(test)]
mod tests;
