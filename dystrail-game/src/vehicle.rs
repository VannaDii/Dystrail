//! Vehicle breakdown system
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Part {
    Tire,
    Battery,
    Alternator,
    FuelPump,
}

impl Part {
    /// Get the translation key for this part
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Part::Tire => "vehicle.parts.tire",
            Part::Battery => "vehicle.parts.battery",
            Part::Alternator => "vehicle.parts.alt",
            Part::FuelPump => "vehicle.parts.pump",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Future: wear level that increases base breakdown chance
    #[serde(default)]
    pub wear: f32,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self { wear: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakdown {
    pub part: Part,
    pub day_started: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VehicleConfig {
    pub breakdown_chance: f32,
}

/// Part weights for weighted random selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartWeights {
    pub tire: u32,
    pub battery: u32,
    pub alt: u32,
    pub pump: u32,
}

impl Default for PartWeights {
    fn default() -> Self {
        Self {
            tire: 50,
            battery: 20,
            alt: 15,
            pump: 15,
        }
    }
}

/// Weighted random selection from a list of options
pub fn weighted_pick<T, R>(options: &[(T, u32)], rng: &mut R) -> Option<T>
where
    R: Rng,
    T: Clone,
{
    let total_weight: u32 = options.iter().map(|(_, weight)| *weight).sum();
    if total_weight == 0 {
        return None;
    }

    let roll = rng.random_range(0..total_weight);
    let mut current_weight = 0;

    for (item, weight) in options {
        current_weight += weight;
        if roll < current_weight {
            return Some(item.clone());
        }
    }

    options.first().map(|(item, _)| item.clone())
}

/// Roll for vehicle breakdown
pub fn breakdown_roll<R: Rng>(base_chance: f32, rng: &mut R) -> bool {
    rng.random::<f32>() < base_chance
}

/// Process daily breakdown chance
pub fn process_daily_breakdown<R: Rng>(game_state: &mut crate::state::GameState, rng: &mut R) {
    let breakdown_chance = 0.1; // 10% chance per day
    if breakdown_roll(breakdown_chance, rng) && game_state.breakdown.is_none() {
        let weights = PartWeights::default();
        let options = [
            (Part::Tire, weights.tire),
            (Part::Battery, weights.battery),
            (Part::Alternator, weights.alt),
            (Part::FuelPump, weights.pump),
        ];

        if let Some(part) = weighted_pick(&options, rng) {
            game_state.breakdown = Some(Breakdown {
                part,
                day_started: i32::try_from(game_state.day).unwrap_or(0),
            });
            game_state.travel_blocked = true;
        }
    }
}
