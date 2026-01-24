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
    pub const fn key(self) -> &'static str {
        match self {
            Self::Tire => "vehicle.parts.tire",
            Self::Battery => "vehicle.parts.battery",
            Self::Alternator => "vehicle.parts.alt",
            Self::FuelPump => "vehicle.parts.pump",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Future: wear level that increases base breakdown chance
    #[serde(default)]
    pub wear: f32,
    /// Remaining vehicle health (percentage 0-100)
    #[serde(default = "Vehicle::default_health")]
    pub health: f32,
    /// Days remaining before natural breakdown rolls resume
    #[serde(default)]
    pub breakdown_cooldown: u32,
    /// Modifier applied to daily wear accumulation
    #[serde(default = "Vehicle::default_wear_multiplier")]
    pub wear_multiplier: f32,
}

impl Default for Vehicle {
    fn default() -> Self {
        Self {
            wear: 0.0,
            health: Self::default_health(),
            breakdown_cooldown: 0,
            wear_multiplier: Self::default_wear_multiplier(),
        }
    }
}

impl Vehicle {
    const fn default_health() -> f32 {
        100.0
    }

    const fn default_wear_multiplier() -> f32 {
        1.0
    }

    /// Apply durability damage, clamping at zero.
    pub fn apply_damage(&mut self, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        self.health = (self.health - amount).max(0.0);
    }

    /// Restore partial durability, clamping to max.
    pub fn repair(&mut self, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        self.health = (self.health + amount).min(Self::default_health());
    }

    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.health <= 20.0
    }

    /// Ensure the vehicle retains at least the provided health floor.
    pub fn ensure_health_floor(&mut self, floor: f32) {
        if floor <= 0.0 {
            return;
        }
        let capped = floor.min(Self::default_health());
        if self.health < capped {
            self.health = capped;
        }
    }

    /// Reset vehicle wear to zero.
    pub const fn reset_wear(&mut self) {
        self.wear = 0.0;
    }

    /// Set vehicle wear to a specific value, clamped within valid bounds.
    pub const fn set_wear(&mut self, wear: f32) {
        let clamped = wear.clamp(0.0, Self::default_health());
        self.wear = clamped;
    }

    /// Apply wear scaled by the current wear multiplier and return the applied amount.
    pub fn apply_scaled_wear(&mut self, base: f32) -> f32 {
        if base <= 0.0 {
            return 0.0;
        }
        let multiplier = self.wear_multiplier.max(0.0);
        let applied = (base * multiplier).max(0.0);
        if applied <= 0.0 {
            return 0.0;
        }
        self.wear = (self.wear + applied).min(Self::default_health());
        self.apply_damage(applied);
        applied
    }

    /// Configure a cooldown to suppress breakdown rolls for the provided number of days.
    pub const fn set_breakdown_cooldown(&mut self, days: u32) {
        self.breakdown_cooldown = days;
    }

    /// Advance the breakdown cooldown by one day.
    pub const fn tick_breakdown_cooldown(&mut self) {
        if self.breakdown_cooldown > 0 {
            self.breakdown_cooldown -= 1;
        }
    }

    /// Returns true when breakdown rolls should be suppressed.
    #[must_use]
    pub const fn breakdown_suppressed(&self) -> bool {
        self.breakdown_cooldown > 0
    }

    /// Set the wear multiplier latch. Values below zero are clamped to zero.
    pub fn set_wear_multiplier(&mut self, multiplier: f32) {
        if multiplier <= 0.0 {
            self.wear_multiplier = 0.0;
        } else {
            self.wear_multiplier = multiplier;
        }
    }

    /// Clear any custom wear multiplier and restore the default setting.
    pub const fn clear_wear_multiplier(&mut self) {
        self.wear_multiplier = Self::default_wear_multiplier();
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    let roll = rng.gen_range(0..total_weight);
    let mut current_weight = 0;

    let mut selected = None;
    for (item, weight) in options {
        if selected.is_none() {
            current_weight += weight;
            if roll < current_weight {
                selected = Some(item.clone());
            }
        }
    }

    selected.or_else(|| options.first().map(|(item, _)| item.clone()))
}

/// Roll for vehicle breakdown
pub fn breakdown_roll<R: Rng>(base_chance: f32, rng: &mut R) -> bool {
    rng.r#gen::<f32>() < base_chance
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
            let day_started = i32::try_from(game_state.day).unwrap_or(0);
            let breakdown = Breakdown { part, day_started };
            game_state.breakdown = Some(breakdown);
            game_state.day_state.travel.travel_blocked = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameState;
    use rand::rngs::mock::StepRng;

    #[test]
    fn critical_threshold_is_inclusive() {
        let mut vehicle = Vehicle {
            health: 20.0,
            ..Vehicle::default()
        };
        assert!(vehicle.is_critical());
        vehicle.health = 20.1;
        assert!(!vehicle.is_critical());
    }

    #[test]
    fn apply_scaled_wear_respects_zero_base_and_multiplier() {
        let mut vehicle = Vehicle {
            health: 100.0,
            wear: 0.0,
            ..Vehicle::default()
        };
        let applied = vehicle.apply_scaled_wear(0.0);
        assert!(applied.abs() <= f32::EPSILON);
        assert!(vehicle.wear.abs() <= f32::EPSILON);
        assert!((vehicle.health - 100.0).abs() <= f32::EPSILON);

        vehicle.set_wear_multiplier(0.0);
        let applied = vehicle.apply_scaled_wear(5.0);
        assert!(applied.abs() <= f32::EPSILON);
        assert!(vehicle.wear.abs() <= f32::EPSILON);
        assert!((vehicle.health - 100.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn non_positive_damage_and_repairs_do_not_change_state() {
        let mut vehicle = Vehicle::default();
        vehicle.apply_damage(0.0);
        assert!((vehicle.health - 100.0).abs() <= f32::EPSILON);
        vehicle.repair(-1.0);
        assert!((vehicle.health - 100.0).abs() <= f32::EPSILON);
        vehicle.ensure_health_floor(0.0);
        assert!((vehicle.health - 100.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn process_daily_breakdown_sets_breakdown_and_blocks_travel() {
        let mut state = GameState::default();
        let mut rng = StepRng::new(0, 0);

        process_daily_breakdown(&mut state, &mut rng);

        let breakdown = state.breakdown.expect("expected breakdown to be set");
        assert_eq!(breakdown.part, Part::Tire);
        assert!(state.day_state.travel.travel_blocked);
    }

    #[test]
    fn weighted_pick_breaks_on_match() {
        let options = vec![(Part::Tire, 2), (Part::Battery, 1)];
        let mut rng = StepRng::new(0, 0);
        let pick = weighted_pick(&options, &mut rng);
        assert_eq!(pick, Some(Part::Tire));
    }
}
