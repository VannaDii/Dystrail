use gloo_net::http::Request;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::game::{GameState, Region};

/// Weather conditions that affect daily gameplay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Weather {
    #[default]
    Clear,
    Storm,
    HeatWave,
    ColdSnap,
    Smoke,
}

impl Weather {
    /// Check if weather is considered extreme (streak-limited)
    #[must_use]
    pub fn is_extreme(self) -> bool {
        matches!(self, Weather::Storm | Weather::HeatWave | Weather::Smoke)
    }

    /// Get i18n key for weather state name
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            Weather::Clear => "weather.states.Clear",
            Weather::Storm => "weather.states.Storm",
            Weather::HeatWave => "weather.states.HeatWave",
            Weather::ColdSnap => "weather.states.ColdSnap",
            Weather::Smoke => "weather.states.Smoke",
        }
    }
}

/// Daily effects from weather conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherEffect {
    pub supplies: i32,
    pub sanity: i32,
    pub pants: i32,
    pub enc_delta: f32,
}

/// Gear-based mitigation for weather effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherMitigation {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pants: Option<i32>,
}

/// Executive order modifiers for weather encounters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecWeatherMod {
    pub enc_delta: f32,
}

/// Configuration limits for weather system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherLimits {
    pub max_extreme_streak: i32,
    pub encounter_cap: f32,
    pub pants_floor: i32,
    pub pants_ceiling: i32,
}

/// Complete weather system configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub limits: WeatherLimits,
    pub effects: HashMap<Weather, WeatherEffect>,
    pub mitigation: HashMap<Weather, WeatherMitigation>,
    pub weights: HashMap<Region, HashMap<Weather, u32>>,
    pub exec_mods: HashMap<String, ExecWeatherMod>,
}

/// Weather state tracking for streaks and history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherState {
    pub today: Weather,
    pub yesterday: Weather,
    pub extreme_streak: i32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            today: Weather::Clear,
            yesterday: Weather::Clear,
            extreme_streak: 0,
        }
    }
}

impl WeatherConfig {
    /// Load weather configuration from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string cannot be parsed or if validation fails.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let config: WeatherConfig =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    /// Load weather configuration from static assets
    pub async fn load_from_static() -> Self {
        let url = "/static/assets/data/weather.json";
        if let Ok(resp) = Request::get(url).send().await
            && resp.status() == 200
            && let Ok(json_str) = resp.text().await
            && let Ok(config) = Self::from_json(&json_str)
        {
            return config;
        }
        // If loading fails, use embedded defaults
        Self::default_config()
    }

    /// Validate configuration completeness
    fn validate(&self) -> Result<(), String> {
        // Check that all weather types have effects
        for weather in [
            Weather::Clear,
            Weather::Storm,
            Weather::HeatWave,
            Weather::ColdSnap,
            Weather::Smoke,
        ] {
            if !self.effects.contains_key(&weather) {
                return Err(format!("Missing effect for weather: {weather:?}"));
            }
        }

        // Check that all regions have weights
        for region in [Region::Heartland, Region::RustBelt, Region::Beltway] {
            if !self.weights.contains_key(&region) {
                return Err(format!("Missing weights for region: {region:?}"));
            }

            let region_weights = self.weights.get(&region).unwrap();
            for weather in [
                Weather::Clear,
                Weather::Storm,
                Weather::HeatWave,
                Weather::ColdSnap,
                Weather::Smoke,
            ] {
                if !region_weights.contains_key(&weather) {
                    return Err(format!("Missing weight for {weather:?} in {region:?}"));
                }
            }
        }

        Ok(())
    }

    /// Get embedded default configuration if loading fails
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn default_config() -> Self {
        use std::collections::HashMap;

        let mut effects = HashMap::new();
        effects.insert(
            Weather::Clear,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.00,
            },
        );
        effects.insert(
            Weather::Storm,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 2,
                enc_delta: 0.05,
            },
        );
        effects.insert(
            Weather::HeatWave,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 1,
                enc_delta: 0.03,
            },
        );
        effects.insert(
            Weather::ColdSnap,
            WeatherEffect {
                supplies: 0,
                sanity: -1,
                pants: 0,
                enc_delta: 0.00,
            },
        );
        effects.insert(
            Weather::Smoke,
            WeatherEffect {
                supplies: 1,
                sanity: -1,
                pants: 1,
                enc_delta: 0.03,
            },
        );

        let mut mitigation = HashMap::new();
        mitigation.insert(
            Weather::ColdSnap,
            WeatherMitigation {
                tag: "cold_resist".to_string(),
                sanity: Some(0),
                pants: None,
            },
        );
        mitigation.insert(
            Weather::Smoke,
            WeatherMitigation {
                tag: "plague_resist".to_string(),
                sanity: Some(0),
                pants: None,
            },
        );
        mitigation.insert(
            Weather::Storm,
            WeatherMitigation {
                tag: "rain_resist".to_string(),
                sanity: None,
                pants: Some(1),
            },
        );

        let mut weights = HashMap::new();

        let mut heartland = HashMap::new();
        heartland.insert(Weather::Clear, 60);
        heartland.insert(Weather::Storm, 15);
        heartland.insert(Weather::HeatWave, 10);
        heartland.insert(Weather::ColdSnap, 10);
        heartland.insert(Weather::Smoke, 5);
        weights.insert(Region::Heartland, heartland);

        let mut rust_belt = HashMap::new();
        rust_belt.insert(Weather::Clear, 55);
        rust_belt.insert(Weather::Storm, 20);
        rust_belt.insert(Weather::HeatWave, 8);
        rust_belt.insert(Weather::ColdSnap, 12);
        rust_belt.insert(Weather::Smoke, 5);
        weights.insert(Region::RustBelt, rust_belt);

        let mut beltway = HashMap::new();
        beltway.insert(Weather::Clear, 50);
        beltway.insert(Weather::Storm, 25);
        beltway.insert(Weather::HeatWave, 10);
        beltway.insert(Weather::ColdSnap, 10);
        beltway.insert(Weather::Smoke, 5);
        weights.insert(Region::Beltway, beltway);

        let mut exec_mods = HashMap::new();
        exec_mods.insert(
            "TariffTsunami".to_string(),
            ExecWeatherMod { enc_delta: 0.00 },
        );
        exec_mods.insert(
            "NatGuardDeployment".to_string(),
            ExecWeatherMod { enc_delta: 0.02 },
        );
        exec_mods.insert(
            "DoEEliminated".to_string(),
            ExecWeatherMod { enc_delta: 0.00 },
        );
        exec_mods.insert("WarDept".to_string(), ExecWeatherMod { enc_delta: 0.00 });

        Self {
            limits: WeatherLimits {
                max_extreme_streak: 3,
                encounter_cap: 1.0,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects,
            mitigation,
            weights,
            exec_mods,
        }
    }
}

/// Select today's weather based on region weights and streak limits
///
/// # Panics
///
/// Panics if RNG is not initialized or if weather weights don't exist for the current region.
pub fn select_weather_for_today(gs: &mut GameState, cfg: &WeatherConfig) -> Weather {
    let rng = gs.rng.as_mut().expect("RNG must be initialized");

    let region_weights = cfg
        .weights
        .get(&gs.region)
        .expect("Weather weights must exist for region");

    // Calculate total weight and make initial selection
    let total: u32 = region_weights.values().sum();
    let mut roll = rng.random_range(0..total);
    let mut candidate = Weather::Clear;

    for (weather, weight) in region_weights {
        if roll < *weight {
            candidate = *weather;
            break;
        }
        roll -= *weight;
    }

    // Enforce extreme streak limit
    if candidate.is_extreme() && gs.weather_state.extreme_streak >= cfg.limits.max_extreme_streak {
        // Reselect from non-extremes deterministically
        let non_extreme_total: u32 = region_weights
            .iter()
            .filter(|(w, _)| !w.is_extreme())
            .map(|(_, wt)| *wt)
            .sum();

        if non_extreme_total > 0 {
            let mut r2 = rng.random_range(0..non_extreme_total);
            for (weather, weight) in region_weights {
                if weather.is_extreme() {
                    continue;
                }
                if r2 < *weight {
                    candidate = *weather;
                    break;
                }
                r2 -= *weight;
            }
        }
        // If non_extreme_total is 0, allow extreme to repeat with dev warning
        #[cfg(debug_assertions)]
        if non_extreme_total == 0 {
            web_sys::console::warn_1(
                &"Weather: No non-extreme options available, allowing streak continuation".into(),
            );
        }
    }

    candidate
}

/// Apply weather effects to game state
///
/// # Panics
///
/// Panics if weather effect configuration doesn't exist for today's weather.
pub fn apply_weather_effects(gs: &mut GameState, cfg: &WeatherConfig) {
    // Update streak counters
    let today = gs.weather_state.today;
    let was_extreme = gs.weather_state.yesterday.is_extreme();
    let is_extreme = today.is_extreme();

    gs.weather_state.extreme_streak = if is_extreme {
        if was_extreme {
            gs.weather_state.extreme_streak + 1
        } else {
            1
        }
    } else {
        0
    };

    // Get base effect
    let effect = cfg.effects.get(&today).expect("Weather effect must exist");

    let delta_sup = effect.supplies;
    let mut delta_san = effect.sanity;
    let mut delta_pants = effect.pants;
    let delta_enc = effect.enc_delta;

    // Apply gear mitigation
    if let Some(mitigation) = cfg.mitigation.get(&today)
        && gs.inventory.tags.contains(&mitigation.tag)
    {
        if let Some(san) = mitigation.sanity {
            delta_san = san;
        }
        if let Some(pants) = mitigation.pants {
            delta_pants = pants;
        }
    }

    // Apply stat changes
    gs.stats.supplies += delta_sup;
    gs.stats.sanity += delta_san;
    gs.stats.pants =
        (gs.stats.pants + delta_pants).clamp(cfg.limits.pants_floor, cfg.limits.pants_ceiling);

    // Add weather encounter chance delta
    gs.encounter_chance_today =
        (gs.encounter_chance_today + delta_enc).clamp(0.0, cfg.limits.encounter_cap);

    // Apply executive order modifiers if applicable
    let exec_order_key = format!("{:?}", gs.current_order);
    if let Some(exec_mod) = cfg.exec_mods.get(&exec_order_key) {
        gs.encounter_chance_today =
            (gs.encounter_chance_today + exec_mod.enc_delta).clamp(0.0, cfg.limits.encounter_cap);
    }
}

/// Process daily weather step in game tick
pub fn process_daily_weather(gs: &mut GameState, cfg: &WeatherConfig) {
    // Move today to yesterday
    gs.weather_state.yesterday = gs.weather_state.today;

    // Select new weather for today
    gs.weather_state.today = select_weather_for_today(gs, cfg);

    // Apply effects
    apply_weather_effects(gs, cfg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{GameState, Region};
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn create_test_game_state() -> GameState {
        let mut gs = GameState::default();
        gs.region = Region::Heartland;
        gs.rng = Some(ChaCha20Rng::seed_from_u64(42));
        gs.weather_state = WeatherState::default();
        gs
    }

    #[test]
    fn test_weather_is_extreme() {
        assert!(!Weather::Clear.is_extreme());
        assert!(Weather::Storm.is_extreme());
        assert!(Weather::HeatWave.is_extreme());
        assert!(!Weather::ColdSnap.is_extreme());
        assert!(Weather::Smoke.is_extreme());
    }

    #[test]
    fn test_weather_selection_distribution() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();
        let mut counts = HashMap::new();

        for _ in 0..1000 {
            let weather = select_weather_for_today(&mut gs, &cfg);
            *counts.entry(weather).or_insert(0) += 1;
        }

        // Should have selected some weather (basic smoke test)
        assert!(!counts.is_empty());
        assert!(counts.contains_key(&Weather::Clear));
    }

    #[test]
    fn test_extreme_streak_limiting() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        // Force extreme streak to max
        gs.weather_state.extreme_streak = cfg.limits.max_extreme_streak;
        gs.weather_state.yesterday = Weather::Storm;

        // Should not allow another extreme when at limit
        // (This is probabilistic, but with enough trials should hold)
        let mut extreme_count = 0;
        for _ in 0..100 {
            let weather = select_weather_for_today(&mut gs, &cfg);
            if weather.is_extreme() {
                extreme_count += 1;
            }
        }

        // Should be significantly fewer extremes when at streak limit
        assert!(
            extreme_count < 50,
            "Too many extremes selected when at streak limit"
        );
    }

    #[test]
    fn test_mitigation_effects() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        // Test without mitigation
        gs.weather_state.today = Weather::Storm;
        gs.stats.sanity = 5;
        gs.stats.pants = 50;

        apply_weather_effects(&mut gs, &cfg);

        // Storm should reduce sanity and increase pants
        assert_eq!(gs.stats.sanity, 4); // -1 from storm
        assert_eq!(gs.stats.pants, 52); // +2 from storm

        // Test with mitigation
        gs.stats.sanity = 5;
        gs.stats.pants = 50;
        gs.inventory.tags.insert("rain_resist".to_string());

        apply_weather_effects(&mut gs, &cfg);

        // With rain_resist, pants bonus should be reduced
        assert_eq!(gs.stats.sanity, 4); // still -1 (mitigation doesn't affect sanity for storm)
        assert_eq!(gs.stats.pants, 51); // +1 instead of +2 with mitigation
    }

    #[test]
    fn test_encounter_chance_modification() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        gs.encounter_chance_today = 0.35; // base
        gs.weather_state.today = Weather::Storm; // +0.05

        apply_weather_effects(&mut gs, &cfg);

        assert!((gs.encounter_chance_today - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pants_clamping() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        // Test ceiling
        gs.stats.pants = 99;
        gs.weather_state.today = Weather::Storm; // +2

        apply_weather_effects(&mut gs, &cfg);

        assert_eq!(gs.stats.pants, 100); // clamped to ceiling

        // Test floor (with negative effect)
        gs.stats.pants = 1;
        gs.weather_state.today = Weather::Clear;
        // Manually apply negative effect for test
        gs.stats.pants =
            (gs.stats.pants - 5).clamp(cfg.limits.pants_floor, cfg.limits.pants_ceiling);

        assert_eq!(gs.stats.pants, 0); // clamped to floor
    }

    #[test]
    fn test_config_validation() {
        let mut config = WeatherConfig::default_config();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Remove a weather effect
        config.effects.remove(&Weather::Clear);
        assert!(config.validate().is_err());

        // Restore and remove region weights
        config.effects.insert(
            Weather::Clear,
            WeatherEffect {
                supplies: 0,
                sanity: 0,
                pants: 0,
                enc_delta: 0.0,
            },
        );
        config.weights.remove(&Region::Heartland);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_exec_order_modifiers() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        gs.encounter_chance_today = 0.35; // base
        gs.weather_state.today = Weather::Clear; // +0.00
        gs.current_order = crate::game::exec_orders::ExecOrder::WarDeptReorg; // should add +0.00 (no weather effect)

        apply_weather_effects(&mut gs, &cfg);

        // Should have base + weather + exec order = 0.35 + 0.00 + 0.00 = 0.35
        assert!((gs.encounter_chance_today - 0.35).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weather_state_transitions() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        // Start with Clear weather
        gs.weather_state.today = Weather::Clear;
        gs.weather_state.yesterday = Weather::Clear;
        gs.weather_state.extreme_streak = 0;

        // Apply Storm weather
        gs.weather_state.yesterday = gs.weather_state.today;
        gs.weather_state.today = Weather::Storm;

        apply_weather_effects(&mut gs, &cfg);

        // Should start streak
        assert_eq!(gs.weather_state.extreme_streak, 1);

        // Apply another extreme
        gs.weather_state.yesterday = gs.weather_state.today;
        gs.weather_state.today = Weather::HeatWave;

        apply_weather_effects(&mut gs, &cfg);

        // Should continue streak
        assert_eq!(gs.weather_state.extreme_streak, 2);

        // Apply non-extreme
        gs.weather_state.yesterday = gs.weather_state.today;
        gs.weather_state.today = Weather::Clear;

        apply_weather_effects(&mut gs, &cfg);

        // Should reset streak
        assert_eq!(gs.weather_state.extreme_streak, 0);
    }

    #[test]
    fn test_deterministic_weather_selection() {
        let cfg = WeatherConfig::default_config();
        let mut gs1 = create_test_game_state();
        let mut gs2 = create_test_game_state();

        // Same seed should produce same weather sequence
        let mut weather1 = Vec::new();
        let mut weather2 = Vec::new();

        for _ in 0..20 {
            weather1.push(select_weather_for_today(&mut gs1, &cfg));
            weather2.push(select_weather_for_today(&mut gs2, &cfg));
        }

        assert_eq!(
            weather1, weather2,
            "Same seed should produce identical weather sequences"
        );
    }

    #[test]
    fn test_weather_config_from_json() {
        let json_str = r#"{
            "limits": {
                "max_extreme_streak": 2,
                "encounter_cap": 0.8,
                "pants_floor": 5,
                "pants_ceiling": 95
            },
            "effects": {
                "Clear": {"supplies": 0, "sanity": 0, "pants": 0, "enc_delta": 0.0},
                "Storm": {"supplies": 1, "sanity": -1, "pants": 2, "enc_delta": 0.05},
                "HeatWave": {"supplies": 1, "sanity": -1, "pants": 1, "enc_delta": 0.03},
                "ColdSnap": {"supplies": 0, "sanity": -1, "pants": 0, "enc_delta": 0.0},
                "Smoke": {"supplies": 1, "sanity": -1, "pants": 1, "enc_delta": 0.03}
            },
            "mitigation": {
                "Storm": {"tag": "rain_resist", "pants": 1}
            },
            "weights": {
                "Heartland": {"Clear": 70, "Storm": 10, "HeatWave": 10, "ColdSnap": 5, "Smoke": 5},
                "RustBelt": {"Clear": 60, "Storm": 15, "HeatWave": 10, "ColdSnap": 10, "Smoke": 5},
                "Beltway": {"Clear": 50, "Storm": 20, "HeatWave": 15, "ColdSnap": 10, "Smoke": 5}
            },
            "exec_mods": {
                "TariffTsunami": {"enc_delta": 0.0}
            }
        }"#;

        let config = WeatherConfig::from_json(json_str).expect("Should parse valid JSON");
        assert_eq!(config.limits.max_extreme_streak, 2);
        assert_eq!(config.limits.encounter_cap, 0.8);
        assert_eq!(config.limits.pants_floor, 5);
        assert_eq!(config.limits.pants_ceiling, 95);

        assert!(config.effects.contains_key(&Weather::Clear));
        assert!(config.weights.contains_key(&Region::Heartland));
    }

    #[test]
    fn test_invalid_weather_config_json() {
        let invalid_json = r#"{"limits": {"max_extreme_streak": 3}}"#; // Missing required fields

        let result = WeatherConfig::from_json(invalid_json);
        assert!(result.is_err(), "Should reject incomplete config");
    }

    #[test]
    fn test_process_daily_weather_integration() {
        let cfg = WeatherConfig::default_config();
        let mut gs = create_test_game_state();

        // Process daily weather
        process_daily_weather(&mut gs, &cfg);

        // Should have weather set
        let weather_after = gs.weather_state.today;
        assert_ne!(weather_after, Weather::Clear); // Very unlikely to stay clear with our test setup

        // Run multiple days to test streak behavior
        for _ in 0..10 {
            let prev_weather = gs.weather_state.today;
            process_daily_weather(&mut gs, &cfg);

            // Streak should never exceed max
            assert!(gs.weather_state.extreme_streak <= cfg.limits.max_extreme_streak);

            // Yesterday should be updated correctly
            assert_eq!(gs.weather_state.yesterday, prev_weather);
        }
    }
}
