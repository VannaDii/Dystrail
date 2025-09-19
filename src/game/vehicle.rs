use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vehicle part that can break down
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Part {
    Tire,
    Battery,
    Alternator,
    FuelPump,
}

impl Part {
    /// Get the translation key for this part
    pub fn key(self) -> &'static str {
        match self {
            Part::Tire => "vehicle.parts.tire",
            Part::Battery => "vehicle.parts.battery",
            Part::Alternator => "vehicle.parts.alt",
            Part::FuelPump => "vehicle.parts.pump",
        }
    }
}

/// Vehicle state including wear and spares
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Future: wear level that increases base breakdown chance
    #[serde(default)]
    pub wear: f32,
    // Spare parts inventory is stored in state.rs Spares struct
}

impl Default for Vehicle {
    fn default() -> Self {
        Self { wear: 0.0 }
    }
}

/// Active breakdown preventing travel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakdown {
    pub part: Part,
    pub day_started: i32,
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

/// Repair action costs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairCosts {
    pub use_spare_supplies: i32,
    pub hack_supplies: i32,
    pub hack_cred: i32,
    pub hack_day: i32,
}

impl Default for RepairCosts {
    fn default() -> Self {
        Self {
            use_spare_supplies: 1,
            hack_supplies: 3,
            hack_cred: 1,
            hack_day: 1,
        }
    }
}

/// Mechanic encounter hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanicHook {
    pub enabled: bool,
    pub chance_clear: f32,
    pub day_cost: i32,
}

impl Default for MechanicHook {
    fn default() -> Self {
        Self {
            enabled: false,
            chance_clear: 0.15,
            day_cost: 1,
        }
    }
}

/// Complete vehicle system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleConfig {
    #[serde(default = "default_base_breakdown_chance")]
    pub base_breakdown_chance: f32,
    #[serde(default = "default_pace_factor")]
    pub pace_factor: HashMap<String, f32>,
    #[serde(default = "default_weather_factor")]
    pub weather_factor: HashMap<String, f32>,
    #[serde(default)]
    pub part_weights: PartWeights,
    #[serde(default)]
    pub repair_costs: RepairCosts,
    #[serde(default)]
    pub mechanic_hook: MechanicHook,
}

fn default_base_breakdown_chance() -> f32 {
    0.005
}

fn default_pace_factor() -> HashMap<String, f32> {
    let mut map = HashMap::new();
    map.insert("steady".to_string(), 1.0);
    map.insert("heated".to_string(), 1.2);
    map.insert("blitz".to_string(), 1.5);
    map
}

fn default_weather_factor() -> HashMap<String, f32> {
    let mut map = HashMap::new();
    map.insert("Clear".to_string(), 1.0);
    map.insert("Storm".to_string(), 1.3);
    map.insert("HeatWave".to_string(), 1.4);
    map.insert("ColdSnap".to_string(), 1.1);
    map.insert("Smoke".to_string(), 1.1);
    map
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            base_breakdown_chance: default_base_breakdown_chance(),
            pace_factor: default_pace_factor(),
            weather_factor: default_weather_factor(),
            part_weights: PartWeights::default(),
            repair_costs: RepairCosts::default(),
            mechanic_hook: MechanicHook::default(),
        }
    }
}

impl VehicleConfig {
    /// Load configuration from JSON, falling back to defaults
    pub async fn load() -> Self {
        match gloo::net::http::Request::get("/static/assets/data/vehicle.json")
            .send()
            .await
        {
            Ok(response) => {
                if response.ok() {
                    if let Ok(text) = response.text().await {
                        if let Ok(config) = serde_json::from_str(&text) {
                            return config;
                        }
                    }
                }
            }
            Err(_) => {}
        }
        // Fall back to defaults if loading fails
        Self::default()
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.base_breakdown_chance) {
            return Err("base_breakdown_chance must be between 0 and 1".to_string());
        }

        for (name, factor) in &self.pace_factor {
            if *factor <= 0.0 {
                return Err(format!("pace_factor for {} must be > 0", name));
            }
        }

        for (name, factor) in &self.weather_factor {
            if *factor <= 0.0 {
                return Err(format!("weather_factor for {} must be > 0", name));
            }
        }

        let total_weight = self.part_weights.tire
            + self.part_weights.battery
            + self.part_weights.alt
            + self.part_weights.pump;
        if total_weight == 0 {
            return Err("At least one part weight must be > 0".to_string());
        }

        if self.repair_costs.use_spare_supplies < 0
            || self.repair_costs.hack_supplies < 0
            || self.repair_costs.hack_cred < 0
            || self.repair_costs.hack_day < 0
        {
            return Err("Repair costs must be >= 0".to_string());
        }

        Ok(())
    }
}

/// Perform breakdown roll based on pace and weather
pub fn breakdown_roll<R: Rng>(
    pace: &str,
    weather: &str, // Future: when weather system exists
    cfg: &VehicleConfig,
    rng: &mut R,
) -> Option<Part> {
    let p_factor = cfg.pace_factor.get(pace).copied().unwrap_or(1.0);
    let w_factor = cfg.weather_factor.get(weather).copied().unwrap_or(1.0);

    let chance = cfg.base_breakdown_chance * p_factor * w_factor;

    let roll: f32 = rng.random();
    if roll < chance {
        Some(weighted_pick(&cfg.part_weights, rng))
    } else {
        None
    }
}

/// Select a part based on weights
pub fn weighted_pick<R: Rng>(weights: &PartWeights, rng: &mut R) -> Part {
    let bag = [
        (Part::Tire, weights.tire),
        (Part::Battery, weights.battery),
        (Part::Alternator, weights.alt),
        (Part::FuelPump, weights.pump),
    ];

    let total: u32 = bag.iter().map(|(_, w)| w).sum();
    if total == 0 {
        return Part::Tire; // fallback
    }

    let mut roll = rng.random_range(0..total);
    for (part, weight) in bag {
        if roll < weight {
            return part;
        }
        roll -= weight;
    }
    Part::Tire // fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_breakdown_chance_calculation() {
        let cfg = VehicleConfig::default();
        let mut rng = ChaCha20Rng::seed_from_u64(42);

        // Test with base values
        let _result = breakdown_roll("steady", "Clear", &cfg, &mut rng);
        // With 0.5% base chance, should be quite rare

        // Test higher pace increases chance
        let mut high_count = 0;
        let mut low_count = 0;

        for _ in 0..10000 {
            if breakdown_roll("blitz", "Storm", &cfg, &mut rng).is_some() {
                high_count += 1;
            }
            if breakdown_roll("steady", "Clear", &cfg, &mut rng).is_some() {
                low_count += 1;
            }
        }

        // blitz + storm should have higher breakdown rate than steady + clear
        assert!(high_count > low_count);
    }

    #[test]
    fn test_weighted_pick_distribution() {
        let weights = PartWeights {
            tire: 50,
            battery: 20,
            alt: 15,
            pump: 15,
        };

        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let mut counts = [0; 4];

        for _ in 0..10000 {
            match weighted_pick(&weights, &mut rng) {
                Part::Tire => counts[0] += 1,
                Part::Battery => counts[1] += 1,
                Part::Alternator => counts[2] += 1,
                Part::FuelPump => counts[3] += 1,
            }
        }

        // Tire should be roughly 50% of picks
        assert!(counts[0] > 4500 && counts[0] < 5500);
        // Battery should be roughly 20% of picks
        assert!(counts[1] > 1500 && counts[1] < 2500);
    }

    #[test]
    fn test_config_validation() {
        let mut cfg = VehicleConfig::default();
        assert!(cfg.validate().is_ok());

        // Test invalid base chance
        cfg.base_breakdown_chance = 1.5;
        assert!(cfg.validate().is_err());

        cfg.base_breakdown_chance = 0.005;

        // Test invalid pace factor
        cfg.pace_factor.insert("test".to_string(), -1.0);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_deterministic_behavior() {
        let cfg = VehicleConfig::default();

        // Test that same seed produces same results
        let mut rng1 = ChaCha20Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha20Rng::seed_from_u64(12345);

        let results1: Vec<_> = (0..100)
            .map(|_| breakdown_roll("steady", "Clear", &cfg, &mut rng1))
            .collect();

        let results2: Vec<_> = (0..100)
            .map(|_| breakdown_roll("steady", "Clear", &cfg, &mut rng2))
            .collect();

        assert_eq!(
            results1, results2,
            "Same seed should produce same breakdown sequence"
        );
    }

    #[test]
    fn test_breakdown_chance_math() {
        let cfg = VehicleConfig::default();

        // Test specific chance calculation
        let base = 0.005; // 0.5%
        let pace_factor = cfg.pace_factor.get("blitz").copied().unwrap_or(1.0);
        let weather_factor = cfg.weather_factor.get("Storm").copied().unwrap_or(1.0);

        let expected_chance = base * pace_factor * weather_factor;

        // Verify chance is calculated correctly (0.005 * 1.5 * 1.3 = 0.00975)
        assert!((expected_chance - 0.00975).abs() < 0.0001);
    }

    #[test]
    fn test_part_key_translation() {
        // Test that each part returns correct translation key
        assert_eq!(Part::Tire.key(), "vehicle.parts.tire");
        assert_eq!(Part::Battery.key(), "vehicle.parts.battery");
        assert_eq!(Part::Alternator.key(), "vehicle.parts.alt");
        assert_eq!(Part::FuelPump.key(), "vehicle.parts.pump");
    }

    #[test]
    fn test_edge_cases() {
        let cfg = VehicleConfig::default();
        let mut rng = ChaCha20Rng::seed_from_u64(999);

        // Test with zero weights (should not panic)
        let zero_weights = PartWeights {
            tire: 0,
            battery: 0,
            alt: 0,
            pump: 1, // At least one must be > 0
        };

        let part = weighted_pick(&zero_weights, &mut rng);
        assert_eq!(part, Part::FuelPump); // Should always pick the only non-zero option

        // Test with maximum values
        let _result = breakdown_roll("blitz", "HeatWave", &cfg, &mut rng);
        // Should handle high multipliers without overflow
        // This is mainly a non-panic test
    }

    #[test]
    fn test_config_defaults() {
        let cfg = VehicleConfig::default();

        // Verify default values match requirements
        assert_eq!(cfg.base_breakdown_chance, 0.005);
        assert_eq!(cfg.pace_factor.get("steady"), Some(&1.0));
        assert_eq!(cfg.pace_factor.get("heated"), Some(&1.2));
        assert_eq!(cfg.pace_factor.get("blitz"), Some(&1.5));

        assert_eq!(cfg.weather_factor.get("Clear"), Some(&1.0));
        assert_eq!(cfg.weather_factor.get("Storm"), Some(&1.3));

        assert_eq!(cfg.part_weights.tire, 50);
        assert_eq!(cfg.part_weights.battery, 20);
        assert_eq!(cfg.part_weights.alt, 15);
        assert_eq!(cfg.part_weights.pump, 15);

        assert_eq!(cfg.repair_costs.use_spare_supplies, 1);
        assert_eq!(cfg.repair_costs.hack_supplies, 3);
        assert_eq!(cfg.repair_costs.hack_cred, 1);
        assert_eq!(cfg.repair_costs.hack_day, 1);
    }
}
